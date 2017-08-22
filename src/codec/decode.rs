use std::{char, str, u8};

use bytes::Bytes;
use chrono::{DateTime, TimeZone, Utc};
use codec::Decode;
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

impl Decode for Null {
    named!(decode<Null>, map_res!(tag!([0x40u8]), |_| Ok::<Null, ()>(Null)));
}

impl Decode for bool {
    named!(decode<bool>, alt!(
        map_res!(tag!([0x56, 0x00]), |_| Ok::<bool, ()>(false)) |
        map_res!(tag!([0x56, 0x01]), |_| Ok::<bool, ()>(true)) |
        map_res!(tag!([0x41]), |_| Result::Ok::<bool, ()>(true)) |
        map_res!(tag!([0x42]), |_| Result::Ok::<bool, ()>(false))
    ));
}

impl Decode for u8 {
    named!(decode<u8>, do_parse!(tag!([0x50u8]) >> byte: be_u8 >> (byte)));
}

impl Decode for u16 {
    named!(decode<u16>, do_parse!(tag!([0x60u8]) >> short: be_u16 >> (short)));
}

impl Decode for u32 {
    named!(decode<u32>, alt!(
        do_parse!(tag!([0x70u8]) >> uint: be_u32 >> (uint)) |
        do_parse!(tag!([0x52u8]) >> uint: be_u8 >> (uint as u32)) |
        do_parse!(tag!([0x43u8]) >> (0))
    ));
}

impl Decode for u64 {
    named!(decode<u64>, alt!(
        do_parse!(tag!([0x80u8]) >> uint: be_u64 >> (uint)) |
        do_parse!(tag!([0x53u8]) >> uint: be_u8 >> (uint as u64)) |
        do_parse!(tag!([0x44u8]) >> (0))
    ));
}

impl Decode for i8 {
    named!(decode<i8>, do_parse!(tag!([0x51u8]) >> byte: be_i8 >> (byte)));
}

impl Decode for i16 {
    named!(decode<i16>, do_parse!(tag!([0x61u8]) >> short: be_i16 >> (short)));
}

impl Decode for i32 {
    named!(decode<i32>, alt!(
        do_parse!(tag!([0x71u8]) >> int: be_i32 >> (int)) |
        do_parse!(tag!([0x54u8]) >> int: be_i8 >> (int as i32))
    ));
}

impl Decode for i64 {
    named!(decode<i64>, alt!(
        do_parse!(tag!([0x81u8]) >> long: be_i64 >> (long)) |
        do_parse!(tag!([0x55u8]) >> long: be_i8 >> (long as i64))
    ));
}

impl Decode for f32 {
    named!(decode<f32>, do_parse!(tag!([0x72u8]) >> float: be_f32 >> (float)));
}

impl Decode for f64 {
    named!(decode<f64>, do_parse!(tag!([0x82u8]) >> double: be_f64 >> (double)));
}

impl Decode for char {
    named!(decode<char>, map_opt!(do_parse!(tag!([0x73u8]) >> int: be_u32 >> (int)), |c| char::from_u32(c)));
}

impl Decode for DateTime<Utc> {
    named!(decode<DateTime<Utc>>, do_parse!(tag!([0x83u8]) >> timestamp: be_i64 >> (datetime_from_millis(timestamp))));
}

impl Decode for Uuid {
    named!(decode<Uuid>, do_parse!(tag!([0x98u8]) >> uuid: map_res!(take!(16), Uuid::from_bytes) >> (uuid)));
}

impl Decode for Bytes {
    named!(decode<Bytes>, alt!(
        do_parse!(tag!([0xA0u8]) >> bytes: length_bytes!(be_u8) >> (Bytes::from(bytes))) |
        do_parse!(tag!([0xB0u8]) >> bytes: length_bytes!(be_u32) >> (Bytes::from(bytes)))
    ));
}

impl Decode for ByteStr {
    named!(decode<ByteStr>, alt!(
        do_parse!(tag!([0xA1u8]) >> string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (ByteStr::from(string))) |
        do_parse!(tag!([0xB1u8]) >> string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (ByteStr::from(string)))
    ));
}

impl Decode for Symbol {
    named!(decode<Symbol>, alt!(
        do_parse!(tag!([0xA3u8]) >> string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (Symbol::from(string))) |
        do_parse!(tag!([0xB3u8]) >> string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (Symbol::from(string)))
    ));
}

impl Decode for Variant {
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

    #[test]
    fn test_ubyte() {
        let b1 = &mut BytesMut::with_capacity(0);
        (255 as u8).encode(b1);
        assert_eq!(Ok(255 as u8), u8::decode(b1).to_full_result());
    }

    #[test]
    fn test_ushort() {
        let b1 = &mut BytesMut::with_capacity(0);
        (350 as u16).encode(b1);
        assert_eq!(Ok(350 as u16), u16::decode(b1).to_full_result());
    }

