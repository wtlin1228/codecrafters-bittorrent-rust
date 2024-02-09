use crate::peer::Peer;
use crate::{torrent_file::parse_torrent_file, tracker::track};
use anyhow::{Context, Error, Ok, Result};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::{collections::HashSet, fs, path::PathBuf};

pub struct Download;

impl Download {
    pub fn download_file(torrent_file_path: &PathBuf, output_file_path: &PathBuf) -> Result<()> {
        // Read the torrent file to get the tracker URL
        let contents = fs::read(torrent_file_path).context("open file")?;
        let torrent_file = parse_torrent_file(&contents[..]).context("parse file")?;

        // Perform the tracker GET request to get a list of peers
        let track_result = track(&torrent_file).context("track peers")?;
        let peer_addr_list: Vec<String> = track_result
            .peer_addr_list
            .iter()
            .map(|addr| addr.to_string())
            .collect();

        // Get how many pieces need to be downloaded
        let hexed_pieces: Vec<String> = torrent_file.info.hex_pieces().context("hex pieces")?;
        let pieces_to_download: HashSet<usize> = (0..hexed_pieces.len()).collect();

        // Start downloading n pieces from m peers
        let (tx, rx) = mpsc::channel::<(u32, Result<Vec<u8>>)>();
        let mut handles: Vec<JoinHandle<()>> = vec![];
        for piece_index in pieces_to_download {
            // 5 pieces, 3 peers
            // get #0 piece from #0 peer
            // get #1 piece from #1 peer
            // get #2 piece from #2 peer
            // get #3 piece from #0 peer
            // get #4 piece from #1 peer
            let peer_addr = peer_addr_list
                .get(piece_index % peer_addr_list.len())
                .with_context(|| format!("get peer address for #{} piece", piece_index))?
                .clone();
            let torrent_file = torrent_file.clone();
            let tx = tx.clone();
            let handle = thread::spawn(move || {
                // Connect to peer
                let peer = Peer::new(peer_addr.clone(), torrent_file);
                if peer.is_err() {
                    tx.send((
                        piece_index as u32,
                        Err(Error::msg(format!("fail to connect to peer {}", peer_addr))),
                    ))
                    .unwrap();
                    return;
                }
                let mut peer = peer.unwrap();

                // Download a piece
                let piece = peer.download_a_piece(piece_index as u32);
                if piece.is_err() {
                    tx.send((
                        piece_index as u32,
                        Err(Error::msg(format!(
                            "fail to download #{} piece",
                            piece_index
                        ))),
                    ))
                    .unwrap();
                    return;
                }
                let piece = piece.unwrap();

                // Send downloaded piece back to the main thread
                tx.send((piece_index as u32, Ok(piece))).unwrap();
            });
            handles.push(handle);
        }

        Ok(())
    }
}
