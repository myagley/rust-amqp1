use bytes::Bytes;
use super::protocol;

/// Length in bytes of the fixed frame header
pub const HEADER_LEN: usize = 8;

/// AMQP Frame type marker (0)
pub const FRAME_TYPE_AMQP: u8 = 0x00;
pub const FRAME_TYPE_SASL: u8 = 0x01;

/// Represents a frame. There are two common variants: AMQP and SASL frames
#[derive(Clone, Debug, PartialEq)]
pub enum Frame {
    Amqp(AmqpFrame),
    Sasl()
}

/// Represents an AMQP Frame
#[derive(Clone, Debug, PartialEq)]
pub struct AmqpFrame {
    channel_id: u16,
    performative: protocol::Frame,
    body: Bytes
}

impl AmqpFrame {
    pub fn new(channel_id: u16, performative: protocol::Frame, body: Bytes) -> AmqpFrame {
        AmqpFrame { channel_id, performative, body }
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
