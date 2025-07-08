use serde_json;
use std::env;

// Available if you need it!
use serde_bencode::{self, from_str};

#[allow(dead_code)]
fn decode_bencoded_value(encoded_value: &str) -> serde_json::Value {
    if encoded_value.chars().next().unwrap().is_digit(10) {
        let colon_index = encoded_value.find(':').unwrap();
        let number_string = &encoded_value[..colon_index];
        let number = number_string.parse::<usize>().unwrap();
        let string = &encoded_value[colon_index + 1..colon_index + 1 + number];
        return serde_json::Value::String(string.to_string());

    } else if encoded_value.chars().next().unwrap() == 'i' {
        let start_string = encoded_value.find("i").unwrap();
        let end_string = encoded_value.find("e").unwrap();
        let string = &encoded_value[start_string+1..end_string-1];
        return serde_json::Value::String(string.to_string());
    }

    else {
        panic!("Unhandled encoded value: {}", encoded_value)
    }
    
}

// Usage: your_program.sh decode "<encoded_value>"
fn main() {
    let args: Vec<String> = env::args().collect();
    let command = &args[1];

    if command == "decode" {
        // You can use print statements as follows for debugging, they'll be visible when running tests.
        eprintln!("Logs from your program will appear here!");

        // Uncomment this block to pass the first stage
        let encoded_value = &args[2];
        let decoded_value: serde_json::Value = from_str(&encoded_value.to_string()).unwrap();
        // let decoded_value = decode_bencoded_value(encoded_value);
        print!("{}\n", decoded_value.to_string());
    } else {
        println!("unknown command: {}", args[1])
    }
}
