use anyhow::{Context, Ok, Result};
use bittorrent_starter_rust::decoder::decode_bencoded_value;
use bittorrent_starter_rust::handshake::Handshake;
use bittorrent_starter_rust::torrent_file::parse_torrent_file;
use bittorrent_starter_rust::tracker::track;
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
    #[command(name = "download_piece")]
    DownloadPiece {
        #[arg(short)]
        output_file_path: PathBuf,
        file_path: PathBuf,
        piece_index: i32,
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
            let info_hash = torrent_file.info.hash_info().context("hash info")?;
            let mut stream = TcpStream::connect(peer).context("connect to peer")?;
            let mut handshake = Handshake::new(info_hash);
            let handshake_bytes = handshake.as_bytes_mut();
            stream
                .write(handshake_bytes)
                .context("send handshake request")?;
            stream
                .read_exact(handshake_bytes)
                .context("read handshake response")?;
            assert_eq!(handshake.protocol_length, 19);
            assert_eq!(&handshake.protocol, b"BitTorrent protocol");
            assert_eq!(handshake.info_hash, info_hash);
            println!("Peer ID: {}", hex::encode(&handshake.peer_id));
        }
        Command::DownloadPiece {
            output_file_path: _output_file_path,
            file_path,
            piece_index: _piece_index,
        } => {
            let contents = fs::read(file_path).context("open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("parse file")?;
            let track_result = track(&torrent_file).context("track peers")?;
            let _first_peer = track_result
                .peers
                .first()
                .context("get the first peer")?
                .to_string();

            todo!("Rrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrrr")
        }
    }
    Ok(())
}
