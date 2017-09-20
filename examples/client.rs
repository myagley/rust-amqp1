#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate amqp1 as amqp;
extern crate bytes;
extern crate futures_await as futures;
extern crate hex_slice;
extern crate tokio_core;
extern crate tokio_io;
extern crate uuid;

use futures::prelude::*;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::io::{read_exact, write_all};
use std::net::SocketAddr;
use futures::{Future, Sink, Stream};
use amqp::{Error, Result, ResultExt};
use amqp::types::{Symbol, ByteStr};
use amqp::io::{AmqpDecoder, AmqpEncoder};
use amqp::protocol::{self, decode_protocol_header, encode_protocol_header, ProtocolId};
use amqp::framing::{AmqpFrame, SaslFrame};
use amqp::protocol::{Frame, SaslCode, SaslFrameBody, SaslInit, SaslOutcome};
use amqp::codec::Encode;
use bytes::{Bytes, BytesMut};
use hex_slice::AsHex;
use uuid::Uuid;


fn main() {
    let mut core = Core::new().unwrap();
    let handle = core.handle();
    let client = send(handle);
    core.run(client).unwrap();
}

#[async]
fn send(handle: Handle) -> Result<()> {
    let addr = "127.0.0.1:5769".parse().unwrap();
    let tcp = await!(TcpStream::connect(&addr, &handle))?;
    let (mut reader, mut writer) = tcp.split();

    // negotiating SASL exchange
    let (reader, writer) = await!(negotiate_protocol(ProtocolId::AmqpSasl, reader, writer))?;

    let sasl_reader = tokio_io::codec::FramedRead::new(reader, AmqpDecoder::<SaslFrame>::new());

    // processing sasl-mechanisms
    let (sasl_frame, sasl_reader) = await!(sasl_reader.into_future()).map_err(|e| e.0)?;
    let plain_symbol = Symbol::from_static("PLAIN");
    if let Some(SaslFrame {
        body: SaslFrameBody::SaslMechanisms(mechs),
    }) = sasl_frame
    {
        if !mechs
            .sasl_server_mechanisms()
            .0
            .iter()
            .any(|m| *m == plain_symbol)
        {
            return Err(
                format!(
                    "only PLAIN SASL mechanism is supported. server supports: {:?}",
                    mechs.sasl_server_mechanisms()
                ).into(),
            );
        }
    } else {
        return Err(
            format!(
                "expected SASL Mechanisms frame to arrive, seen `{:?}` instead.",
                sasl_frame
            ).into(),
        );
    }
    // sending sasl-init
    let sasl_writer = tokio_io::codec::FramedWrite::new(writer, AmqpEncoder::<SaslFrame>::new());
    let initial_response = SaslInit::prepare_response("", "duggie", "pow wow");
    let sasl_init = SaslInit {
        mechanism: plain_symbol,
        initial_response: Some(initial_response),
        hostname: None,
    };
    let sasl_writer = await!(sasl_writer.send(SaslFrame::new(SaslFrameBody::SaslInit(sasl_init))))?;
    // processing sasl-outcome
    let (sasl_frame, sasl_reader) = await!(sasl_reader.into_future()).map_err(|e| e.0)?;
    if let Some(SaslFrame {
        body: SaslFrameBody::SaslOutcome(outcome),
    }) = sasl_frame
    {
        if outcome.code() != SaslCode::Ok {
            return Err(
                format!(
                    "SASL auth did not result in Ok outcome, seen `{:?}` instead. More info: {:02x}",
                    outcome.code(),
                    outcome
                        .additional_data()
                        .unwrap_or(&Bytes::new())
                        .as_ref()
                        .as_hex()
                ).into(),
            );
        }
    } else {
        return Err(
            format!(
                "expected SASL Outcome frame to arrive, seen `{:?}` instead.",
                sasl_frame
            ).into(),
        );
    }

    let writer = sasl_writer.into_inner();
    let reader = sasl_reader.into_inner();
    let (reader, writer) = await!(negotiate_protocol(ProtocolId::Amqp, reader, writer))?;

    let reader = tokio_io::codec::FramedRead::new(reader, AmqpDecoder::<AmqpFrame>::new());
    let writer = tokio_io::codec::FramedWrite::new(writer, AmqpEncoder::<AmqpFrame>::new());

    let (reader, writer) = await!(open_connection(reader, writer))?;
    let (channel, reader, writer) = await!(begin_session(reader, writer))?;
    let (handle, reader, writer) = await!(attach_link(channel, reader, writer))?;
    let (delivery_tag, writer) = await!(transfer(writer, channel, handle, Bytes::from(vec![1,2,3,4,5,6])))?;
    println!("sent transfer: {:2x}", delivery_tag.as_ref().as_hex());
    let (frame_opt, reader) = await!(reader.into_future()).map_err(|e| e.0)?;
    println!("last seen: {:?}", frame_opt); // wait for flow
    let (frame_opt, reader) = await!(reader.into_future()).map_err(|e| e.0)?;
    println!("last seen #2: {:?}", frame_opt); // wait for flow

    Ok(())
}

