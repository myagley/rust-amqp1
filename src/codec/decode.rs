use std::{char, str, u8};

use bytes::Bytes;
use chrono::{DateTime, TimeZone, Utc};
use codec::{Constructor, Decode};
use framing::{AmqpFrame, Frame, AMQP_TYPE, HEADER_LEN};
use nom::{ErrorKind, IResult, be_f32, be_f64, be_i16, be_i32, be_i64, be_i8, be_u16, be_u32, be_u64, be_u8};
use types::{ByteStr, Null, Symbol, Variant};
use uuid::Uuid;

pub const INVALID_FRAME: u32 = 0x0001;

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

impl Constructor<Null> {
    fn null(bytes: &[u8]) -> IResult<&[u8], Null, u32> {
        IResult::Done(bytes, Null)
    }
}

impl Decode for Null {
    named!(constructor<Constructor<Null>>, map!(tag!([0x40u8]), |_| Constructor { decode: Constructor::<Null>::null }));

    named!(decode<Null>, map!(tag!([0x40u8]), |_| Null));
}

// Bool
impl Constructor<bool> {
    named!(fixed1<bool>, alt!(
        map!(tag!([0x00u8]), |_| false) |
        map!(tag!([0x01u8]), |_| true)
    ));

    fn false_fixed0(bytes: &[u8]) -> IResult<&[u8], bool, u32> {
        IResult::Done(bytes, false)
    }

    fn true_fixed0(bytes: &[u8]) -> IResult<&[u8], bool, u32> {
        IResult::Done(bytes, true)
    }
}

impl Decode for bool {
    named!(constructor<Constructor<bool>>, alt!(
        map!(tag!([0x56u8]), |_| Constructor { decode: Constructor::<bool>::fixed1 }) |
        map!(tag!([0x41u8]), |_| Constructor { decode: Constructor::<bool>::true_fixed0 }) |
        map!(tag!([0x42u8]), |_| Constructor { decode: Constructor::<bool>::false_fixed0 })
    ));

    named!(decode<bool>, alt!(
        map_res!(tag!([0x56, 0x00]), |_| Ok::<bool, ()>(false)) |
        map_res!(tag!([0x56, 0x01]), |_| Ok::<bool, ()>(true)) |
        map_res!(tag!([0x41]), |_| Result::Ok::<bool, ()>(true)) |
        map_res!(tag!([0x42]), |_| Result::Ok::<bool, ()>(false))
    ));
}

impl Decode for u8 {
    named!(constructor<Constructor<u8>>, map!(tag!([0x50u8]), |_| (Constructor { decode: be_u8 })));

    named!(decode<u8>, do_parse!(tag!([0x50u8]) >> byte: be_u8 >> (byte)));
}

impl Decode for u16 {
    named!(constructor<Constructor<u16>>, map!(tag!([0x60u8]), |_| (Constructor { decode: be_u16 })));

    named!(decode<u16>, do_parse!(tag!([0x60u8]) >> short: be_u16 >> (short)));
}

// u32
impl Constructor<u32> {
    named!(small<u32>, map!(be_u8, |i| i as u32));

    fn zero(bytes: &[u8]) -> IResult<&[u8], u32, u32> {
        IResult::Done(bytes, 0)
    }
}

impl Decode for u32 {
    named!(constructor<Constructor<u32>>, alt!(
        map!(tag!([0x70u8]), |_| Constructor { decode: be_u32 }) |
        map!(tag!([0x52u8]), |_| Constructor { decode: Constructor::<u32>::small }) |
        map!(tag!([0x43u8]), |_| Constructor { decode: Constructor::<u32>::zero })
    ));

    named!(decode<u32>, alt!(
        do_parse!(tag!([0x70u8]) >> uint: be_u32 >> (uint)) |
        do_parse!(tag!([0x52u8]) >> uint: be_u8 >> (uint as u32)) |
        do_parse!(tag!([0x43u8]) >> (0))
    ));
}

// u64
impl Constructor<u64> {
    named!(small<u64>, map!(be_u8, |i| i as u64));

    fn zero(bytes: &[u8]) -> IResult<&[u8], u64, u32> {
        IResult::Done(bytes, 0)
    }
}

impl Decode for u64 {
    named!(constructor<Constructor<u64>>, alt!(
        map!(tag!([0x80u8]), |_| Constructor { decode: be_u64 }) |
        map!(tag!([0x53u8]), |_| Constructor { decode: Constructor::<u64>::small }) |
        map!(tag!([0x44u8]), |_| Constructor { decode: Constructor::<u64>::zero })
    ));

