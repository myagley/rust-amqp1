use std::{char, str, u8};

use bytes::{BigEndian, BufMut, Bytes, BytesMut};
use chrono::{DateTime, TimeZone, Utc};
use codec::{encode, Decode, Encode};
use nom::{IResult, be_f32, be_f64, be_i16, be_i32, be_i64, be_i8, be_u16, be_u32, be_u64, be_u8};
use types::{ByteStr, Null, Symbol};
use uuid::Uuid;

pub struct Constructor<T> {
    format_code: u8,
    size: fn(&T) -> usize,
    decode: fn(&[u8]) -> IResult<&[u8], T, u32>,
    encode: fn(&T, &mut BytesMut) -> (),
}

fn size0<T>(_: &T) -> usize {
    0
}

fn size1<T>(_: &T) -> usize {
    1
}

fn size2<T>(_: &T) -> usize {
   2 
}

fn size4<T>(_: &T) -> usize {
   4 
}

fn size8<T>(_: &T) -> usize {
   8 
}

fn size16<T>(_: &T) -> usize {
  16 
}

impl<T> Clone for Constructor<T> {
    fn clone(&self) -> Self {
        Constructor {
            format_code: self.format_code,
            size: self.size,
            decode: self.decode,
            encode: self.encode,
        }
    }
}

impl<T> Encode for Constructor<T> {
    fn encoded_size(&self) -> usize {
        1
    }

    fn encode(&self, buf: &mut BytesMut) {
        encode::ensure_capacity(self, buf);
        buf.put_u8(self.format_code);
    }
}

impl<T> Constructor<T> {
    pub fn decode_value<'a>(&self, bytes: &'a[u8]) -> IResult<&'a[u8], T, u32> {
        (self.decode)(bytes)
    }

    pub fn encode_value(&self, t: &T, bytes: &mut BytesMut) {
        let size = self.encoded_value_size(t);
        if bytes.remaining_mut() < size {
            bytes.reserve(size);
        }
        (self.encode)(t, bytes)
    }

    pub fn encoded_value_size(&self, t: &T) -> usize {
        (self.size)(t)
    }
}

impl Constructor<Null> {
    fn decode_null(bytes: &[u8]) -> IResult<&[u8], Null, u32> {
        IResult::Done(bytes, Null)
    }

    fn encode_null(_: &Null, _: &mut BytesMut) { }
}

impl Decode for Constructor<Null> {
    named!(decode<Self>, map!(tag!([0x40u8]), |_|
        Constructor {
            format_code: 0x40u8,
            size: size0::<Null>,
            decode: Constructor::<Null>::decode_null,
            encode: Constructor::<Null>::encode_null,
        }
    ));
}

// Bool
impl Constructor<bool> {
    named!(decode_fixed1<bool>, alt!(
        map!(tag!([0x00u8]), |_| false) |
        map!(tag!([0x01u8]), |_| true)
    ));

    fn decode_false_fixed0(bytes: &[u8]) -> IResult<&[u8], bool, u32> {
        IResult::Done(bytes, false)
    }

    fn decode_true_fixed0(bytes: &[u8]) -> IResult<&[u8], bool, u32> {
        IResult::Done(bytes, true)
    }

    fn encode_fixed0(_: &bool, _: &mut BytesMut) { }

    fn encode_fixed1(b: &bool, bytes: &mut BytesMut) {
        if *b { bytes.put_u8(0x01u8) } else { bytes.put_u8(0x00u8) }
    }
}

impl Decode for Constructor<bool> {
    named!(decode<Constructor<bool>>, alt!(
        map!(tag!([0x56u8]), |_|
            Constructor {
                format_code: 0x56u8,
                size: size1::<bool>,
                decode: Constructor::<bool>::decode_fixed1,
                encode: Constructor::<bool>::encode_fixed1,
            }
        ) |
        map!(tag!([0x41u8]), |_|
            Constructor {
                format_code: 0x41u8,
                size: size0::<bool>,
                decode: Constructor::<bool>::decode_true_fixed0,
                encode: Constructor::<bool>::encode_fixed0,
            }
        ) |
        map!(tag!([0x42u8]), |_|
            Constructor {
                format_code: 0x42u8,
                size: size0::<bool>,
                decode: Constructor::<bool>::decode_false_fixed0,
                encode: Constructor::<bool>::encode_fixed0,
            }
        )
    ));
}

impl Constructor<u8> {
    fn encode_fixed(b: &u8, bytes: &mut BytesMut) {
        bytes.put_u8(*b)
    }
}