#[async]
fn negotiate_protocol<TR: AsyncRead + 'static, TW: AsyncWrite + 'static>(protocol_id: ProtocolId, reader: TR, writer: TW) -> Result<(TR, TW)> {
    let header_buf = encode_protocol_header(protocol_id);
    let (writer, _) = await!(write_all(writer, header_buf))?;
    let mut header_buf = [0; 8];
    let (reader, header_buf) = await!(read_exact(reader, header_buf))?;
    let recv_protocol_id = decode_protocol_header(&header_buf)?; // todo: surface for higher level to be able to respond properly / validate
    if recv_protocol_id != protocol_id {
        return Err(
            format!(
                "Expected `{:?}` protocol id, seen `{:?} instead.`",
                protocol_id,
                recv_protocol_id
            ).into(),
        );
    }
    Ok((reader, writer))
}

#[async]
fn open_connection<FR, FW>(reader: FR, writer: FW) -> Result<(FR, FW)>
    where FR: Stream<Item = AmqpFrame, Error = Error> + 'static,
        FW: Sink<SinkItem = AmqpFrame, SinkError = Error> + 'static
{
    let open = protocol::Open {
        container_id: "container-id".into(),
        hostname: None,
        max_frame_size: ::std::u16::MAX as u32,
        channel_max: ::std::u16::MAX,
        idle_time_out: None,
        outgoing_locales: None,
        incoming_locales: None,
        offered_capabilities: None,
        desired_capabilities: None,
        properties: None,
    };
    let writer = await!(writer.send(AmqpFrame::new(0, Frame::Open(open), Bytes::new())))?;
    let (frame_opt, reader) = await!(reader.into_future()).map_err(|e| e.0)?;

    if let Some(frame) = frame_opt {
        println!("{:?}", frame);
        if let Frame::Open(ref open) = *frame.performative() {
            Ok((reader, writer))
        }
        else {
            Err(format!("Expected Open performative to arrive, seen `{:?}` instead.", frame).into())
        }
    }
    else {
        Err("Connection is closed.".into())        
    }
}

#[async]
fn begin_session<FR, FW>(reader: FR, writer: FW) -> Result<(u16, FR, FW)>
    where FR: Stream<Item = AmqpFrame, Error = Error> + 'static,
        FW: Sink<SinkItem = AmqpFrame, SinkError = Error> + 'static
{
    let begin = protocol::Begin {
        remote_channel: None,
        next_outgoing_id: 1,
        incoming_window: 5000,
        outgoing_window: 5000,
        handle_max: ::std::u32::MAX,
        offered_capabilities: None,
        desired_capabilities: None,
        properties: None,
    };
    let writer = await!(writer.send(AmqpFrame::new(0, Frame::Begin(begin), Bytes::new())))?;
    let (frame_opt, reader) = await!(reader.into_future()).map_err(|e| e.0)?;

    if let Some(frame) = frame_opt {
        println!("{:?}", frame);
        if let Frame::Begin(ref begin) = *frame.performative() {
            if let Some(ch) = begin.remote_channel() {
                Ok((ch, reader, writer))
            }
            else {
                Err("Received Begin has no remote channel assigned.".into())
            }
        }
        else {
            Err(format!("Expected Begin performative to arrive, seen `{:?}` instead.", frame).into())
        }
    }
    else {
        Err("Connection is closed.".into())
    }
}

