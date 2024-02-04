use anyhow::{Context, Ok, Result};
use bytes::{BufMut, Bytes, BytesMut};
use std::io::{Read, Write};
use std::net::TcpStream;
use std::u32;

#[derive(PartialEq)]
pub enum PeerMessageType {
    Choke,
    Unchoke,
    Interested,
    NotInterested,
    Have,
    Bitfield,
    Request,
    Piece,
    Cancel,
}

impl PeerMessageType {
    fn get_message_id(&self) -> u8 {
        match self {
            PeerMessageType::Choke => 0,
            PeerMessageType::Unchoke => 1,
            PeerMessageType::Interested => 2,
            PeerMessageType::NotInterested => 3,
            PeerMessageType::Have => 4,
            PeerMessageType::Bitfield => 5,
            PeerMessageType::Request => 6,
            PeerMessageType::Piece => 7,
            PeerMessageType::Cancel => 8,
        }
    }
}

#[derive(Debug)]
pub struct PeerMessage {
    pub message_length_prefix: u32,
    pub message_id: u8,
    pub payload: Bytes,
}

impl PeerMessage {
    pub fn get_message_type(&self) -> PeerMessageType {
        match self.message_id {
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
    let message_id: u8 = message_id[0];
    println!("  message_id = {}", message_id);
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
    message.put_u32(payload.len() as u32);
    println!("  message_id = {}", message_type.get_message_id());
    message.put_u8(message_type.get_message_id());
    println!("  payload = {:?}", payload);
    message.put(payload);
    let message = message.freeze();
    stream.write(&message).context("send peer message")?;
    Ok(())
}