impl Decode for Constructor<u8> {
    named!(decode<Constructor<u8>>, map!(tag!([0x50u8]), |_| 
        Constructor {
            format_code: 0x50u8,
            size: size1::<u8>,
            decode: be_u8,
            encode: Constructor::<u8>::encode_fixed,
        }
    ));
}

impl Constructor<u16> {
    fn encode_fixed(b: &u16, bytes: &mut BytesMut) {
        bytes.put_u16::<BigEndian>(*b)
    }
}

impl Decode for Constructor<u16> {
    named!(decode<Constructor<u16>>, map!(tag!([0x60u8]), |_|
        Constructor {
            format_code: 0x60u8,
            size: size2::<u16>,
            decode: be_u16,
            encode: Constructor::<u16>::encode_fixed,
        }
    ));
}

// u32
impl Constructor<u32> {
    named!(decode_small<u32>, map!(be_u8, |i| i as u32));

    fn decode_zero(bytes: &[u8]) -> IResult<&[u8], u32, u32> {
        IResult::Done(bytes, 0)
    }

    fn encode_zero(_: &u32, _: &mut BytesMut) { }

    fn encode_small(b: &u32, bytes: &mut BytesMut) {
        bytes.put_u8(*b as u8)
    }

    fn encode_large(b: &u32, bytes: &mut BytesMut) {
        bytes.put_u32::<BigEndian>(*b)
    }
}

impl Decode for Constructor<u32> {
    named!(decode<Constructor<u32>>, alt!(
        map!(tag!([0x70u8]), |_|
            Constructor {
                format_code: 0x70u8,
                size: size4::<u32>,
                decode: be_u32,
                encode: Constructor::<u32>::encode_large,
            }
        ) |
        map!(tag!([0x52u8]), |_|
            Constructor {
                format_code: 0x52u8,
                size: size1::<u32>,
                decode: Constructor::<u32>::decode_small,
                encode: Constructor::<u32>::encode_small,
            }
        ) |
        map!(tag!([0x43u8]), |_|
            Constructor {
                format_code: 0x43u8,
                size: size0::<u32>,
                decode: Constructor::<u32>::decode_zero,
                encode: Constructor::<u32>::encode_zero,
            }
        )
    ));
}

// u64
impl Constructor<u64> {
    named!(decode_small<u64>, map!(be_u8, |i| i as u64));

    fn decode_zero(bytes: &[u8]) -> IResult<&[u8], u64, u32> {
        IResult::Done(bytes, 0)
    }

    fn encode_zero(_: &u64, _: &mut BytesMut) { }

    fn encode_small(b: &u64, bytes: &mut BytesMut) {
        bytes.put_u8(*b as u8)
    }

    fn encode_large(b: &u64, bytes: &mut BytesMut) {
        bytes.put_u64::<BigEndian>(*b)
    }
}

impl Decode for Constructor<u64> {
    named!(decode<Constructor<u64>>, alt!(
        map!(tag!([0x80u8]), |_|
            Constructor {
                format_code: 0x80u8,
                size: size8::<u64>,
                decode: be_u64,
                encode: Constructor::<u64>::encode_large,
            }
        ) |
        map!(tag!([0x53u8]), |_|
            Constructor {
                format_code: 0x53u8,
                size: size1::<u64>,
                decode: Constructor::<u64>::decode_small,
                encode: Constructor::<u64>::encode_small,
            }
        ) |
        map!(tag!([0x44u8]), |_|
            Constructor {
                format_code: 0x44u8,
                size: size0::<u64>,
                decode: Constructor::<u64>::decode_zero,
                encode: Constructor::<u64>::encode_zero,
            })
    ));
}

impl Constructor<i8> {
    fn encode_fixed(b: &i8, bytes: &mut BytesMut) {
        bytes.put_i8(*b)
    }
}

impl Decode for Constructor<i8> {
    named!(decode<Constructor<i8>>, map!(tag!([0x51u8]), |_| 
        Constructor {
            format_code: 0x51u8,
            size: size1::<i8>,
            decode: be_i8,
            encode: Constructor::<i8>::encode_fixed,
        }
    ));
}

impl Constructor<i16> {
    fn encode_fixed(b: &i16, bytes: &mut BytesMut) {
        bytes.put_i16::<BigEndian>(*b)
    }
}

impl Decode for Constructor<i16> {
    named!(decode<Constructor<i16>>, map!(tag!([0x61u8]), |_|
        Constructor {
            format_code: 0x61u8,
            size: size2::<i16>,
            decode: be_i16,
            encode: Constructor::<i16>::encode_fixed,
        }
    ));
}

// i32
impl Constructor<i32> {
    named!(decode_small<i32>, map!(be_i8, |i| i as i32));

    fn encode_small(b: &i32, bytes: &mut BytesMut) {
        bytes.put_i8(*b as i8);
    }