    named!(decode<u64>, alt!(
        do_parse!(tag!([0x80u8]) >> uint: be_u64 >> (uint)) |
        do_parse!(tag!([0x53u8]) >> uint: be_u8 >> (uint as u64)) |
        do_parse!(tag!([0x44u8]) >> (0))
    ));
}

impl Decode for i8 {
    named!(constructor<Constructor<i8>>, map!(tag!([0x51u8]), |_| (Constructor { decode: be_i8 })));

    named!(decode<i8>, do_parse!(tag!([0x51u8]) >> byte: be_i8 >> (byte)));
}

impl Decode for i16 {
    named!(constructor<Constructor<i16>>, map!(tag!([0x61u8]), |_| (Constructor { decode: be_i16 })));

    named!(decode<i16>, do_parse!(tag!([0x61u8]) >> short: be_i16 >> (short)));
}

// i32
impl Constructor<i32> {
    named!(small<i32>, map!(be_i8, |i| i as i32));
}

impl Decode for i32 {
    named!(constructor<Constructor<i32>>, alt!(
        map!(tag!([0x71u8]), |_| Constructor { decode: be_i32 }) |
        map!(tag!([0x54u8]), |_| Constructor { decode: Constructor::<i32>::small })
    ));

    named!(decode<i32>, alt!(
        do_parse!(tag!([0x71u8]) >> int: be_i32 >> (int)) |
        do_parse!(tag!([0x54u8]) >> int: be_i8 >> (int as i32))
    ));
}

// i64
impl Constructor<i64> {
    named!(small<i64>, map!(be_i8, |i| i as i64));
}

impl Decode for i64 {
    named!(constructor<Constructor<i64>>, alt!(
        map!(tag!([0x81u8]), |_| Constructor { decode: be_i64 } ) |
        map!(tag!([0x55u8]), |_| Constructor { decode: Constructor::<i64>::small })
    ));

    named!(decode<i64>, alt!(
        do_parse!(tag!([0x81u8]) >> long: be_i64 >> (long)) |
        do_parse!(tag!([0x55u8]) >> long: be_i8 >> (long as i64))
    ));
}

impl Decode for f32 {
    named!(constructor<Constructor<f32>>, map!(tag!([0x72u8]), |_| Constructor { decode: be_f32 } ));

    named!(decode<f32>, do_parse!(tag!([0x72u8]) >> float: be_f32 >> (float)));
}

impl Decode for f64 {
    named!(constructor<Constructor<f64>>, map!(tag!([0x82u8]), |_| Constructor { decode: be_f64 } ));

    named!(decode<f64>, do_parse!(tag!([0x82u8]) >> double: be_f64 >> (double)));
}

// char
impl Constructor<char> {
    named!(from_u32<char>, map_opt!(be_u32, char::from_u32));
}

impl Decode for char {
    named!(constructor<Constructor<char>>, map!(tag!([0x73u8]), |_| Constructor { decode: Constructor::<char>::from_u32 } ));

    named!(decode<char>, map_opt!(do_parse!(tag!([0x73u8]) >> int: be_u32 >> (int)), |c| char::from_u32(c)));
}

impl Constructor<DateTime<Utc>> {
    named!(from_millis<DateTime<Utc>>, map!(be_i64, datetime_from_millis));
}

impl Decode for DateTime<Utc> {
    named!(constructor<Constructor<DateTime<Utc>>>, map!(tag!([0x83u8]), |_| Constructor { decode: Constructor::<DateTime<Utc>>::from_millis }));

    named!(decode<DateTime<Utc>>, do_parse!(tag!([0x83u8]) >> timestamp: be_i64 >> (datetime_from_millis(timestamp))));
}

impl Constructor<Uuid> {
    named!(from_bytes<Uuid>, map_res!(take!(16), Uuid::from_bytes));
}

impl Decode for Uuid {
    named!(constructor<Constructor<Uuid>>, map!(tag!([0x98u8]), |_| Constructor { decode: Constructor::<Uuid>::from_bytes }));

    named!(decode<Uuid>, do_parse!(tag!([0x98u8]) >> uuid: map_res!(take!(16), Uuid::from_bytes) >> (uuid)));
}

// Bytes
impl Constructor<Bytes> {
    named!(short<Bytes>, do_parse!(bytes: length_bytes!(be_u8) >> (Bytes::from(bytes))));
    named!(long<Bytes>, do_parse!(bytes: length_bytes!(be_u32) >> (Bytes::from(bytes))));
}

