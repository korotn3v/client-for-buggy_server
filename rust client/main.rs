use std::env;
use std::error::Error;
use std::io::{Write, BufRead, BufReader, Read};
use std::net::TcpStream;
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

    let host = "127.0.0.1";
    let port = 8080;
    let mut current_offset = 0;
    let chunk_size = 64 * 1024; // 64 KB
    let mut data: Vec<u8> = Vec::new();

    while current_offset < total_bytes {
        let remaining = total_bytes - current_offset;
        let request_size = remaining.min(chunk_size);
        let range_header = format!("bytes={}-{}", current_offset, current_offset + request_size);
        println!("Ask for range: {}", range_header);

        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(addr)?;

        let request = format!(
            "GET / HTTP/1.1\r\nHost: {}:{}\r\nRange: {}\r\nConnection: close\r\n\r\n",
            host, port, range_header
        );
        stream.write_all(request.as_bytes())?;

        let mut reader = BufReader::new(stream);

        let mut status_line = String::new();
        reader.read_line(&mut status_line)?;
        let status_parts: Vec<&str> = status_line.split_whitespace().collect();
        if status_parts.len() < 2 {
            eprintln!("Invalid HTTP response: {}", status_line);
            std::process::exit(1);
        }
        let status_code: u16 = status_parts[1].parse()?;
        if !(status_code == 200 || status_code == 206) {
            eprintln!("Error: status code {}", status_code);
            std::process::exit(1);
        }

        let mut content_length: Option<usize> = None;
        loop {
            let mut line = String::new();
            reader.read_line(&mut line)?;
            if line == "\r\n" || line.is_empty() {
                break;
            }
            if line.to_lowercase().starts_with("content-length:") {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let len_str = parts[1].trim();
                    content_length = Some(len_str.parse()?);
                }
            }
        }
        let content_length = match content_length {
            Some(len) => len,
            None => {
                eprintln!("No Content-Length header found.");
                std::process::exit(1);
            }
        };

        let mut chunk = vec![0u8; content_length];
        let mut read_total = 0;
        while read_total < content_length {
            let read_bytes = reader.read(&mut chunk[read_total..])?;
            if read_bytes == 0 { break; }
            read_total += read_bytes;
        }
        let received = read_total;
        if received == 0 {
            eprintln!("Get 0 bytes, end the download.");
            break;
        }
        data.extend_from_slice(&chunk[..received]);
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

