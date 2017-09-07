use std::{char, str, u8};

use bytes::Bytes;
use chrono::{DateTime, TimeZone, Utc};
use codec::{self, Decode, Decode2};
use framing::{AmqpFrame, Frame, HEADER_LEN};
use nom::{ErrorKind, IResult, be_f32, be_f64, be_i16, be_i32, be_i64, be_i8, be_u16, be_u32, be_u64, be_u8};
use types::{self, ByteStr, Null, Symbol, Variant, Descriptor};
use uuid::Uuid;
use ordered_float::OrderedFloat;

pub const INVALID_FRAME: u32 = 0x0001;
pub const INVALID_FORMATCODE: u32 = 0x0002;
pub const INVALID_DESCRIPTOR: u32 = 0x0003;

macro_rules! error_if (
  ($i:expr, $cond:expr, $code:expr) => (
    {
      if $cond {
        IResult::Error(error_code!(ErrorKind::Custom($code)))
      } else {
        IResult::Done($i, ())
      }
    }
  );
  ($i:expr, $cond:expr, $err:expr) => (
    error!($i, $cond, $err);
  );
);

#[macro_export]
macro_rules! validate_code (
  ($format:ident, $code:expr) => (
    {
      if $format != $code {
        return IResult::Error(error_code!(ErrorKind::Custom(::codec::INVALID_FORMATCODE)));
      }
    }
  );
);

impl<T: Decode2> Decode for T {
    named!(decode<T>, do_parse!(fmt: be_u8 >> v: call!(T::decode_with_format, fmt) >> (v)));
}

impl Decode2 for Null {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], Null> {
        validate_code!(format, codec::FORMATCODE_NULL);
        IResult::Done(input, Null)
    }
}

impl Decode2 for bool {
    named_args!(decode_with_format(format: u8) <bool>, switch!(value!(format),
            codec::FORMATCODE_BOOLEAN => map!(be_u8, |b| b != 0) |
            codec::FORMATCODE_BOOLEAN_TRUE => value!(true) | 
            codec::FORMATCODE_BOOLEAN_FALSE => value!(false)
        )
    );
}

impl Decode2 for u8 {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], u8> {
        validate_code!(format, codec::FORMATCODE_UBYTE);
        be_u8(input)
    }
}

impl Decode2 for u16 {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], u16> {
        validate_code!(format, codec::FORMATCODE_USHORT);
        be_u16(input)
    }
}

impl Decode2 for u32 {
    named_args!(decode_with_format(format: u8) <u32>, switch!(value!(format),
        codec::FORMATCODE_UINT => call!(be_u32) |
        codec::FORMATCODE_SMALLUINT => map!(be_u8, |v| v as u32) | 
        codec::FORMATCODE_UINT_0 => value!(0)
    ));
}

impl Decode2 for u64 {
    named_args!(decode_with_format(format: u8) <u64>, switch!(value!(format),
        codec::FORMATCODE_ULONG => call!(be_u64) |
        codec::FORMATCODE_SMALLULONG => map!(be_u8, |v| v as u64) | 
        codec::FORMATCODE_ULONG_0 => value!(0)
    ));
}

impl Decode2 for i8 {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], i8> {
        validate_code!(format, codec::FORMATCODE_BYTE);
        be_i8(input)
    }
}

impl Decode2 for i16 {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], i16> {
        validate_code!(format, codec::FORMATCODE_SHORT);
        be_i16(input)
    }
}

impl Decode2 for i32 {
    // todo: hand-roll?
    // fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], i32> {
    //     match format {
    //         codec::FORMATCODE_INT => be_i32(input),
    //         codec::FORMATCODE_SMALLINT => map!(input, be_i8, |v| v as i32),
    //         _ => IResult::Error(error_code!(ErrorKind::Custom(INVALID_FORMATCODE)))
    //     }
    // }

    named_args!(decode_with_format(format: u8) <i32>, switch!(value!(format),
        codec::FORMATCODE_INT => call!(be_i32) |
        codec::FORMATCODE_SMALLINT => map!(be_i8, |v| v as i32)
    ));
}

