use anyhow::{Context, Ok, Result};
use serde::Serialize;
use sha1::{Digest, Sha1};

use crate::decoder::{decode, Decoded};

#[derive(PartialEq, Debug, Clone)]
pub struct TorrentFile {
    pub announce: String,
    pub info: TorrentFileInfo,
}

#[derive(Serialize, PartialEq, Debug, Clone)]
pub struct TorrentFileInfo {
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: u64,
    #[serde(with = "serde_bytes")]
    pub pieces: Vec<u8>,
    pub length: u64,
}

impl TorrentFileInfo {
    pub fn hash_info(&self) -> Result<[u8; 20]> {
        let bencoded_info_dictionary =
            serde_bencode::to_bytes(&self).context("hash info dictionary")?;
        let mut hasher = Sha1::new();
        hasher.update(bencoded_info_dictionary);
        Ok(hasher.finalize().into())
    }

    pub fn hex_info(&self) -> Result<String> {
        Ok(hex::encode(self.hash_info().context("get hash info")?))
    }

    pub fn url_encoded_hash_info(&self) -> Result<String> {
        Ok(self.hash_info().context("get hash info")?.iter().fold(
            "".to_string(),
            |mut acc, &byte| {
                acc.push_str("%");
                acc.push_str(&hex::encode([byte]));
                acc
            },
        ))
    }

    pub fn hex_pieces(&self) -> Result<Vec<String>> {
        Ok(self
            .pieces
            .chunks(20)
            .map(|chunk| hex::encode(chunk))
            .collect())
    }
}

pub fn parse_torrent_file(contents: &[u8]) -> Result<TorrentFile> {
    let decoded_value = decode(contents).context("decode file contents")?.1;

    let mut announce: Option<String> = None;
    let mut length: Option<u64> = None;
    let mut name: Option<String> = None;
    let mut piece_length: Option<u64> = None;
    let mut pieces: Option<Vec<u8>> = None;
    if let Decoded::Dictionary(dict) = decoded_value {
        if let Decoded::String(s) = dict.get("announce").context("should contain announce")? {
            announce = Some(
                std::str::from_utf8(s)
                    .context("announce isn't in valid UTF-8 format")?
                    .to_string(),
            );
        };
        if let Decoded::Dictionary(info) = dict.get("info").context("should contain info")? {
            if let Decoded::Integer(n) = info.get("length").context("should contain length")? {
                length = Some(n.to_owned() as u64);
            }
            if let Decoded::String(s) = info.get("name").context("should contain name")? {
                name = Some(
                    std::str::from_utf8(s)
                        .context("name isn't in valid UTF-8 format")?
                        .to_string(),
                );
            }
            if let Decoded::Integer(n) = info
                .get("piece length")
                .context("should contain piece length")?
            {
                piece_length = Some(n.to_owned() as u64);
            }
            if let Decoded::String(s) = info.get("pieces").context("should contain pieces")? {
                pieces = Some(s.to_vec());
            }
        }
    }

    Ok(TorrentFile {
        announce: announce.context("get announce from torrent file")?,
        info: TorrentFileInfo {
            length: length.context("get info.length from torrent file")?,
            name: name.context("get info.name from torrent file")?,
            piece_length: piece_length.context("get info.piece_length")?,
            pieces: pieces.context("get info.pieces")?,
        },
    })
}
