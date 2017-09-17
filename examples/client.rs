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
use amqp::io::{AmqpDecoder, AmqpEncoder};
use amqp::protocol::{ProtocolId, encode_protocol_header, decode_protocol_header};
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
    println!("{:?}", protocol_id);
    let sasl_reader = tokio_io::codec::FramedRead::new(reader, AmqpDecoder::<amqp::framing::SaslFrame>::new());
    let (mechanisms, sasl_reader) = await!(sasl_reader.into_future()).map_err(|e| e.0)?;
    println!("{:?}", mechanisms);
    Ok(())
}