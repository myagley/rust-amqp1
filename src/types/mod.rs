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
use std::boxed::Box;
use bytes::Bytes;
use uuid::Uuid;
use std::fmt::{self, Debug, Formatter};

pub trait Described {
    fn descriptor_name(&self) -> &str;
    fn descriptor_domain(&self) -> u32;
    fn descriptor_code(&self) -> u32;
}

pub type Map = HashMap<Variant, Variant>;
pub type Fields = HashMap<Symbol, Variant>;
// pub type Symbols = Vec<Symbol>; // type="symbol" multiple="true"
pub type FilterSet = HashMap<Symbol, Option<String>>;
pub type Timestamp = DateTime<Utc>;
pub type Symbols = Vec<Symbol>;
pub type IetfLanguageTags = Vec<IetfLanguageTag>;

include!(concat!(env!("OUT_DIR"), "/definitions.rs"));

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MessageId {
    Ulong(u64),
    Uuid(Uuid),
    Binary(Bytes),
    String(ByteStr)
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeliveryState {
    Received(Received),
    Accepted(Accepted),
    Rejected(Rejected),
    Released(Released),
    Modified(Modified)
}

#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Accepted(Accepted),
    Rejected(Rejected),
    Released(Released),
    Modified(Modified)
}
