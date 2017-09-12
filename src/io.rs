use tokio_io::codec::{Decoder, Encoder};
use bytes::{BytesMut, ByteOrder, BigEndian};
use super::errors::{Result, Error};
use super::protocol::{ProtocolId, decode_protocol_header, PROTOCOL_HEADER_LEN};
use super::framing::{Frame, HEADER_LEN};
use codec::Decode;

pub struct Codec {
    state: DecodeState,
    protocol_id: Option<ProtocolId>
}

#[derive(Debug, Clone, Copy)]
enum DecodeState {
    ProtocolHeader,
    FrameHeader,
    Frame(usize),
}

impl Codec {
    fn decode_protocol_header(&self, src: &mut BytesMut) -> Result<Option<(ProtocolId)>> {
        if src.len() < PROTOCOL_HEADER_LEN {
            src.reserve(PROTOCOL_HEADER_LEN);
            return Ok(None);
        }
        
        // todo: validate / respond
        let header = src.split_to(PROTOCOL_HEADER_LEN);
        let protocol_id = decode_protocol_header(header.as_ref())?; // todo: surface for higher level to be able to respond properly / validate
        Ok(Some(protocol_id))
    }
}

impl Decoder for Codec {
    type Item = Frame;
    type Error = Error;

    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>> {
        loop {
            match self.state {
                DecodeState::ProtocolHeader => {
                    match self.decode_protocol_header(src)? {
                        Some(protocol_id) => {
                            self.protocol_id = Some(protocol_id);
                            self.state = DecodeState::FrameHeader;
                            continue;
                        }
                        None => {
                            return Ok(None);
                        }
                    }
                },
                DecodeState::FrameHeader => {
                    let len = src.len();
                    if len < HEADER_LEN {
                        return Ok(None);
                    }
                    let size = BigEndian::read_u32(src.as_ref()) as usize;
                    // todo: max frame size check
                    self.state = DecodeState::Frame(size);
                    if len < size {
                        src.reserve(size); // extend receiving buffer to fit the whole frame -- todo: too eager?
                        return Ok(None);
                    }
                },
                DecodeState::Frame(size) => {
                    if src.len() < size {
                        return Ok(None);
                    }

                    let frame_buf = src.split_to(size);
                    let (remainder, frame) = Frame::decode(frame_buf.as_ref())?;
                    if remainder.len() > 0 { // todo: could it really happen?
                        return Err("bytes left unparsed at the frame trail".into());
                    }
                    src.reserve(HEADER_LEN);
                    return Ok(Some(frame));
                }
            }
        }
    }
}

impl Encoder for Codec {
    type Item = Frame;
    type Error = Error;

    fn encode(&mut self, item: Self::Item, dst: &mut BytesMut) -> Result<()> {
        // let content_size = calc_remaining_length(&item);
        // dst.reserve(content_size + 5);
        // dst.writer().write_packet(&item);
        unimplemented!()
    }
}
