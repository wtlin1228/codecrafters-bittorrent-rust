use crate::torrent_file::TorrentFile;
use anyhow::{Context, Ok, Result};
use bytes::{BufMut, Bytes, BytesMut};

const HANDSHAKE_MESSAGE_SIZE: usize = 1 + 19 + 8 + 20 + 20;

pub fn prepare_handshake_message(torrent_file: &TorrentFile) -> Result<Bytes> {
    let mut message = BytesMut::with_capacity(HANDSHAKE_MESSAGE_SIZE);
    message.put_u8(19 as u8); // protocol length, 1 byte
    message.put_slice(b"BitTorrent protocol"); // protocol, 19 bytes
    message.put_bytes(0, 8); // reserved bytes, 8 bytes
    message.put_slice(&torrent_file.info.hash_info().context("fail to hash info")?); // info hash, 20 bytes
    message.put_slice(b"00112233445566778899"); // peer id, 20 bytes
    let message = message.freeze();
    Ok(message)
}
