use std::marker::Sized;

use bytes::BytesMut;
use nom::IResult;

mod decode;
mod encode;

pub trait Encode {
    fn encoded_size(&self) -> usize;
    fn encode(&self, buf: &mut BytesMut) -> ();
}

pub trait Decode
    where Self: Sized
{
    fn constructor(bytes: &[u8]) -> IResult<&[u8], Constructor<Self>, u32>;

    fn decode(bytes: &[u8]) -> IResult<&[u8], Self, u32>;
}

pub struct Constructor<T: Decode> {
    decode: fn(&[u8]) -> IResult<&[u8], T, u32>,
}

impl<T: Decode> Clone for Constructor<T> {
    fn clone(&self) -> Self {
        Constructor {
            decode: self.decode
        }
    }
}

impl<T: Decode> Constructor<T> {
    pub fn construct<'a>(&self, bytes: &'a[u8]) -> IResult<&'a[u8], T, u32> {
        (self.decode)(bytes)
    }
}