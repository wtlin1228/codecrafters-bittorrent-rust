use anyhow::{Context, Ok, Result};
use bittorrent_starter_rust::{
    decoder::decode_bencoded_value,
    handshake::{get_handshake_response, prepare_handshake_message},
    peer_message::{get_peer_message, send_peer_message, PeerMessageType},
    torrent_file::parse_torrent_file,
    tracker::track,
};
use bytes::Bytes;
use clap::{Parser, Subcommand};
use std::fs;
use std::io::Write;
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
    #[command(name = "download_piece")]
    DownloadPiece {
        #[arg(short)]
        output_file_path: PathBuf,
        file_path: PathBuf,
        piece_index: usize,
    },
}

fn main() -> Result<()> {
    match Args::parse().command {
        Command::Decode { encoded_value } => {
            let decoded_value =
                decode_bencoded_value(encoded_value.as_bytes()).context("decode value")?;
            println!("{}", decoded_value.to_string());
        }
        Command::Info { file_path } => {
            let contents = fs::read(file_path).context("open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("parse file")?;
            println!("Tracker URL: {}", torrent_file.announce);
            println!("Length: {}", torrent_file.info.length);
            println!(
                "Info Hash: {}",
                torrent_file.info.hex_info().context("hash info")?
            );
            println!("Piece Length: {}", torrent_file.info.piece_length);
            println!("Piece Hashes");
            for s in torrent_file.info.hex_pieces().context("hex pieces")? {
                println!("{}", s);
            }
        }
        Command::Peers { file_path } => {
            let contents = fs::read(file_path).context("open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("parse file")?;
            let track_result = track(&torrent_file).context("track peers")?;
            for peer in track_result.peers {
                println!("{}", peer.to_string());
            }
        }
        Command::Handshake { file_path, peer } => {
            let contents = fs::read(file_path).context("open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("parse file")?;
            let mut stream = TcpStream::connect(peer).context("connect to peer")?;
            let message =
                prepare_handshake_message(&torrent_file).context("prepare handshake message")?;
            stream.write(&message).context("send handshake message")?;
            let response = get_handshake_response(&mut stream).context("get handshake response")?;
            println!("Peer ID: {}", hex::encode(&response[response.len() - 20..]));
        }
        Command::DownloadPiece {
            output_file_path,
            file_path,
            piece_index,
        } => {
            let contents = fs::read(file_path).context("open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("parse file")?;
            let track_result = track(&torrent_file).context("track peers")?;
            let first_peer = track_result
                .peers
                .first()
                .context("get the first peer")?
                .to_string();
            let mut stream = TcpStream::connect(first_peer).context("connect to peer")?;
            let message =
                prepare_handshake_message(&torrent_file).context("prepare handshake message")?;
            stream.write(&message).context("send handshake message")?;
            let _handshake_response =
                get_handshake_response(&mut stream).context("get handshake response")?;

            // Wait for a bitfield message from the peer
            let bitfield = get_peer_message(&mut stream).context("get bitfield message")?;
            println!("{:?}", bitfield);

            // Send an interested message
            send_peer_message(&mut stream, PeerMessageType::Interested, Bytes::new())
                .context("send interested message")?;

            // Wait until you receive an unchoke message back
            let mut response_message = get_peer_message(&mut stream).context("get message")?;
            println!("{:?}", response_message);
            while response_message.get_message_type() != PeerMessageType::Unchoke {
                response_message = get_peer_message(&mut stream).context("get message")?;
            }
        }
    }
    Ok(())
}
