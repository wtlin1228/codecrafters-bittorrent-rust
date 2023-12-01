use serde_json;
use std::env;

// Available if you need it!
// use serde_bencode

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    match encoded_value.chars().next() {
        // string is encoded as <number>:<string>
        Some(c) if c.is_digit(10) => {
            let colon_index = encoded_value.find(':').unwrap();
            let number_string = &encoded_value[..colon_index];
            let number = number_string.parse::<i64>().unwrap();
            let string = &encoded_value[colon_index + 1..colon_index + 1 + number as usize];
            return serde_json::Value::String(string.to_string());
        }
        // integer is encoded as i<number>e
        Some(c) if c == 'i' => {
            let end_index = encoded_value.find('e').unwrap();
            let number_string = &encoded_value[1..end_index];
            let integer = number_string.parse::<i64>().unwrap();
            return serde_json::Value::Number(serde_json::Number::from(integer));
        }
        _ => panic!("Unhandled encoded value: {}", encoded_value),
    };
}

// Usage: your_bittorrent.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        // println!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value = decode_bencoded_value(encoded_value);
        println!("{}", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
