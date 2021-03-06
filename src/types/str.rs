use std::str;

use bytes::Bytes;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ByteStr(Bytes);

impl ByteStr {
    pub unsafe fn from_utf8_unchecked(slice: Bytes) -> ByteStr {
        ByteStr(slice)
    }

    pub fn from_static(s: &'static str) -> ByteStr {
        ByteStr(Bytes::from_static(s.as_bytes()))
    }

    pub fn slice(&self, from: usize, to: usize) -> ByteStr {
        assert!(self.as_str().is_char_boundary(from));
        assert!(self.as_str().is_char_boundary(to));
        ByteStr(self.0.slice(from, to))
    }

    pub fn slice_to(&self, idx: usize) -> ByteStr {
        assert!(self.as_str().is_char_boundary(idx));
        ByteStr(self.0.slice_to(idx))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_ref()
    }

    pub fn as_str(&self) -> &str {
        unsafe { str::from_utf8_unchecked(self.0.as_ref()) }
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<'a> From<&'a str> for ByteStr {
    fn from(s: &'a str) -> ByteStr {
        ByteStr(Bytes::from(s))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slice() {
        let a = ByteStr::from("hello");
        let b = a.slice(1, 5);
        assert_eq!(ByteStr::from("ello"), b);
    }

    #[test]
    fn as_str() {
        let a = ByteStr::from("hello");
        assert_eq!("hello", a.as_str());
    }
}