#[async]
fn attach_link<FR, FW>(channel: u16, reader: FR, writer: FW) -> Result<(u32, FR, FW)>
    where FR: Stream<Item = AmqpFrame, Error = Error> + 'static,
        FW: Sink<SinkItem = AmqpFrame, SinkError = Error> + 'static
{
    let target = protocol::Target {
        address: Some(ByteStr::from_static("par-pref/control/7dddd1cd-64b8-4388-b5d0-cd2efb577596")),
        durable: protocol::TerminusDurability::None,
        expiry_policy: protocol::TerminusExpiryPolicy::SessionEnd,
        timeout: 0,
        dynamic: false,
        dynamic_node_properties: None,
        capabilities: None
    };
    let attach = protocol::Attach {
        name: ByteStr::from_static("CtlReq_"),
        handle: 0,
        role: protocol::Role::Sender,
        snd_settle_mode: protocol::SenderSettleMode::Settled,
        rcv_settle_mode: protocol::ReceiverSettleMode::First,
        source: None,
        target: Some(target),
        unsettled: None,
        incomplete_unsettled: false,
        initial_delivery_count: None,
        max_message_size: None,
        offered_capabilities: None,
        desired_capabilities: None,
        properties: None,
    };
    let writer = await!(writer.send(AmqpFrame::new(channel, Frame::Attach(attach), Bytes::new())))?;
    let (frame_opt, reader) = await!(reader.into_future()).map_err(|e| e.0)?;

    if let Some(frame) = frame_opt {
        println!("{:?}", frame);
        if let Frame::Attach(ref attach) = *frame.performative() {
            Ok((attach.handle(), reader, writer))
        }
        else {
            Err(format!("Expected Attach performative to arrive, seen `{:?}` instead.", frame).into())
        }
    }
    else {
        Err("Connection is closed.".into())
    }
}

#[async]
fn transfer<FW>(writer: FW, channel: u16, handle: protocol::Handle, payload: Bytes) -> Result<(Bytes, FW)>
    where FW: Sink<SinkItem = AmqpFrame, SinkError = Error> + 'static
{
    let delivery_tag = Bytes::from(&Uuid::new_v4().as_bytes()[..]);
    let transfer = protocol::Transfer {
        handle: handle,
        delivery_id: Some(0),
        delivery_tag: Some(delivery_tag.clone()),
        message_format: None,
        settled: Some(true),
        more: false,
        rcv_settle_mode: None,
        state: None,
        resume: false,
        aborted: false,
        batchable: false
    };
    let mut payload_section = BytesMut::with_capacity(payload.encoded_size());
    payload.encode(&mut payload_section);
    let writer = await!(writer.send(AmqpFrame::new(channel, Frame::Transfer(transfer), payload_section.freeze())))?;
    Ok((delivery_tag, writer))




    // let (frame_opt, reader) = await!(reader.into_future()).map_err(|e| e.0)?;
    // if let Some(frame) = frame_opt {
    //     println!("{:?}", frame);
    //     if let Frame::Attach(ref attach) = *frame.performative() {
    //         Ok((attach.handle(), reader, writer))
    //     }
    //     else {
    //         Err(format!("Expected Attach performative to arrive, seen `{:?}` instead.", frame).into())
    //     }
    // }
    // else {
    //     Err("Connection is closed.".into())
    // }
}