impl Decode2 for i64 {
    named_args!(decode_with_format(format: u8) <i64>, switch!(value!(format),
        codec::FORMATCODE_LONG => call!(be_i64) |
        codec::FORMATCODE_SMALLLONG => map!(be_i8, |v| v as i64)
    ));
}

impl Decode2 for f32 {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], f32> {
        validate_code!(format, codec::FORMATCODE_FLOAT);
        be_f32(input)
    }
}

impl Decode2 for f64 {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], f64> {
        validate_code!(format, codec::FORMATCODE_DOUBLE);
        be_f64(input)
    }
}

impl Decode2 for char {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], char> {
        validate_code!(format, codec::FORMATCODE_CHAR);
        map_opt!(input, be_u32, |c| char::from_u32(c))
    }
}

impl Decode2 for DateTime<Utc> {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], DateTime<Utc>> {
        validate_code!(format, codec::FORMATCODE_TIMESTAMP);
        map!(input, be_i64, |ts| datetime_from_millis(ts))
    }
}

impl Decode2 for Uuid {
    fn decode_with_format(input: &[u8], format: u8) -> IResult<&[u8], Uuid> {
        validate_code!(format, codec::FORMATCODE_UUID);
        map_res!(input, take!(16), Uuid::from_bytes)
    }
}

impl Decode2 for Bytes {
    named_args!(decode_with_format(format: u8) <Bytes>, switch!(value!(format),
        codec::FORMATCODE_BINARY8 => map!(length_bytes!(be_u8), |v| Bytes::from(v)) |
        codec::FORMATCODE_BINARY32 => map!(length_bytes!(be_u32), |v| Bytes::from(v))
    ));
}

impl Decode2 for ByteStr {
    named_args!(decode_with_format(format: u8) <ByteStr>, switch!(value!(format),
        codec::FORMATCODE_STRING8 => map_res!(length_bytes!(be_u8), |v| str::from_utf8(v).map(|s| ByteStr::from(s))) |
        codec::FORMATCODE_STRING32 => map_res!(length_bytes!(be_u32), |v| str::from_utf8(v).map(|s| ByteStr::from(s)))
    ));
}

impl Decode2 for Symbol {
    named_args!(decode_with_format(format: u8) <Symbol>, switch!(value!(format),
        codec::FORMATCODE_SYMBOL8 => map_res!(length_bytes!(be_u8), |v| str::from_utf8(v).map(|s| Symbol::from(s))) |
        codec::FORMATCODE_SYMBOL32 => map_res!(length_bytes!(be_u32), |v| str::from_utf8(v).map(|s| Symbol::from(s)))
    ));
}

