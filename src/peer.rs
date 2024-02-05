use anyhow::Result;
use std::net::{SocketAddrV4, TcpStream};

use crate::torrent_file::TorrentFile;

pub struct Peer {
    addr: SocketAddrV4,
    stream: TcpStream,
}

impl Peer {
    pub fn new(peer_addr: SocketAddrV4, torrent_file: &TorrentFile) -> Result<Self> {
        let peer = TcpStream::connect(peer_addr);
        todo!()
    }
}
