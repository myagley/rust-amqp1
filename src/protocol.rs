use bytes::{Buf, BufMut, BytesMut, Bytes, BigEndian};
use super::errors::*;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use uuid::Uuid;
use nom::{ErrorKind};
use super::codec::{self, Decode, DecodeFormatted, Encode, decode_map_header, decode_list_header, INVALID_FORMATCODE};
use super::types::*;

pub(crate) struct CompoundHeader {
    pub size: u32,
    pub count: u32,
}

impl  CompoundHeader {
    pub fn empty() -> CompoundHeader {
         CompoundHeader { size: 0, count: 0 }
    }
}

pub const PROTOCOL_HEADER_LEN: usize = 8;
const PROTOCOL_HEADER_PREFIX: &'static [u8] = b"AMQP";
const PROTOCOL_VERSION: &'static [u8] = &[1, 0, 0];

pub enum ProtocolId {
    Amqp = 0,
    AmqpTls = 1,
    AmqpSasl = 2
}

pub fn decode_protocol_header(src: &[u8]) -> Result<ProtocolId> {
    if &src[0..3] != PROTOCOL_HEADER_PREFIX {
        return Err("Protocol header is invalid.".into());
    }
    let protocol_id = src[4];
    if &src[5..7] != PROTOCOL_VERSION {
        return Err("Protocol version is incompatible.".into());
    }
    match protocol_id {
        0 => Ok(ProtocolId::Amqp),
        1 => Ok(ProtocolId::AmqpTls),
        2 => Ok(ProtocolId::AmqpSasl),
        _ => Err("Unknown protocol id.".into())
    }
}

pub trait Described {
    fn descriptor_name(&self) -> &str;
    fn descriptor_domain(&self) -> u32;
    fn descriptor_code(&self) -> u32;
}

pub type Map = HashMap<Variant, Variant>;
pub type Fields = HashMap<Symbol, Variant>;
pub type FilterSet = HashMap<Symbol, Option<ByteStr>>;
pub type Timestamp = DateTime<Utc>;
pub type Symbols = Vec<Symbol>;
pub type IetfLanguageTags = Vec<IetfLanguageTag>;

include!(concat!(env!("OUT_DIR"), "/definitions.rs"));

#[derive(Debug, Hash, Eq, PartialEq, Clone)]
pub enum AnnotationKey {
    Ulong(u64),
    Symbol(Symbol)
}

pub type Annotations = HashMap<Symbol, Variant>;

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum MessageId {
    Ulong(u64),
    Uuid(Uuid),
    Binary(Bytes),
    String(ByteStr)
}

impl DecodeFormatted for MessageId {
    fn decode_with_format(input: &[u8], format: u8) -> Result<(&[u8], Self)> {
        match format {
            codec::FORMATCODE_SMALLULONG |
            codec::FORMATCODE_ULONG |
            codec::FORMATCODE_ULONG_0 => u64::decode_with_format(input, format).map(|(i, o)| (i, MessageId::Ulong(o))),
            codec::FORMATCODE_UUID => Uuid::decode_with_format(input, format).map(|(i, o)| (i, MessageId::Uuid(o))),
            codec::FORMATCODE_BINARY8 |
            codec::FORMATCODE_BINARY32 => Bytes::decode_with_format(input, format).map(|(i, o)| (i, MessageId::Binary(o))),
            codec::FORMATCODE_STRING8 |
            codec::FORMATCODE_STRING32 => ByteStr::decode_with_format(input, format).map(|(i, o)| (i, MessageId::String(o))),
            _ => Err(ErrorKind::Custom(codec::INVALID_FORMATCODE).into())
        }
    }
}

impl Encode for MessageId {
    fn encoded_size(&self) -> usize {
        match *self {
            MessageId::Ulong(v) => v.encoded_size(),
            MessageId::Uuid(v) => v.encoded_size(),
            MessageId::Binary(v) => v.encoded_size(),
            MessageId::String(v) => v.encoded_size()
        }
    }
    fn encode(&self, buf: &mut BytesMut) {
        match *self {
            MessageId::Ulong(v) => v.encode(buf),
            MessageId::Uuid(v) => v.encode(buf),
            MessageId::Binary(v) => v.encode(buf),
            MessageId::String(v) => v.encode(buf)
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorCondition {
    AmqpError(AmqpError),
    ConnectionError(ConnectionError),
    SessionError(SessionError),
    LinkError(LinkError),
    Custom(Symbol)
}

impl DecodeFormatted for ErrorCondition {
    fn decode_with_format(input: &[u8], format: u8) -> Result<(&[u8], Self)> {
        let (input, result) = Symbol::decode_with_format(input, format)?;
        if let Ok(r) = AmqpError::try_from(&result) {
            return Ok((input, ErrorCondition::AmqpError(r)));
        }
        if let Ok(r) = ConnectionError::try_from(&result) {
            return Ok((input, ErrorCondition::ConnectionError(r)));
        }
        if let Ok(r) = SessionError::try_from(&result) {
            return Ok((input, ErrorCondition::SessionError(r)));
        }
        if let Ok(r) = LinkError::try_from(&result) {
            return Ok((input, ErrorCondition::LinkError(r)));
        }
        Ok((input, ErrorCondition::Custom(result)))
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum DistributionMode {
    Move,
    Copy,
    Custom(Symbol)
}

impl DecodeFormatted for DistributionMode {
    fn decode_with_format(input: &[u8], format: u8) -> Result<(&[u8], Self)> {
        let (input, result) = Symbol::decode_with_format(input, format)?;
        let result = match result.as_str() {
            "move" => DistributionMode::Move,
            "copy" => DistributionMode::Copy,
            _ => DistributionMode::Custom(result)
        };
        Ok((input, result))
    }
}

