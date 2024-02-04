use anyhow::{Context, Ok, Result};
use bittorrent_starter_rust::{
    decoder::decode_bencoded_value,
    handshake::{get_handshake_response, prepare_handshake_message},
    torrent_file::parse_torrent_file,
    tracker::track,
};
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
}

fn main() -> Result<()> {
    match Args::parse().command {
        Command::Decode { encoded_value } => {
            let decoded_value =
                decode_bencoded_value(encoded_value.as_bytes()).context("fail to decode value")?;
            println!("{}", decoded_value.to_string());
        }
        Command::Info { file_path } => {
            let contents = fs::read(file_path).context("fail to open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("fail to parse file")?;
            println!("Tracker URL: {}", torrent_file.announce);
            println!("Length: {}", torrent_file.info.length);
            println!(
                "Info Hash: {}",
                torrent_file.info.hex_info().context("fail to hash info")?
            );
            println!("Piece Length: {}", torrent_file.info.piece_length);
            println!("Piece Hashes");
            for s in torrent_file
                .info
                .hex_pieces()
                .context("fail to hex pieces")?
            {
                println!("{}", s);
            }
        }
        Command::Peers { file_path } => {
            let contents = fs::read(file_path).context("fail to open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("fail to parse file")?;
            let track_result = track(torrent_file).context("fail to track peers")?;
            for peer in track_result.peers {
                println!("{}", peer.to_string());
            }
        }
        Command::Handshake { file_path, peer } => {
            let contents = fs::read(file_path).context("fail to open file")?;
            let torrent_file = parse_torrent_file(&contents[..]).context("fail to parse file")?;
            let mut stream = TcpStream::connect(peer).context("fail to connect to peer")?;
            let message = prepare_handshake_message(&torrent_file)
                .context("fail to prepare handshake message")?;
            stream
                .write(&message)
                .context("fail to send handshake message")?;
            let response =
                get_handshake_response(&mut stream).context("fail to get handshake response")?;
            println!("Peer ID: {}", hex::encode(&response[response.len() - 20..]));
        }
    }
    Ok(())
}
