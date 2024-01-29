use bittorrent_starter_rust::decode_bencoded_value;
use std::env;
use std::fs;

// Available if you need it!
// use serde_bencode

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value.as_bytes());
        println!("{}", decoded_value.to_string());
    } else if command == "info" {
        let file_path = &args[2];
        let contents = fs::read(file_path).expect("Should have been able to read the file");
        let decoded_value = decode_bencoded_value(&contents[..]);
        println!(
            "Tracker URL: {}",
            decoded_value.get("announce").unwrap().as_str().unwrap()
        );
        println!(
            "Length: {}",
            decoded_value.get("info").unwrap().get("length").unwrap()
        );
    } else {
        println!("unknown command: {}", args[1])
    }
}
