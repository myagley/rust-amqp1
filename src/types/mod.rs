mod null;
mod str;
mod symbol;
mod variant;

pub use self::null::Null;
pub use self::str::ByteStr;
pub use self::symbol::Symbol;
pub use self::variant::Variant;

use chrono::{DateTime, Utc};
use std::collections::HashMap;
use bytes::{BufMut, Bytes, BytesMut};
use uuid::Uuid;
use super::codec::{Decode, Encode};
use nom::{IResult, Needed};
use super::errors::*;

pub trait Described {
    fn descriptor_name(&self) -> &str;
    fn descriptor_domain(&self) -> u32;
    fn descriptor_code(&self) -> u32;
}

pub type Map = HashMap<Variant, Variant>;
pub type Fields = HashMap<Symbol, Variant>;
pub type FilterSet = HashMap<Symbol, Option<String>>;
pub type Timestamp = DateTime<Utc>;
pub type Symbols = Vec<Symbol>;
pub type IetfLanguageTags = Vec<IetfLanguageTag>;

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum AnnotationKey {
    Ulong(u64),
    Symbol(Symbol)
}

pub type Annotations = HashMap<Symbol, Variant>;

include!(concat!(env!("OUT_DIR"), "/definitions.rs"));

// macro_rules! decode_size_and_count {
//     ($buf:ident, $code:expr, $code8:expr, $code32:expr) => {
//         match $code {
//             $code8 => {
//                 if buf.len() < 2 {
//                     Err()
//                 }
//                 buf[0]
//             },
//             $code32 => {

//             },
//             _ => 
//         }
//     }
// }

// fn test(b: &[u8]) -> Result<(u32, u32, &[u8]), Error> {
//     let () = decode_size_and_count!(b, 0xc0, 0xc0, 0xd0);
// }

// #[derive(Debug, Eq, PartialEq, Clone)]
// pub enum MessageId {
//     Ulong(u64),
//     Uuid(Uuid),
//     Binary(Bytes),
//     String(ByteStr)
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum DeliveryState {
//     Received(Received),
//     Accepted(Accepted),
//     Rejected(Rejected),
//     Released(Released),
//     Modified(Modified)
// }

// #[derive(Clone, Debug, PartialEq)]
// pub enum Outcome {
//     Accepted(Accepted),
//     Rejected(Rejected),
//     Released(Released),
//     Modified(Modified)
// }