    #[test]
    fn test_uint() {
        let b1 = &mut BytesMut::with_capacity(0);
        (0 as u32).encode(b1);
        assert_eq!(Ok(0 as u32), u32::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        (128 as u32).encode(b2);
        assert_eq!(Ok(128 as u32), u32::decode(b2).to_full_result());

        let b3 = &mut BytesMut::with_capacity(0);
        (2147483647 as u32).encode(b3);
        assert_eq!(Ok(2147483647 as u32), u32::decode(b3).to_full_result());
    }

    #[test]
    fn test_ulong() {
        let b1 = &mut BytesMut::with_capacity(0);
        (0 as u64).encode(b1);
        assert_eq!(Ok(0 as u64), u64::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        (128 as u64).encode(b2);
        assert_eq!(Ok(128 as u64), u64::decode(b2).to_full_result());

        let b3 = &mut BytesMut::with_capacity(0);
        (2147483649 as u64).encode(b3);
        assert_eq!(Ok(2147483649 as u64), u64::decode(b3).to_full_result());
    }

    #[test]
    fn test_byte() {
        let b1 = &mut BytesMut::with_capacity(0);
        (-128 as i8).encode(b1);
        assert_eq!(Ok(-128 as i8), i8::decode(b1).to_full_result());
    }

    #[test]
    fn test_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        (-255 as i16).encode(b1);
        assert_eq!(Ok(-255 as i16), i16::decode(b1).to_full_result());
    }

    #[test]
    fn test_int() {
        let b1 = &mut BytesMut::with_capacity(0);
        0.encode(b1);
        assert_eq!(Ok(0), i32::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        (-50000).encode(b2);
        assert_eq!(Ok(-50000), i32::decode(b2).to_full_result());

        let b3 = &mut BytesMut::with_capacity(0);
        (-128).encode(b3);
        assert_eq!(Ok(-128), i32::decode(b3).to_full_result());
    }

    #[test]
    fn test_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        (0 as i64).encode(b1);
        assert_eq!(Ok(0 as i64), i64::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        (-2147483647 as i64).encode(b2);
        assert_eq!(Ok(-2147483647 as i64), i64::decode(b2).to_full_result());

        let b3 = &mut BytesMut::with_capacity(0);
        (-128 as i64).encode(b3);
        assert_eq!(Ok(-128 as i64), i64::decode(b3).to_full_result());
    }

    #[test]
    fn test_float() {
        let b1 = &mut BytesMut::with_capacity(0);
        (1.234 as f32).encode(b1);
        assert_eq!(Ok(1.234 as f32), f32::decode(b1).to_full_result());
    }

    #[test]
    fn test_double() {
        let b1 = &mut BytesMut::with_capacity(0);
        (1.234 as f64).encode(b1);
        assert_eq!(Ok(1.234 as f64), f64::decode(b1).to_full_result());
    }

    #[test]
    fn test_char() {
        let b1 = &mut BytesMut::with_capacity(0);
        '💯'.encode(b1);
        assert_eq!(Ok('💯'), char::decode(b1).to_full_result());
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
    fn test_uuid() {
        let b1 = &mut BytesMut::with_capacity(0);
        let bytes = [4, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        let u1 = Uuid::from_bytes(&bytes).expect("parse error");
        u1.encode(b1);

        let expected = Uuid::parse_str("0436430c2b02624c2032570501212b57").expect("parse error");
        assert_eq!(Ok(expected), Uuid::decode(b1).to_full_result());
    }

    #[test]
    fn test_binary_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        let bytes = [4u8, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        Bytes::from(&bytes[..]).encode(b1);

        let expected = [4u8, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        assert_eq!(
            Ok(Bytes::from(&expected[..])),
            Bytes::decode(b1).to_full_result()
        );
    }

    #[test]
    fn test_binary_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        let bytes = [4u8; 500];
        Bytes::from(&bytes[..]).encode(b1);

        let expected = [4u8; 500];
        assert_eq!(
            Ok(Bytes::from(&expected[..])),
            Bytes::decode(b1).to_full_result()
        );
    }

    #[test]
    fn test_string_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        ByteStr::from("Hello there").encode(b1);

        assert_eq!(
            Ok(ByteStr::from("Hello there")),
            ByteStr::decode(b1).to_full_result()
        );
    }

    #[test]
    fn test_string_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        let s1 = ByteStr::from(LOREM);
        s1.encode(b1);

        let expected = ByteStr::from(LOREM);
        assert_eq!(Ok(expected), ByteStr::decode(b1).to_full_result());
    }

    #[test]
    fn test_symbol_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        Symbol::from("Hello there").encode(b1);

        assert_eq!(
            Ok(Symbol::from("Hello there")),
            Symbol::decode(b1).to_full_result()
        );
    }

    #[test]
    fn test_symbol_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        let s1 = Symbol::from(LOREM);
        s1.encode(b1);

        let expected = Symbol::from(LOREM);
        assert_eq!(Ok(expected), Symbol::decode(b1).to_full_result());
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

    #[test]
    fn variant_ubyte() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Ubyte(255).encode(b1);
        assert_eq!(
            Ok(Variant::Ubyte(255)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_ushort() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Ushort(350).encode(b1);
        assert_eq!(
            Ok(Variant::Ushort(350)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_uint() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Uint(0).encode(b1);
        assert_eq!(Ok(Variant::Uint(0)), Variant::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        Variant::Uint(128).encode(b2);
        assert_eq!(Ok(Variant::Uint(128)), Variant::decode(b2).to_full_result());

        let b3 = &mut BytesMut::with_capacity(0);
        Variant::Uint(2147483647).encode(b3);
        assert_eq!(
            Ok(Variant::Uint(2147483647)),
            Variant::decode(b3).to_full_result()
        );
    }

    #[test]
    fn variant_ulong() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Ulong(0).encode(b1);
        assert_eq!(Ok(Variant::Ulong(0)), Variant::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        Variant::Ulong(128).encode(b2);
        assert_eq!(
            Ok(Variant::Ulong(128)),
            Variant::decode(b2).to_full_result()
        );

        let b3 = &mut BytesMut::with_capacity(0);
        Variant::Ulong(2147483649).encode(b3);
        assert_eq!(
            Ok(Variant::Ulong(2147483649)),
            Variant::decode(b3).to_full_result()
        );
    }

    #[test]
    fn variant_byte() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Byte(-128).encode(b1);
        assert_eq!(
            Ok(Variant::Byte(-128)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Short(-255).encode(b1);
        assert_eq!(
            Ok(Variant::Short(-255)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_int() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Int(0).encode(b1);
        assert_eq!(Ok(Variant::Int(0)), Variant::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        Variant::Int(-50000).encode(b2);
        assert_eq!(
            Ok(Variant::Int(-50000)),
            Variant::decode(b2).to_full_result()
        );

        let b3 = &mut BytesMut::with_capacity(0);
        Variant::Int(-128).encode(b3);
        assert_eq!(Ok(Variant::Int(-128)), Variant::decode(b3).to_full_result());
    }

    #[test]
    fn variant_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Ulong(0).encode(b1);
        assert_eq!(Ok(Variant::Ulong(0)), Variant::decode(b1).to_full_result());

        let b2 = &mut BytesMut::with_capacity(0);
        Variant::Long(-2147483647).encode(b2);
        assert_eq!(
            Ok(Variant::Long(-2147483647)),
            Variant::decode(b2).to_full_result()
        );

        let b3 = &mut BytesMut::with_capacity(0);
        Variant::Long(-128).encode(b3);
        assert_eq!(
            Ok(Variant::Long(-128)),
            Variant::decode(b3).to_full_result()
        );
    }

    #[test]
    fn variant_float() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Float(1.234).encode(b1);
        assert_eq!(
            Ok(Variant::Float(1.234)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_double() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Double(1.234).encode(b1);
        assert_eq!(
            Ok(Variant::Double(1.234)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_char() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Char('💯').encode(b1);
        assert_eq!(
            Ok(Variant::Char('💯')),
            Variant::decode(b1).to_full_result()
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
    fn variant_uuid() {
        let b1 = &mut BytesMut::with_capacity(0);
        let bytes = [4, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        let u1 = Uuid::from_bytes(&bytes).expect("parse error");
        Variant::Uuid(u1).encode(b1);

        let expected = Variant::Uuid(
            Uuid::parse_str("0436430c2b02624c2032570501212b57").expect("parse error"),
        );
        assert_eq!(Ok(expected), Variant::decode(b1).to_full_result());
    }

    #[test]
    fn variant_binary_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        let bytes = [4u8, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        Variant::Binary(Bytes::from(&bytes[..])).encode(b1);

        let expected = [4u8, 54, 67, 12, 43, 2, 98, 76, 32, 50, 87, 5, 1, 33, 43, 87];
        assert_eq!(
            Ok(Variant::Binary(Bytes::from(&expected[..]))),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_binary_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        let bytes = [4u8; 500];
        Variant::Binary(Bytes::from(&bytes[..])).encode(b1);

        let expected = [4u8; 500];
        assert_eq!(
            Ok(Variant::Binary(Bytes::from(&expected[..]))),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_string_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::String(ByteStr::from("Hello there")).encode(b1);

        assert_eq!(
            Ok(Variant::String(ByteStr::from("Hello there"))),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_string_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        let s1 = ByteStr::from(LOREM);
        Variant::String(s1).encode(b1);

        let expected = ByteStr::from(LOREM);
        assert_eq!(
            Ok(Variant::String(expected)),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn variant_symbol_short() {
        let b1 = &mut BytesMut::with_capacity(0);
        Variant::Symbol(Symbol::from("Hello there")).encode(b1);

        assert_eq!(
            Ok(Variant::Symbol(Symbol::from("Hello there"))),
            Variant::decode(b1).to_full_result()
        );
    }

    #[test]
    fn symbol_long() {
        let b1 = &mut BytesMut::with_capacity(0);
        let s1 = Symbol::from(LOREM);
        Variant::Symbol(s1).encode(b1);

        let expected = Symbol::from(LOREM);
        assert_eq!(
            Ok(Variant::Symbol(expected)),
            Variant::decode(b1).to_full_result()
        );
    }
}