    fn encode_large(b: &i32, bytes: &mut BytesMut) {
        bytes.put_i32::<BigEndian>(*b);
    }
}

impl Decode for Constructor<i32> {
    named!(decode<Constructor<i32>>, alt!(
        map!(tag!([0x71u8]), |_|
            Constructor {
                format_code: 0x71u8,
                size: size4::<i32>,
                decode: be_i32,
                encode: Constructor::<i32>::encode_large,
            }
        ) |
        map!(tag!([0x54u8]), |_|
            Constructor {
                format_code: 0x54u8,
                size: size1::<i32>,
                decode: Constructor::<i32>::decode_small,
                encode: Constructor::<i32>::encode_small,
            }
        )
    ));
}

// i64
impl Constructor<i64> {
    named!(decode_small<i64>, map!(be_i8, |i| i as i64));

    fn encode_small(b: &i64, bytes: &mut BytesMut) {
        bytes.put_i8(*b as i8);
    }

    fn encode_large(b: &i64, bytes: &mut BytesMut) {
        bytes.put_i64::<BigEndian>(*b);
    }
}

impl Decode for Constructor<i64> {
    named!(decode<Constructor<i64>>, alt!(
        map!(tag!([0x81u8]), |_|
            Constructor {
                format_code: 0x81u8,
                size: size8::<i64>,
                decode: be_i64,
                encode: Constructor::<i64>::encode_large,
            }
        ) |
        map!(tag!([0x55u8]), |_|
            Constructor {
                format_code: 0x55u8,
                size: size1::<i64>,
                decode: Constructor::<i64>::decode_small,
                encode: Constructor::<i64>::encode_small,
            }
        )
    ));
}

impl Constructor<f32> {
    fn encode_float(b: &f32, bytes: &mut BytesMut) {
        bytes.put_f32::<BigEndian>(*b);
    }
}

impl Decode for Constructor<f32> {
    named!(decode<Constructor<f32>>, map!(tag!([0x72u8]), |_| 
        Constructor {
            format_code: 0x72,
            size: size4::<f32>,
            decode: be_f32,
            encode: Constructor::<f32>::encode_float,
        }
    ));
}

impl Constructor<f64> {
    fn encode_double(b: &f64, bytes: &mut BytesMut) {
        bytes.put_f64::<BigEndian>(*b);
    }
}

impl Decode for Constructor<f64> {
    named!(decode<Constructor<f64>>, map!(tag!([0x82u8]), |_|
        Constructor {
            format_code: 0x82,
            size: size8::<f64>,
            decode: be_f64,
            encode: Constructor::<f64>::encode_double,
        }
    ));
}

// char
impl Constructor<char> {
    named!(decode_u32<char>, map_opt!(be_u32, char::from_u32));

    fn encode_u32(b: &char, bytes: &mut BytesMut) {
        bytes.put_u32::<BigEndian>(*b as u32)
    }
}

impl Decode for Constructor<char> {
    named!(decode<Constructor<char>>, map!(tag!([0x73u8]), |_|
        Constructor {
            format_code: 0x73,
            size: size4::<char>,
            decode: Constructor::<char>::decode_u32,
            encode: Constructor::<char>::encode_u32,
        }
    ));
}

// DateTime<Utc>
impl Constructor<DateTime<Utc>> {
    named!(decode_millis<DateTime<Utc>>, map!(be_i64, datetime_from_millis));

    fn encode_millis(b: &DateTime<Utc>, bytes: &mut BytesMut) {
        let timestamp = b.timestamp() * 1000 + (b.timestamp_subsec_millis() as i64);
        bytes.put_i64::<BigEndian>(timestamp);
    }
}

impl Decode for Constructor<DateTime<Utc>> {
    named!(decode<Constructor<DateTime<Utc>>>, map!(tag!([0x83u8]), |_|
        Constructor {
            format_code: 0x83,
            size: size8::<DateTime<Utc>>,
            decode: Constructor::<DateTime<Utc>>::decode_millis,
            encode: Constructor::<DateTime<Utc>>::encode_millis,
        }
    ));
}

impl Constructor<Uuid> {
    named!(decode_bytes<Uuid>, map_res!(take!(16), Uuid::from_bytes));

    fn encode_bytes(b: &Uuid, bytes: &mut BytesMut) {
        bytes.put_slice(b.as_bytes());
    }
}

impl Decode for Constructor<Uuid> {
    named!(decode<Constructor<Uuid>>, map!(tag!([0x98u8]), |_|
        Constructor {
            format_code: 0x98,
            size: size16::<Uuid>,
            decode: Constructor::<Uuid>::decode_bytes,
            encode: Constructor::<Uuid>::encode_bytes,
        }
    ));
}

