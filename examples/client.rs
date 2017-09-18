#![feature(proc_macro, conservative_impl_trait, generators)]

extern crate amqp1 as amqp;
extern crate futures_await as futures;
extern crate tokio_io;
extern crate tokio_core;
extern crate bytes;

use futures::prelude::*;
use tokio_core::net::TcpStream;
use tokio_core::reactor::{Core, Handle};
use tokio_io::AsyncRead;
use tokio_io::io::{read_exact, write_all};
use std::net::SocketAddr;
use futures::{Future, Sink, Stream};
use amqp::{Error, Result, ResultExt};
use amqp::types::Symbol;
use amqp::io::{AmqpDecoder, AmqpEncoder};
use amqp::protocol::{ProtocolId, encode_protocol_header, decode_protocol_header};
use amqp::framing::{AmqpFrame, SaslFrame};
use amqp::protocol::{Frame, SaslFrameBody, SaslInit};
use bytes::BytesMut;

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
    let header_buf = encode_protocol_header(ProtocolId::AmqpSasl);
    let (writer, _) = await!(write_all(writer, header_buf))?;
    let mut header_buf = [0; 8];
    let (reader, header_buf) = await!(read_exact(reader, header_buf))?;
    let protocol_id = decode_protocol_header(&header_buf)?; // todo: surface for higher level to be able to respond properly / validate
    if protocol_id != ProtocolId::AmqpSasl {
        return Err(format!("expected SASL protocol id, seen `{:?} instead.`", protocol_id).into());
    }
    let sasl_reader = tokio_io::codec::FramedRead::new(reader, AmqpDecoder::<SaslFrame>::new());
    let (sasl_frame, sasl_reader) = await!(sasl_reader.into_future()).map_err(|e| e.0)?;

    let plain_symbol = Symbol::from_static("PLAIN");
    if let Some(SaslFrame { body: SaslFrameBody::SaslMechanisms(mechs)}) = sasl_frame {
        if !mechs.sasl_server_mechanisms().0.iter().any(|m| *m == plain_symbol) {
            return Err(format!("only PLAIN SASL mechanism is supported. server supports: {:?}", mechs.sasl_server_mechanisms()).into());
        }
    }
    else {
        return Err(format!("expected SASL mechanisms frame to arrive, seen `{:?}` instead.", sasl_frame).into());
    }
    let sasl_writer = tokio_io::codec::FramedWrite::new(writer, AmqpEncoder::<SaslFrame>::new());
    let initial_response = SaslInit::prepare_response("", "duggie", "pow wow"); 
    let sasl_writer = await!(sasl_writer.send(SaslFrame::new(SaslFrameBody::SaslInit(SaslInit { mechanism: plain_symbol, initial_response: Some(initial_response), hostname: None }))))?;

    let (sasl_frame, sasl_reader) = await!(sasl_reader.into_future()).map_err(|e| e.0)?;
    println!("sasl.outcome: {:?}", sasl_frame);
    Ok(())
}