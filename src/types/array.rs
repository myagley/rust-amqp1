use std::iter::ExactSizeIterator;
use std::marker::PhantomData;

use bytes::Bytes;
use codec::Decode;
use nom::IResult;

pub struct Array<T>
    where T: Decode
{
    count: usize,
    bytes: Bytes,
    phantom: PhantomData<T>,
}

impl<T: Decode> Array<T> {
    pub fn new(count: usize, bytes: Bytes) -> Array<T> {
        Array {
            count,
            bytes,
            phantom: PhantomData,
        }
    }
}

pub struct IntoIter<T>
    where T: Decode
{
    count: usize,
    bytes: Bytes,
    phantom: PhantomData<T>,
}

impl<'a, T: Decode> Iterator for IntoIter<T> {
    type Item = T;

    #[inline]
    fn next(&mut self) -> Option<T> {
        if self.bytes.is_empty() {
            None
        } else {
            let (remaining, result) = match T::decode(&self.bytes[..]) {
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
            phantom: PhantomData,
        }
    }
}
