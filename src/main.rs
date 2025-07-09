use serde_json;
use std::{env, path::PathBuf};
use serde_derive::{Serialize, Deserialize};
use serde_bencode::{self};

#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum BencodeValue {
    Int(i64),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<BencodeValue>),
    Dict(std::collections::BTreeMap<Vec<u8>, BencodeValue>),
}
#[derive(Debug, Deserialize, Serialize)]
struct Torrent {
    announce: String,
    info: TorrentInfo
}

#[derive(Debug, Deserialize, Serialize)]
struct TorrentInfo {
    length: usize,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>
}
fn load_torrent_file<T>(file_path: T) -> anyhow::Result<Torrent> where T: Into<PathBuf> {
    let content = std::fs::read(file_path.into()).unwrap();
    let torrent: Torrent = serde_bencode::from_bytes(&content).unwrap();
    Ok(torrent)
}
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        eprintln!("Logs from your program will appear here!");
        let encoded_value = &args[2];
        let value: BencodeValue = serde_bencode::from_str(encoded_value).unwrap();
        println!("{}", serde_json::to_string(&value).unwrap())
    } 
    else if command == "info" {
        let file_path = &args[2];
            let torrent: Torrent = load_torrent_file(file_path).unwrap();
            println!("{}", torrent.announce);
            println!("{}", torrent.info.length);
        }
    else {
        eprintln!("unknown command: {}", args[1])
    }
}