impl Decode2 for Variant {
    named_args!(decode_with_format(format: u8) <Variant>, switch!(value!(format),
        codec::FORMATCODE_NULL => value!(Variant::Null) |
        codec::FORMATCODE_BOOLEAN => map!(call!(bool::decode_with_format, format), Variant::Boolean) |
        codec::FORMATCODE_BOOLEAN_FALSE => value!(Variant::Boolean(false)) |
        codec::FORMATCODE_BOOLEAN_TRUE => value!(Variant::Boolean(true)) |
        codec::FORMATCODE_UINT_0 => value!(Variant::Uint(0)) |
        codec::FORMATCODE_ULONG_0 => value!(Variant::Ulong(0)) |
        codec::FORMATCODE_UBYTE => map!(call!(u8::decode_with_format, format), Variant::Ubyte) |
        codec::FORMATCODE_USHORT => map!(call!(u16::decode_with_format, format), Variant::Ushort) |
        codec::FORMATCODE_UINT => map!(call!(u32::decode_with_format, format), Variant::Uint) |
        codec::FORMATCODE_ULONG => map!(call!(u64::decode_with_format, format), Variant::Ulong) |
        codec::FORMATCODE_BYTE => map!(call!(i8::decode_with_format, format), Variant::Byte) |
        codec::FORMATCODE_SHORT => map!(call!(i16::decode_with_format, format), Variant::Short) |
        codec::FORMATCODE_INT => map!(call!(i32::decode_with_format, format), Variant::Int) |
        codec::FORMATCODE_LONG => map!(call!(i64::decode_with_format, format), Variant::Long) |
        codec::FORMATCODE_SMALLUINT => map!(call!(u32::decode_with_format, format), Variant::Uint) |
        codec::FORMATCODE_SMALLULONG => map!(call!(u64::decode_with_format, format), Variant::Ulong) |
        codec::FORMATCODE_SMALLINT => map!(call!(i32::decode_with_format, format), Variant::Int) |
        codec::FORMATCODE_SMALLLONG => map!(call!(i64::decode_with_format, format), Variant::Long) |
        codec::FORMATCODE_FLOAT => map!(call!(f32::decode_with_format, format), |v| Variant::Float(OrderedFloat(v))) |
        codec::FORMATCODE_DOUBLE => map!(call!(f64::decode_with_format, format), |v| Variant::Double(OrderedFloat(v))) |
        // codec::FORMATCODE_DECIMAL32 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_DECIMAL64 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_DECIMAL128 => map!(call!(x::decode_with_format, format), Variant::X) |
        codec::FORMATCODE_CHAR => map!(call!(char::decode_with_format, format), Variant::Char) |
        codec::FORMATCODE_TIMESTAMP => map!(call!(DateTime::<Utc>::decode_with_format, format), Variant::Timestamp) |
        codec::FORMATCODE_UUID => map!(call!(Uuid::decode_with_format, format), Variant::Uuid) |
        codec::FORMATCODE_BINARY8 => map!(call!(Bytes::decode_with_format, format), Variant::Binary) |
        codec::FORMATCODE_BINARY32 => map!(call!(Bytes::decode_with_format, format), Variant::Binary) |
        codec::FORMATCODE_STRING8 => map!(call!(ByteStr::decode_with_format, format), Variant::String) |
        codec::FORMATCODE_STRING32 => map!(call!(ByteStr::decode_with_format, format), Variant::String) |
        codec::FORMATCODE_SYMBOL8 => map!(call!(Symbol::decode_with_format, format), Variant::Symbol) |
        codec::FORMATCODE_SYMBOL32 => map!(call!(Symbol::decode_with_format, format), Variant::Symbol)
        // codec::FORMATCODE_LIST0 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_LIST8 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_LIST32 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_MAP8 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_MAP32 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_ARRAY8 => map!(call!(x::decode_with_format, format), Variant::X) |
        // codec::FORMATCODE_ARRAY32 => map!(call!(x::decode_with_format, format), Variant::X) |
    ));
}

impl<T: Decode2> Decode2 for Option<T> {
     named_args!(decode_with_format(format: u8) <Option<T>>, switch!(value!(format),
            codec::FORMATCODE_NULL => value!(None) | 
            _ => map!(call!(T::decode_with_format, format), |v| Some(v))));
}

impl Decode2 for Descriptor {
    named_args!(decode_with_format(format: u8) <Descriptor>, switch!(value!(format),
        codec::FORMATCODE_SMALLULONG => map!(call!(u64::decode_with_format, format), Descriptor::Ulong) |
        codec::FORMATCODE_ULONG => map!(call!(u64::decode_with_format, format), Descriptor::Ulong) |
        codec::FORMATCODE_SYMBOL8 => map!(call!(Symbol::decode_with_format, format), Descriptor::Symbol) |
        codec::FORMATCODE_SYMBOL32 => map!(call!(Symbol::decode_with_format, format), Descriptor::Symbol)
     ));
}

named_args!(parse_amqp_frame(size: u32, doff: u8) <Frame>, do_parse!(
    channel_id: be_u16 >>
    extended_header: take!(doff as u32 * 4 - 8) >>
    performative: call!(types::Frame::decode) >>
    body: map!(take!(size - doff as u32 * 4), Bytes::from) >>
    (Frame::Amqp(AmqpFrame::new(channel_id, performative, body)))));

