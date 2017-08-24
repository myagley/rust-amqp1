use std::iter::ExactSizeIterator;

use bytes::Bytes;
use codec::{Constructor, Decode};
use nom::IResult;

pub struct Array<T>
    where T: Decode
{
    count: usize,
    bytes: Bytes,
    constructor: Constructor<T>,
}

impl<T: Decode> Array<T> {
    pub fn new(constructor: Constructor<T>, count: usize, bytes: Bytes) -> Array<T> {
        Array {
            count,
            bytes,
            constructor,
        }
    }
}

pub struct IntoIter<T>
    where T: Decode
{
    count: usize,
    bytes: Bytes,
    constructor: Constructor<T>,
}

impl<'a, T: Decode> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.bytes.is_empty() {
            None
        } else {
            let (remaining, result) = match self.constructor.decode_value(&self.bytes[..]) {
                IResult::Done(remaining, result) => {
                    let num = remaining.as_ptr() as usize - self.bytes.as_ptr() as usize;
                    let remaining = self.bytes.slice_from(num);

                    (remaining, result.into())
                }
                IResult::Error(_) => return None,
                IResult::Incomplete(_) => return None,
            };
            self.bytes = remaining;
            result
        }
    }
}

impl<T: Decode> ExactSizeIterator for IntoIter<T> {
    fn len(&self) -> usize {
        self.count
    }
}

impl<'a, T> IntoIterator for &'a Array<T>
    where T: Decode
{
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            count: self.count,
            bytes: self.bytes.clone(),
            constructor: self.constructor.clone(),
        }
    }
}
