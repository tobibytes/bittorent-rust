use serde_json;
use std::borrow::Cow;
use tokio;
use sha1_smol::Sha1;
use std::{env, path::PathBuf};
use serde_derive::{Serialize, Deserialize};
use serde_bencode::{self};
use reqwest;
const COMPACT_DEFAULT: u8 = 1;
const PORT: usize = 6881;
const PEER_ID: &str = "kP8rXzN3QaYmVwB7TfL2";
const DOWNLOADED_DEFAULT: usize = 0;
const UPLOADED_DEFAULT: usize = 0;
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
struct Peers {
    #[serde(with = "serde_bytes")]
    port: [u8; 2], 
    #[serde(with = "serde_bytes")]
    ip: [u8; 4],
}
#[derive(Debug, Serialize, Deserialize)]
struct TrackerResponse {
    interval: usize,
    #[serde(with = "serde_bytes")]
    peers: Vec<u8>,
}
#[derive(Debug, Deserialize,Default, Serialize)]
struct TorrentInfo {
    length: usize,
    name: String,
    #[serde(rename = "piece length")]
    piece_length: usize,
    #[serde(with = "serde_bytes")]
    pieces: Vec<u8>
}

#[derive(Debug, Serialize, Deserialize)]
struct PeersQuery<'a> {
    info_hash: Vec<u8>,
    peer_id: &'a str,
    port: usize,
    uploaded: usize,
    downloaded: usize,
    left: usize,
    compact: u8
}

impl<'a> PeersQuery<'a> {
    fn new(info_hash: Vec<u8>, left: usize, uploaded: usize, downloaded: usize) -> Self {
        PeersQuery {
            info_hash,
            peer_id: PEER_ID,
            port: PORT,
            uploaded,
            downloaded,
            left,
            compact: COMPACT_DEFAULT,
        }
    }
}
fn load_torrent_file<T>(file_path: T) -> anyhow::Result<Torrent> where T: Into<PathBuf> {
    let content = std::fs::read(file_path.into()).unwrap();
    let torrent: Torrent = serde_bencode::from_bytes(&content).unwrap();
    Ok(torrent)
}
fn get_info(file_path: &str)-> (String, Vec<u8>, usize){
    let torrent: Torrent = load_torrent_file(file_path).unwrap();
    // println!("{:?}", torrent);
    let torrent_info_bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
    let torrent_info_sha = sha1_smol::Sha1::from(&torrent_info_bytes).digest().to_string();
    eprintln!("Tracker URL: {}", torrent.announce);
    eprintln!("Length: {}", torrent.info.length);
    println!("Info Hash: {}", torrent_info_sha);
    eprintln!("Piece Length: {}", torrent.info.piece_length);
    eprintln!("Piece Hashes:");
    let mut i = 0;
    let torrent_pieces_bytes_len = torrent.info.pieces.len();
    loop {
	if i >= torrent_pieces_bytes_len {
	    break;
	}
	let piece_sha = &torrent.info.pieces[i..i+20];
	let torrent_piece_sha = hex::encode(piece_sha);;
	i += 20;
};
(torrent.announce, torrent_info_bytes, torrent.info.length)
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match &args[1][..] {
 "decode" => {
        eprintln!("Logs from your program will appear here!");

	fn decode_value(encoded_value: &str) {
 let value: BencodeValue = serde_bencode::from_str(encoded_value).unwrap();
        println!("{}", serde_json::to_string(&value).unwrap());

}
               return Ok(());
    } 
 "info" => {
        let file_path = &args[2];
	get_info(file_path);
	return Ok(());
        } 
"peers" => {
        let file_path = &args[2];
        let decoded_torrent_tuple = get_info(file_path);
	    let url = decoded_torrent_tuple.0;
        let info_hash: Vec<u8> = decoded_torrent_tuple.1;
        let encoded_info_hash = percent_encode_sha1(&info_hash);
        let query = PeersQuery::new(info_hash, decoded_torrent_tuple.2, 0, 0);
        let format_query = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",&url, &encoded_info_hash, &query.peer_id, &query.port, &query.uploaded, &query.downloaded, &query.left, &query.compact);
       let body = reqwest::get(&format_query).await?.bytes().await?;
         let tracker_response: TrackerResponse = serde_bencode::from_bytes(&body).unwrap();
         decode_peers(&tracker_response.peers);
       return Ok(());
}
    _ => {
	println!("unkown command");
	return Ok(());
    }
   }
}
fn percent_encode_sha1(info_bytes: &[u8]) -> String {
    let digest = Sha1::from(info_bytes).digest();
    let digest_bytes = digest.bytes();
    digest_bytes
        .iter()
        .map(|b| format!("%{:02X}", b))
        .collect::<String>()
}

fn decode_peers(bytes: &[u8]) {
    for chunk in bytes.chunks(6) {
        if chunk.len() < 6 {
            continue; // skip incomplete peer entry
        }

        // First 4 bytes = IP
        let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);

        // Next 2 bytes = port (big-endian)
        let port = u16::from_be_bytes([chunk[4], chunk[5]]);

        println!("{}:{}", ip, port);

    }
}