impl Decode for Frame {
    named!(decode<Frame>,
        do_parse!(
            size: be_u32 >>
            error_if!(size < HEADER_LEN as u32, INVALID_FRAME) >>

            doff: be_u8 >>
            error_if!(doff < 2, INVALID_FRAME) >>

            frame: switch!(be_u8,
                FRAME_TYPE_AMQP => call!(parse_amqp_frame, size, doff) |
                FRAME_TYPE_SASL => value!(Frame::Sasl())
            ) >>
            (frame)
        )
    );
}

fn datetime_from_millis(millis: i64) -> DateTime<Utc> {
    let seconds = millis / 1000;
    if seconds < 0 {
        // In order to handle time before 1970 correctly, we need to subtract a second
        // and use the nanoseconds field to add it back. This is a result of the nanoseconds
        // parameter being u32
        let nanoseconds = ((1000 + (millis - (seconds * 1000))) * 1_000_000).abs() as u32;
        Utc.timestamp(seconds - 1, nanoseconds)
    } else {
        let nanoseconds = ((millis - (seconds * 1000)) * 1_000_000).abs() as u32;
        Utc.timestamp(seconds, nanoseconds)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bytes::{BufMut, BytesMut};
    use codec::Encode;

    const LOREM: &str = include_str!("lorem.txt");

    macro_rules! decode_tests {
        ($($name:ident: $kind:ident, $test:expr, $expected:expr,)*) => {
        $(
            #[test]
            fn $name() {
                let b1 = &mut BytesMut::with_capacity(0);
                ($test).encode(b1);
                assert_eq!(Ok($expected), $kind::decode(b1).to_full_result());
            }
        )*
        }
    }

    decode_tests! {
        ubyte: u8, 255_u8, 255_u8,
        ushort: u16, 350_u16, 350_u16,

        uint_zero: u32, 0_u32, 0_u32,
        uint_small: u32, 128_u32, 128_u32,
        uint_big: u32, 2147483647_u32, 2147483647_u32,

        ulong_zero: u64, 0_u64, 0_u64,
        ulong_small: u64, 128_u64, 128_u64,
        uulong_big: u64, 2147483649_u64, 2147483649_u64,

        byte: i8, -128_i8, -128_i8,
        short: i16, -255_i16, -255_i16,

        int_zero: i32, 0_i32, 0_i32,
        int_small: i32, -50000_i32, -50000_i32,
        int_neg: i32, -128_i32, -128_i32,

        long_zero: i64, 0_i64, 0_i64,
        long_big: i64, -2147483647_i64, -2147483647_i64,
        long_small: i64, -128_i64, -128_i64,

        float: f32, 1.234_f32, 1.234_f32,
        double: f64, 1.234_f64, 1.234_f64,

        test_char: char, '💯', '💯',

        uuid: Uuid, Uuid::from_bytes(&[4, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87]).expect("parse error"),
            Uuid::parse_str("0436430c2b02624c2032570501212b57").expect("parse error"),

        binary_short: Bytes, Bytes::from(&[4u8, 5u8][..]), Bytes::from(&[4u8, 5u8][..]),
        binary_long: Bytes, Bytes::from(&[4u8; 500][..]), Bytes::from(&[4u8; 500][..]),

        string_short: ByteStr, ByteStr::from("Hello there"), ByteStr::from("Hello there"),
        string_long: ByteStr, ByteStr::from(LOREM), ByteStr::from(LOREM),

        symbol_short: Symbol, Symbol::from("Hello there"), Symbol::from("Hello there"),
        symbol_long: Symbol, Symbol::from(LOREM), Symbol::from(LOREM),

        variant_ubyte: Variant, Variant::Ubyte(255_u8), Variant::Ubyte(255_u8),
        variant_ushort: Variant, Variant::Ushort(350_u16), Variant::Ushort(350_u16),

        variant_uint_zero: Variant, Variant::Uint(0_u32), Variant::Uint(0_u32),
        variant_uint_small: Variant, Variant::Uint(128_u32), Variant::Uint(128_u32),
        variant_uint_big: Variant, Variant::Uint(2147483647_u32), Variant::Uint(2147483647_u32),

        variant_ulong_zero: Variant, Variant::Ulong(0_u64), Variant::Ulong(0_u64),
        variant_ulong_small: Variant, Variant::Ulong(128_u64), Variant::Ulong(128_u64),
        variant_ulong_big: Variant, Variant::Ulong(2147483649_u64), Variant::Ulong(2147483649_u64),

        variant_byte: Variant, Variant::Byte(-128_i8), Variant::Byte(-128_i8),
        variant_short: Variant, Variant::Short(-255_i16), Variant::Short(-255_i16),

        variant_int_zero: Variant, Variant::Int(0_i32), Variant::Int(0_i32),
        variant_int_small: Variant, Variant::Int(-50000_i32), Variant::Int(-50000_i32),
        variant_int_neg: Variant, Variant::Int(-128_i32), Variant::Int(-128_i32),

        variant_long_zero: Variant, Variant::Long(0_i64), Variant::Long(0_i64),
        variant_long_big: Variant, Variant::Long(-2147483647_i64), Variant::Long(-2147483647_i64),
        variant_long_small: Variant, Variant::Long(-128_i64), Variant::Long(-128_i64),

        variant_float: Variant, Variant::Float(1.234_f32), Variant::Float(1.234_f32),
        variant_double: Variant, Variant::Double(1.234_f64), Variant::Double(1.234_f64),

        variant_char: Variant, Variant::Char('💯'), Variant::Char('💯'),

        variant_uuid: Variant, Variant::Uuid(Uuid::from_bytes(&[4, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87]).expect("parse error")),
            Variant::Uuid(Uuid::parse_str("0436430c2b02624c2032570501212b57").expect("parse error")),

        variant_binary_short: Variant, Variant::Binary(Bytes::from(&[4u8, 5u8][..])), Variant::Binary(Bytes::from(&[4u8, 5u8][..])),
        variant_binary_long: Variant, Variant::Binary(Bytes::from(&[4u8; 500][..])), Variant::Binary(Bytes::from(&[4u8; 500][..])),

        variant_string_short: Variant, Variant::String(ByteStr::from("Hello there")), Variant::String(ByteStr::from("Hello there")),
        variant_string_long: Variant, Variant::String(ByteStr::from(LOREM)), Variant::String(ByteStr::from(LOREM)),

        variant_symbol_short: Variant, Variant::Symbol(Symbol::from("Hello there")), Variant::Symbol(Symbol::from("Hello there")),
        variant_symbol_long: Variant, Variant::Symbol(Symbol::from(LOREM)), Variant::Symbol(Symbol::from(LOREM)),
   }

    #[test]
    fn test_null() {
        let mut b = BytesMut::with_capacity(0);
        Null.encode(&mut b);
        let t = Null::decode(&mut b).to_full_result();
        assert_eq!(Ok(Null), t);
    }

    #[test]
    fn test_bool_true() {
        let b1 = &mut BytesMut::with_capacity(0);
        b1.put_u8(0x41);
        assert_eq!(Ok(true), bool::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        b2.put_u8(0x56);
        b2.put_u8(0x01);
        assert_eq!(Ok(true), bool::decode(b2).to_full_result());
    }

    #[test]
    fn test_bool_false() {
        let b1 = &mut BytesMut::with_capacity(0);
        b1.put_u8(0x42u8);
        assert_eq!(Ok(false), bool::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        b2.put_u8(0x56);
        b2.put_u8(0x00);
        assert_eq!(Ok(false), bool::decode(b2).to_full_result());
    }

    /// UTC with a precision of milliseconds. For example, 1311704463521
    /// represents the moment 2011-07-26T18:21:03.521Z.
    #[test]
    fn test_timestamp() {
        let b1 = &mut BytesMut::with_capacity(0);
        let datetime = Utc.ymd(2011, 7, 26).and_hms_milli(18, 21, 3, 521);
        datetime.encode(b1);

        let expected = Utc.ymd(2011, 7, 26).and_hms_milli(18, 21, 3, 521);
        assert_eq!(Ok(expected), DateTime::<Utc>::decode(b1).to_full_result());
    }

    #[test]
    fn test_timestamp_pre_unix() {
        let b1 = &mut BytesMut::with_capacity(0);
        let datetime = Utc.ymd(1968, 7, 26).and_hms_milli(18, 21, 3, 521);
        datetime.encode(b1);

        let expected = Utc.ymd(1968, 7, 26).and_hms_milli(18, 21, 3, 521);
        assert_eq!(Ok(expected), DateTime::<Utc>::decode(b1).to_full_result());
    }

    #[test]
    fn variant_null() {
        let mut b = BytesMut::with_capacity(0);
        Variant::Null.encode(&mut b);
        let t = Variant::decode(&mut b).to_full_result();
        assert_eq!(Ok(Variant::Null), t);
    }

    #[test]
    fn variant_bool_true() {
        let b1 = &mut BytesMut::with_capacity(0);
        b1.put_u8(0x41);
        assert_eq!(
            Ok(Variant::Boolean(true)),
            Variant::decode(b1).to_full_result()
        );

        let b2 = &mut BytesMut::with_capacity(0);
        b2.put_u8(0x56);
        b2.put_u8(0x01);
        assert_eq!(
            Ok(Variant::Boolean(true)),
            Variant::decode(b2).to_full_result()
        );
    }

    #[test]
    fn variant_bool_false() {
        let b1 = &mut BytesMut::with_capacity(0);
        b1.put_u8(0x42u8);
        assert_eq!(
            Ok(Variant::Boolean(false)),
            Variant::decode(b1).to_full_result()
        );

        let b2 = &mut BytesMut::with_capacity(0);
        b2.put_u8(0x56);
        b2.put_u8(0x00);
        assert_eq!(
            Ok(Variant::Boolean(false)),
            Variant::decode(b2).to_full_result()
        );
    }

    /// UTC with a precision of milliseconds. For example, 1311704463521
    /// represents the moment 2011-07-26T18:21:03.521Z.
    #[test]
    fn variant_timestamp() {
        let b1 = &mut BytesMut::with_capacity(0);
        let datetime = Utc.ymd(2011, 7, 26).and_hms_milli(18, 21, 3, 521);
        Variant::Timestamp(datetime).encode(b1);

        let expected = Utc.ymd(2011, 7, 26).and_hms_milli(18, 21, 3, 521);
        assert_eq!(
            Ok(Variant::Timestamp(expected)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_timestamp_pre_unix() {
        let b1 = &mut BytesMut::with_capacity(0);
        let datetime = Utc.ymd(1968, 7, 26).and_hms_milli(18, 21, 3, 521);
        Variant::Timestamp(datetime).encode(b1);

        let expected = Utc.ymd(1968, 7, 26).and_hms_milli(18, 21, 3, 521);
        assert_eq!(
            Ok(Variant::Timestamp(expected)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn option_i8() {
        let b1 = &mut BytesMut::with_capacity(0);
        Some(42i8).encode(b1);

        assert_eq!(
            Ok(Some(42)),
            Option::<i8>::decode(b1).to_full_result()
        );

        let b2 = &mut BytesMut::with_capacity(0);
        let o1: Option<i8> = None;
        o1.encode(b2);

        assert_eq!(
            Ok(None),
            Option::<i8>::decode(b2).to_full_result()
        );
    }

    #[test]
    fn option_string() {
        let b1 = &mut BytesMut::with_capacity(0);
        Some(ByteStr::from("hello")).encode(b1);

        assert_eq!(
            Ok(Some(ByteStr::from("hello"))),
            Option::<ByteStr>::decode(b1).to_full_result()
        );

        let b2 = &mut BytesMut::with_capacity(0);
        let o1: Option<ByteStr> = None;
        o1.encode(b2);

        assert_eq!(
            Ok(None),
            Option::<ByteStr>::decode(b2).to_full_result()
        );
    }
}