// Bytes
impl Constructor<Bytes> {
    named!(decode_short<Bytes>, do_parse!(bytes: length_bytes!(be_u8) >> (Bytes::from(bytes))));
    named!(decode_long<Bytes>, do_parse!(bytes: length_bytes!(be_u32) >> (Bytes::from(bytes))));

    fn encode_short(b: &Bytes, bytes: &mut BytesMut) {
        bytes.put_u8(b.len() as u8);
        bytes.put(b);
    }

    fn encode_long(b: &Bytes, bytes: &mut BytesMut) {
        bytes.put_u32::<BigEndian>(b.len() as u32);
        bytes.put(b);
    }

    fn size_short(b: &Bytes) -> usize {
        1 + b.len()
    }

    fn size_long(b: &Bytes) -> usize {
        4 + b.len()
    }
}

impl Decode for Constructor<Bytes> {
    named!(decode<Constructor<Bytes>>, alt!(
        map!(tag!([0xA0u8]), |_|
            Constructor {
                format_code: 0xA0,
                size: Constructor::<Bytes>::size_short,
                decode: Constructor::<Bytes>::decode_short,
                encode: Constructor::<Bytes>::encode_short,
            }
        ) |
        map!(tag!([0xB0u8]), |_|
            Constructor {
                format_code: 0xB0,
                size: Constructor::<Bytes>::size_long,
                decode: Constructor::<Bytes>::decode_long,
                encode: Constructor::<Bytes>::encode_long,
            }
        )
    ));
}

// ByteStr
impl Constructor<ByteStr> {
    named!(decode_short<ByteStr>, do_parse!(string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (ByteStr::from(string))));
    named!(decode_long<ByteStr>, do_parse!(string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (ByteStr::from(string))));

    fn encode_short(b: &ByteStr, bytes: &mut BytesMut) {
        bytes.put_u8(b.len() as u8);
        bytes.put(b.as_bytes());
    }

    fn encode_long(b: &ByteStr, bytes: &mut BytesMut) {
        bytes.put_u32::<BigEndian>(b.len() as u32);
        bytes.put(b.as_bytes());
    }

    fn size_short(b: &ByteStr) -> usize {
        1 + b.len()
    }

    fn size_long(b: &ByteStr) -> usize {
        4 + b.len()
    }
}

impl Decode for Constructor<ByteStr> {
    named!(decode<Constructor<ByteStr>>, alt!(
        map!(tag!([0xA1u8]), |_|
            Constructor {
                format_code: 0xA1,
                size: Constructor::<ByteStr>::size_short,
                decode: Constructor::<ByteStr>::decode_short,
                encode: Constructor::<ByteStr>::encode_short,
            }
        ) |
        map!(tag!([0xB1u8]), |_|
            Constructor {
                format_code: 0xB1,
                size: Constructor::<ByteStr>::size_long,
                decode: Constructor::<ByteStr>::decode_long,
                encode: Constructor::<ByteStr>::encode_long,
            }
        )
    ));
}

// Symbol
impl Constructor<Symbol> {
    named!(decode_short<Symbol>, do_parse!(string: map_res!(length_bytes!(be_u8), str::from_utf8) >> (Symbol::from(string))));
    named!(decode_long<Symbol>, do_parse!(string: map_res!(length_bytes!(be_u32), str::from_utf8) >> (Symbol::from(string))));

    fn encode_short(b: &Symbol, bytes: &mut BytesMut) {
        bytes.put_u8(b.len() as u8);
        bytes.put(b.as_bytes());
    }

    fn encode_long(b: &Symbol, bytes: &mut BytesMut) {
        bytes.put_u32::<BigEndian>(b.len() as u32);
        bytes.put(b.as_bytes());
    }

    fn size_short(b: &Symbol) -> usize {
        1 + b.len()
    }

    fn size_long(b: &Symbol) -> usize {
        4 + b.len()
    }
}

impl Decode for Constructor<Symbol> {
    named!(decode<Constructor<Symbol>>, alt!(
        map!(tag!([0xA3u8]), |_|
            Constructor {
                format_code: 0xA3,
                size: Constructor::<Symbol>::size_short,
                decode: Constructor::<Symbol>::decode_short,
                encode: Constructor::<Symbol>::encode_short,
            }
        ) |
        map!(tag!([0xB3u8]), |_|
            Constructor {
                format_code: 0xB3,
                size: Constructor::<Symbol>::size_long,
                decode: Constructor::<Symbol>::decode_long,
                encode: Constructor::<Symbol>::encode_long,
            }
        )
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
