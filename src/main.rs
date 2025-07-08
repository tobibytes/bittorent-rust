use serde_json;
use std::env;
use serde_derive::{Serialize, Deserialize};
use serde_bencode::{self};


#[derive(Debug, Deserialize, Serialize)]
#[serde(untagged)]
enum BencodeValue {
    Int(i64),
    Str(String),
    List(Vec<BencodeValue>),
    Dict(std::collections::BTreeMap<String, BencodeValue>),
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        eprintln!("Logs from your program will appear here!");

        let encoded_value = &args[2];
        let value: BencodeValue = serde_bencode::from_str(encoded_value).unwrap();
        println!("{}", serde_json::to_string(&value).unwrap());
    } else {
        eprintln!("unknown command: {}", args[1])
    }
}