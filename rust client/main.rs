use std::env;
use std::error::Error;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use sha2::{Sha256, Digest};

fn to_hex_string(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{:02x}", b)).collect()
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
    let mut data: Vec<u8> = Vec::new();

    while current_offset < total_bytes {
        let range_header = format!("bytes={}-{}", current_offset, total_bytes);
        println!("Ask for range: {}", range_header);

        let addr = format!("{}:{}", host, port);
        let mut stream = TcpStream::connect(&addr)?;

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
                content_length = Some(line.split(':').nth(1).unwrap().trim().parse()?);
            }
        }

        let content_length = content_length.ok_or("No Content-Length header found")?;

        let mut chunk = vec![0u8; content_length];
        let mut read_total = 0;
        while read_total < content_length {
            let read_bytes = reader.read(&mut chunk[read_total..])?;
            if read_bytes == 0 {
                break;
            }
            read_total += read_bytes;
        }

        if read_total == 0 {
            eprintln!("Get 0 bytes, ending download.");
            break;
        }

        data.extend_from_slice(&chunk[..read_total]);
        current_offset += read_total;
        println!("Received {} bytes, total {}/{} bytes", read_total, current_offset, total_bytes);
    }

    println!("Total received: {} bytes", data.len());

    let mut hasher = Sha256::new();
    hasher.update(&data);
    let computed_hash = to_hex_string(&hasher.finalize());

    println!("Computed SHA-256: {}", computed_hash);
    println!("Expected SHA-256: {}", expected_hash);

    if computed_hash == expected_hash {
        println!("Data is correct!");
    } else {
        println!("Error in data!");
    }

    Ok(())
}
