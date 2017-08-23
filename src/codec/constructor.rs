use std::{char, str, u8};

use bytes::Bytes;
use chrono::{DateTime, TimeZone, Utc};
use codec::Decode;
use nom::{IResult, be_f32, be_f64, be_i16, be_i32, be_i64, be_i8, be_u16, be_u32, be_u64, be_u8};
use types::{ByteStr, Null, Symbol};
use uuid::Uuid;

pub struct Constructor<T> {
    decode: fn(&[u8]) -> IResult<&[u8], T, u32>,
}

impl<T> Clone for Constructor<T> {
    fn clone(&self) -> Self {
        Constructor {
            decode: self.decode
        }
    }
}

impl<T> Constructor<T> {
    pub fn decode<'a>(&self, bytes: &'a[u8]) -> IResult<&'a[u8], T, u32> {
        (self.decode)(bytes)
    }
}

impl Constructor<Null> {
    fn null(bytes: &[u8]) -> IResult<&[u8], Null, u32> {
        IResult::Done(bytes, Null)
    }
}

impl Decode for Constructor<Null> {
    named!(decode<Self>, map!(tag!([0x40u8]), |_| Constructor { decode: Constructor::<Null>::null }));
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

impl Decode for Constructor<bool> {
    named!(decode<Constructor<bool>>, alt!(
        map!(tag!([0x56u8]), |_| Constructor { decode: Constructor::<bool>::fixed1 }) |
        map!(tag!([0x41u8]), |_| Constructor { decode: Constructor::<bool>::true_fixed0 }) |
        map!(tag!([0x42u8]), |_| Constructor { decode: Constructor::<bool>::false_fixed0 })
    ));
}

impl Decode for Constructor<u8> {
    named!(decode<Constructor<u8>>, map!(tag!([0x50u8]), |_| (Constructor { decode: be_u8 })));
}

impl Decode for Constructor<u16> {
    named!(decode<Constructor<u16>>, map!(tag!([0x60u8]), |_| (Constructor { decode: be_u16 })));
}

// u32
impl Constructor<u32> {
    named!(small<u32>, map!(be_u8, |i| i as u32));

    fn zero(bytes: &[u8]) -> IResult<&[u8], u32, u32> {
        IResult::Done(bytes, 0)
    }
}

impl Decode for Constructor<u32> {
    named!(decode<Constructor<u32>>, alt!(
        map!(tag!([0x70u8]), |_| Constructor { decode: be_u32 }) |
        map!(tag!([0x52u8]), |_| Constructor { decode: Constructor::<u32>::small }) |
        map!(tag!([0x43u8]), |_| Constructor { decode: Constructor::<u32>::zero })
    ));
}

// u64
impl Constructor<u64> {
    named!(small<u64>, map!(be_u8, |i| i as u64));

    fn zero(bytes: &[u8]) -> IResult<&[u8], u64, u32> {
        IResult::Done(bytes, 0)
    }
}

impl Decode for Constructor<u64> {
    named!(decode<Constructor<u64>>, alt!(
        map!(tag!([0x80u8]), |_| Constructor { decode: be_u64 }) |
        map!(tag!([0x53u8]), |_| Constructor { decode: Constructor::<u64>::small }) |
        map!(tag!([0x44u8]), |_| Constructor { decode: Constructor::<u64>::zero })
    ));
}

impl Decode for Constructor<i8> {
    named!(decode<Constructor<i8>>, map!(tag!([0x51u8]), |_| (Constructor { decode: be_i8 })));
}

impl Decode for Constructor<i16> {
    named!(decode<Constructor<i16>>, map!(tag!([0x61u8]), |_| (Constructor { decode: be_i16 })));
}

// i32
impl Constructor<i32> {
    named!(small<i32>, map!(be_i8, |i| i as i32));
}

impl Decode for Constructor<i32> {
    named!(decode<Constructor<i32>>, alt!(
        map!(tag!([0x71u8]), |_| Constructor { decode: be_i32 }) |
        map!(tag!([0x54u8]), |_| Constructor { decode: Constructor::<i32>::small })
    ));
}

// i64
impl Constructor<i64> {
    named!(small<i64>, map!(be_i8, |i| i as i64));
}

impl Decode for Constructor<i64> {
    named!(decode<Constructor<i64>>, alt!(
        map!(tag!([0x81u8]), |_| Constructor { decode: be_i64 } ) |
        map!(tag!([0x55u8]), |_| Constructor { decode: Constructor::<i64>::small })
    ));
}

impl Decode for Constructor<f32> {
    named!(decode<Constructor<f32>>, map!(tag!([0x72u8]), |_| Constructor { decode: be_f32 } ));
}

impl Decode for Constructor<f64> {
    named!(decode<Constructor<f64>>, map!(tag!([0x82u8]), |_| Constructor { decode: be_f64 } ));
}

// char
impl Constructor<char> {
    named!(from_u32<char>, map_opt!(be_u32, char::from_u32));
}

impl Decode for Constructor<char> {
    named!(decode<Constructor<char>>, map!(tag!([0x73u8]), |_| Constructor { decode: Constructor::<char>::from_u32 } ));
}

// DateTime<Utc>
impl Constructor<DateTime<Utc>> {
    named!(from_millis<DateTime<Utc>>, map!(be_i64, datetime_from_millis));
}

impl Decode for Constructor<DateTime<Utc>> {
    named!(decode<Constructor<DateTime<Utc>>>, map!(tag!([0x83u8]), |_| Constructor { decode: Constructor::<DateTime<Utc>>::from_millis }));
}

impl Constructor<Uuid> {
    named!(from_bytes<Uuid>, map_res!(take!(16), Uuid::from_bytes));
}

impl Decode for Constructor<Uuid> {
    named!(decode<Constructor<Uuid>>, map!(tag!([0x98u8]), |_| Constructor { decode: Constructor::<Uuid>::from_bytes }));
}

// Bytes
impl Constructor<Bytes> {
    named!(short<Bytes>, do_parse!(bytes: length_bytes!(be_u8) >> (Bytes::from(bytes))));
    named!(long<Bytes>, do_parse!(bytes: length_bytes!(be_u32) >> (Bytes::from(bytes))));
}

impl Decode for Constructor<Bytes> {
    named!(decode<Constructor<Bytes>>, alt!(
        map!(tag!([0xA0u8]), |_| Constructor { decode: Constructor::<Bytes>::short } ) |
        map!(tag!([0xB0u8]), |_| Constructor { decode: Constructor::<Bytes>::long } )
    ));
}

// ByteStr
impl Constructor<ByteStr> {
    named!(short<ByteStr>, do_parse!(string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (ByteStr::from(string))));
    named!(long<ByteStr>, do_parse!(string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (ByteStr::from(string))));
}

impl Decode for Constructor<ByteStr> {
    named!(decode<Constructor<ByteStr>>, alt!(
        map!(tag!([0xA1u8]), |_| Constructor { decode: Constructor::<ByteStr>::short } ) |
        map!(tag!([0xB1u8]), |_| Constructor { decode: Constructor::<ByteStr>::long } )
    ));
}

// Symbol
impl Constructor<Symbol> {
    named!(short<Symbol>, do_parse!(string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (Symbol::from(string))));
    named!(long<Symbol>, do_parse!(string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (Symbol::from(string))));
}

impl Decode for Constructor<Symbol> {
    named!(decode<Constructor<Symbol>>, alt!(
        map!(tag!([0xA3u8]), |_| Constructor { decode: Constructor::<Symbol>::short } ) |
        map!(tag!([0xB3u8]), |_| Constructor { decode: Constructor::<Symbol>::long } )
    ));
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
