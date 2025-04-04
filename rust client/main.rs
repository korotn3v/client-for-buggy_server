use std::env;
use std::error::Error;
use reqwest::blocking::Client;
use sha2::{Sha256, Digest};

fn to_hex_string(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for &byte in bytes {
        s.push_str(&format!("{:02x}", byte));
    }
    s
}

fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <number_of_bytes> <expected_sha256_hash>", args[0]);
        std::process::exit(1);
    }

    let total_bytes: usize = args[1].parse()?;
    let expected_hash = args[2].to_lowercase();

    let client = Client::new();
    let url = "http://127.0.0.1:8080/";

    let mut current_offset = 0;
    let chunk_size = 64 * 1024; // 64 KB
    let mut data: Vec<u8> = Vec::new();

    while current_offset < total_bytes {
        let remaining = total_bytes - current_offset;
        let request_size = remaining.min(chunk_size);
        // if we ask 0-65536, python will give us 0-65535
        let range_header = format!("bytes={}-{}", current_offset, current_offset + request_size);

        println!("Ask for range: {}", range_header);

        let response = client.get(url)
            .header("Range", &range_header)
            .send()?;

        if !response.status().is_success() && response.status().as_u16() != 206 {
            eprintln!("Error: status code {}", response.status());
            std::process::exit(1);
        }

        let chunk = response.bytes()?;
        let received = chunk.len();

        if received == 0 {
            eprintln!("Get 0 bytes, end the download.");
            break;
        }

        data.extend_from_slice(&chunk);
        current_offset += received;
        println!("Get {} bytes, summary {}/{} bytes", received, current_offset, total_bytes);
    }

    println!("Total get: {} bytes", data.len());

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_hash = to_hex_string(&hasher.finalize());

    println!("computed_hash SHA-256: {}", computed_hash);
    println!("expected_hash SHA-256: {}", expected_hash);

    if computed_hash == expected_hash {
        println!("data is correct!");
    } else {
        println!("error in data!");
    }

    Ok(())
}

