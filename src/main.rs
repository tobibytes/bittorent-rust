use serde_json;
use std::net::TcpStream;
use std::io::prelude::*;
use rand::Rng;
use tokio;
use hex;
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

#[derive(Debug)]
struct Peer {
    ip: String,
    port: usize,
    address: String
}

impl Peer {
    fn new(ip_address: &String) -> Peer {
        let add_arr: Vec<String> = ip_address.split(':').map(String::from).collect();
        let ip_string = &add_arr[0];
        let port_str: usize = add_arr[1].parse().unwrap();
        return Peer {
            ip: ip_string.clone(),
            port: port_str,
            address: ip_address.clone()
    }
}

    fn construct_message(self: &Self, buffer: &mut Vec<u8>, info_hash: &Vec<u8>) {
        let mut rng = rand::thread_rng();
        let rand_arr: [u8;20] = rng.gen();
        let digest = Sha1::from(info_hash).digest();
        let digest_bytes = digest.bytes();
        
        buffer.push(19);   
        buffer.extend_from_slice("BitTorrent protocol".as_bytes());   
        buffer.extend_from_slice(&[0;8]);
        buffer.extend_from_slice(&digest_bytes);
        buffer.extend_from_slice(&rand_arr);
    }
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

fn get_info(file_path: &str, show_info: bool)-> (String, Vec<u8>, usize){
    let torrent: Torrent = load_torrent_file(file_path).unwrap();
    let torrent_info_bytes = serde_bencode::to_bytes(&torrent.info).unwrap();
    let torrent_info_sha = sha1_smol::Sha1::from(&torrent_info_bytes).digest().to_string();
    if show_info {
    println!("Tracker URL: {}", torrent.announce);
    println!("Length: {}", torrent.info.length);
    println!("Info Hash: {}", torrent_info_sha);
    println!("Piece Length: {}", torrent.info.piece_length);
    println!("Piece Hashes:");
    };
    let mut i = 0;
    let torrent_pieces_bytes_len = torrent.info.pieces.len();
    loop {
	if i >= torrent_pieces_bytes_len {
	    break;
	}
	let piece_sha = &torrent.info.pieces[i..i+20];
	let torrent_piece_sha = hex::encode(&piece_sha);
        if show_info {
            println!("{}", torrent_piece_sha);
        }
	i += 20;
    };
    (torrent.announce, torrent_info_bytes, torrent.info.length)
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
            continue;
        }
        
        let ip = format!("{}.{}.{}.{}", chunk[0], chunk[1], chunk[2], chunk[3]);

        let port = u16::from_be_bytes([chunk[4], chunk[5]]);
        println!("{}:{}", ip, port);

    }
}
fn decode_value(encoded_value: &str) {
 let value: BencodeValue = serde_bencode::from_str(encoded_value).unwrap();
 println!("{}", serde_json::to_string(&value).unwrap());

}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    match command.as_str() {
 "decode" => {
        eprintln!("Logs from your program will appear here!");
        decode_value(&args[2]);
        return Ok(());
    } 
 "info" => {
        let file_path = &args[2];
	get_info(file_path, true);
	return Ok(());
        } 
"peers" => {
        let file_path = &args[2];
        let decoded_torrent_tuple = get_info(file_path, false);
	    let url = decoded_torrent_tuple.0;
        let info_hash: Vec<u8> = decoded_torrent_tuple.1;
        let encoded_info_hash = percent_encode_sha1(&info_hash);
        let query = PeersQuery::new(info_hash, decoded_torrent_tuple.2, UPLOADED_DEFAULT, DOWNLOADED_DEFAULT);
        let format_query = format!("{}?info_hash={}&peer_id={}&port={}&uploaded={}&downloaded={}&left={}&compact={}",&url, &encoded_info_hash, &query.peer_id, &query.port, &query.uploaded, &query.downloaded, &query.left, &query.compact);
       let body = reqwest::get(&format_query).await?.bytes().await?;
         let tracker_response: TrackerResponse = serde_bencode::from_bytes(&body).unwrap();
         decode_peers(&tracker_response.peers);
       return Ok(());
}
"handshake" => {
    let file_path = &args[2];
    let peer_ip_address = &args[3];
    let peer = Peer::new(peer_ip_address);
    let mut stream = TcpStream::connect(&peer.address)?;
    let decoded_info = get_info(&file_path, false);
    let mut send_buffer: Vec<u8> = Vec::new();
    let mut rec_buffer: [u8;68] = [0;68];
    peer.construct_message(&mut send_buffer, &decoded_info.1);
    stream.write_all(&send_buffer)?;
    stream.flush()?;
    // read
    let rec_bytes = stream.read(&mut rec_buffer)?;
    let pid = hex::encode(&rec_buffer[48..]);
    println!("Peer ID: {}", pid);
    return Ok(())
} 
    _ => {
	println!("unkown command");
	return Ok(());
    }
   }
}

