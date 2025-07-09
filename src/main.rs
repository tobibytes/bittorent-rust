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
    Dict(std::collections::BTreeMap<String, BencodeValue>),
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
            // println!("{:?}", torrent);
            let torrent_info_bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
            let torrent_info_sha = sha1_smol::Sha1::from(&torrent_info_bytes).digest().to_string();
            println!("Tracker URL: {}", torrent.announce);
            println!("Length: {}", torrent.info.length);
            println!("Info Hash: {}", torrent_info_sha);
            println!("Pieces Hashes:");
            let mut i = 0;
            let torrent_pieces_bytes_len = torrent.info.pieces.len();
            loop {
                if i >= torrent_pieces_bytes_len {
                    break;
                }
                let piece_sha = &torrent.info.pieces[i..i+20];
                let torrent_piece_sha =sha1_smol::Sha1::from(&piece_sha).digest().to_string();
                println!("{}", torrent_piece_sha);
                i += 20;
            };

        }
    else {
        eprintln!("unknown command: {}", args[1])
    }
}