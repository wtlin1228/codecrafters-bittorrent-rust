use anyhow::Context;
use bittorrent_starter_rust::{
    decoder::decode_bencoded_value, torrent_file::parse_torrent_file, tracker::track,
};
use bytes::{BufMut, BytesMut};
use clap::{Parser, Subcommand};
use std::fs;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::path::PathBuf;

#[derive(Parser, Debug)]
struct Args {
    #[command(subcommand)]
    command: Command,
}
#[derive(Debug, Subcommand)]
enum Command {
    Decode {
        encoded_value: String,
    },
    Info {
        file_path: PathBuf,
    },
    Peers {
        file_path: PathBuf,
    },
    Handshake {
        file_path: PathBuf,
        peer: SocketAddr,
    },
}

fn main() {
    match Args::parse().command {
        Command::Decode { encoded_value } => {
            let decoded_value = decode_bencoded_value(encoded_value.as_bytes()).unwrap();
            println!("{}", decoded_value.to_string());
        }
        Command::Info { file_path } => {
            let contents = fs::read(file_path).unwrap();
            let torrent_file = parse_torrent_file(&contents[..]).unwrap();
            println!("Tracker URL: {}", torrent_file.announce);
            println!("Length: {}", torrent_file.info.length);
            println!("Info Hash: {}", torrent_file.info.hex_info().unwrap());
            println!("Piece Length: {}", torrent_file.info.piece_length);
            println!("Piece Hashes");
            for s in torrent_file.info.hex_pieces().unwrap() {
                println!("{}", s);
            }
        }
        Command::Peers { file_path } => {
            let contents = fs::read(file_path).unwrap();
            let torrent_file = parse_torrent_file(&contents[..]).unwrap();
            let track_result = track(torrent_file).unwrap();
            for peer in track_result.peers {
                println!("{}", peer.to_string());
            }
        }
        Command::Handshake { file_path, peer } => {
            let contents = fs::read(file_path).unwrap();
            let torrent_file = parse_torrent_file(&contents[..]).unwrap();

            let mut buf = BytesMut::with_capacity(1 + 19 + 8 + 20 + 20);
            buf.put_u8(19 as u8); // protocol length, 1 byte
            buf.put_slice(b"BitTorrent protocol"); // protocol, 19 bytes
            buf.put_bytes(0, 8); // reserved bytes, 8 bytes
            buf.put_slice(&torrent_file.info.hash_info().unwrap()); // info hash, 20 bytes
            buf.put_slice(b"00112233445566778899"); // peer id, 20 bytes

            let mut stream = TcpStream::connect(peer)
                .context("fail to connect to peer")
                .unwrap();

            stream
                .write(&buf)
                .context("fail to send handshake message")
                .unwrap();

            let mut buf = [0; 1 + 19 + 8 + 20 + 20];
            stream
                .read(&mut buf)
                .context("fail to read handshake response")
                .unwrap();

            println!("Peer ID: {}", hex::encode(&buf[buf.len() - 20..]));
        }
    }
}
