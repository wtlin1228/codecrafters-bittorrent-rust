use anyhow::{Context, Ok, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::u32;

#[derive(Debug, PartialEq)]
#[repr(u8)]
pub enum PeerMessageType {
    Choke = 0,
    Unchoke = 1,
    Interested = 2,
    NotInterested = 3,
    Have = 4,
    Bitfield = 5,
    Request = 6,
    Piece = 7,
    Cancel = 8,
}

impl PeerMessageType {
    fn has_payload(&self) -> bool {
        match self {
            PeerMessageType::Choke => false,
            PeerMessageType::Unchoke => false,
            PeerMessageType::Interested => false,
            PeerMessageType::NotInterested => false,
            PeerMessageType::Have => true,
            PeerMessageType::Bitfield => true,
            PeerMessageType::Request => true,
            PeerMessageType::Piece => true,
            PeerMessageType::Cancel => true,
        }
    }
}

#[derive(Debug)]
pub struct PeerMessage {
    pub message_length_prefix: u32,
    pub message_id: PeerMessageType,
    pub payload: Bytes,
}

pub fn get_message_type(message_id: u8) -> PeerMessageType {
    match message_id {
        0 => PeerMessageType::Choke,
        1 => PeerMessageType::Unchoke,
        2 => PeerMessageType::Interested,
        3 => PeerMessageType::NotInterested,
        4 => PeerMessageType::Have,
        5 => PeerMessageType::Bitfield,
        6 => PeerMessageType::Request,
        7 => PeerMessageType::Piece,
        8 => PeerMessageType::Cancel,
        _ => unreachable!(),
    }
}

pub fn get_peer_message(stream: &mut TcpStream) -> Result<PeerMessage> {
    println!("Waiting peer message");
    let mut message_length_prefix = [0; 4];
    stream
        .read_exact(&mut message_length_prefix)
        .context("read message length prefix")?;
    let message_length_prefix: u32 = u32::from_be_bytes(message_length_prefix);
    println!("  message_length_prefix = {}", message_length_prefix);
    let mut message_id = [0; 1];
    stream
        .read_exact(&mut message_id)
        .context("read message id")?;
    let message_id = get_message_type(message_id[0]);
    println!("  message_id = {:?}", message_id);

    if !message_id.has_payload() {
        return Ok(PeerMessage {
            message_length_prefix,
            message_id,
            payload: Bytes::new(),
        });
    }

    let mut payload = BytesMut::with_capacity(message_length_prefix as usize - 1);
    stream.read_exact(&mut payload).context("read payload")?;
    println!("  payload = {:?}", payload);
    Ok(PeerMessage {
        message_length_prefix,
        message_id,
        payload: payload.freeze(),
    })
}

pub fn send_peer_message(
    stream: &mut TcpStream,
    message_type: PeerMessageType,
    payload: Bytes,
) -> Result<()> {
    println!("Sending peer message");
    let mut message = BytesMut::with_capacity(5 + payload.len());
    println!("  message_length_prefix = {}", 1 + payload.len());
    message.put_u32(1 + payload.len() as u32);
    println!("  message_id = {:?}", message_type);
    message.put_u8(message_type as u8);
    println!("  payload = {:?}", payload);
    message.put(payload);
    let message = message.freeze();
    stream.write(&message).context("send peer message")?;
    Ok(())
}