impl Decode for Bytes {
    named!(constructor<Constructor<Bytes>>, alt!(
        map!(tag!([0xA0u8]), |_| Constructor { decode: Constructor::<Bytes>::short } ) |
        map!(tag!([0xB0u8]), |_| Constructor { decode: Constructor::<Bytes>::long } )
    ));

    named!(decode<Bytes>, alt!(
        do_parse!(tag!([0xA0u8]) >> bytes: length_bytes!(be_u8) >> (Bytes::from(bytes))) |
        do_parse!(tag!([0xB0u8]) >> bytes: length_bytes!(be_u32) >> (Bytes::from(bytes)))
    ));
}

impl Constructor<ByteStr> {
    named!(short<ByteStr>, do_parse!(string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (ByteStr::from(string))));
    named!(long<ByteStr>, do_parse!(string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (ByteStr::from(string))));
}

impl Decode for ByteStr {
    named!(constructor<Constructor<ByteStr>>, alt!(
        map!(tag!([0xA1u8]), |_| Constructor { decode: Constructor::<ByteStr>::short } ) |
        map!(tag!([0xB1u8]), |_| Constructor { decode: Constructor::<ByteStr>::long } )
    ));

    named!(decode<ByteStr>, alt!(
        do_parse!(tag!([0xA1u8]) >> string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (ByteStr::from(string))) |
        do_parse!(tag!([0xB1u8]) >> string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (ByteStr::from(string)))
    ));
}

impl Constructor<Symbol> {
    named!(short<Symbol>, do_parse!(string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (Symbol::from(string))));
    named!(long<Symbol>, do_parse!(string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (Symbol::from(string))));
}

impl Decode for Symbol {
    named!(constructor<Constructor<Symbol>>, alt!(
        map!(tag!([0xA3u8]), |_| Constructor { decode: Constructor::<Symbol>::short } ) |
        map!(tag!([0xB3u8]), |_| Constructor { decode: Constructor::<Symbol>::long } )
    ));

    named!(decode<Symbol>, alt!(
        do_parse!(tag!([0xA3u8]) >> string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (Symbol::from(string))) |
        do_parse!(tag!([0xB3u8]) >> string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (Symbol::from(string)))
    ));
}

impl Decode for Variant {
    fn constructor(bytes: &[u8]) -> IResult<&[u8], Constructor<Variant>, u32> {
        IResult::Done(bytes, Constructor { decode: Variant::decode })
    }

    named!(decode<Variant>, alt!(
        map!(Null::decode, |_| Variant::Null) |
        map!(bool::decode, Variant::Boolean) |
        map!(u8::decode, Variant::Ubyte) |
        map!(u16::decode, Variant::Ushort) |
        map!(u32::decode, Variant::Uint) |
        map!(u64::decode, Variant::Ulong) |
        map!(i8::decode, Variant::Byte) |
        map!(i16::decode, Variant::Short) |
        map!(i32::decode, Variant::Int) |
        map!(i64::decode, Variant::Long) |
        map!(f32::decode, Variant::Float) |
        map!(f64::decode, Variant::Double) |
        map!(char::decode, Variant::Char) |
        map!(DateTime::<Utc>::decode, Variant::Timestamp) |
        map!(Uuid::decode, Variant::Uuid) |
        map!(Bytes::decode, Variant::Binary) |
        map!(ByteStr::decode, Variant::String) |
        map!(Symbol::decode, Variant::Symbol)
    ));
}

impl Decode for Frame {
    fn constructor(bytes: &[u8]) -> IResult<&[u8], Constructor<Frame>, u32> {
        IResult::Done(bytes, Constructor { decode: Frame::decode })
    }

    named!(decode<Frame>,
        do_parse!(
            size: be_u32 >>
            error_if!(size < HEADER_LEN as u32, INVALID_FRAME) >>

            doff: be_u8 >>
            error_if!(doff < 2, INVALID_FRAME) >>

            frame: alt!(
                // AMQP Frame
                do_parse!(
                    typ:  tag!([AMQP_TYPE]) >>  // Amqp frame Type

                    channel_id: be_u16 >>
                    extended_header: map!(take!(doff as u32 * 4 - 8), Bytes::from)  >>
                    body: map!(take!(size - doff as u32 * 4), Bytes::from) >>

                    (Frame::Amqp(AmqpFrame::new(channel_id, body)))
                )
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
}
