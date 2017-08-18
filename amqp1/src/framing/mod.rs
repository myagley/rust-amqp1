mod open;

use bytes::Bytes;

pub use self::open::{Open, OpenBuilder};

/// Length in bytes of the fixed frame header
pub const HEADER_LEN: usize = 8;

/// AMQP Frame type marker (0)
pub const AMQP_TYPE: u8 = 0;

/// Represents a frame. There are two common variants: AMQP and SASL frames
#[derive(Debug, PartialEq, Eq)]
pub enum Frame {
    Amqp(AmqpFrame),
}

/// Represents an AMQP Frame
#[derive(Debug, PartialEq, Eq)]
pub struct AmqpFrame {
    channel_id: u16,
    body: Bytes,
}

impl AmqpFrame {
    pub fn new(channel_id: u16, body: Bytes) -> AmqpFrame {
        AmqpFrame {
            channel_id,
            body,
        }
    }

    #[inline]
    pub fn channel_id(&self) -> u16 {
        self.channel_id
    }

    #[inline]
    pub fn body(&self) -> &Bytes {
        &self.body
    }
}

/// Represents a performative of an AMQP Frame.
pub enum Performative {
    Open(Open),
}