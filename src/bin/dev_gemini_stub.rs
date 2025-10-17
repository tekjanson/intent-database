use std::io::{Read, Write};
use std::net::TcpListener;

fn main() {
    // Very small HTTP server: accepts one connection at a time and responds
    // to POST /gemini with the request body echoed back as plain text.
    let listener = TcpListener::bind("127.0.0.1:8081").expect("bind stub");
    println!("dev_gemini_stub listening on 127.0.0.1:8081");
    for stream in listener.incoming() {
        if let Ok(mut s) = stream {
            let mut buf = [0u8; 4096];
            if let Ok(n) = s.read(&mut buf) {
                let req = String::from_utf8_lossy(&buf[..n]);
                // very naive parsing
                let mut lines = req.lines();
                let first = lines.next().unwrap_or("");
                if first.starts_with("POST /gemini") {
                    // find start of body
                    if let Some(pos) = req.find("\r\n\r\n") {
                        let body = &req[pos + 4..];
                        let resp = format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", body.len(), body);
                        let _ = s.write_all(resp.as_bytes());
                    } else {
                        let resp = "HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\n\r\n";
                        let _ = s.write_all(resp.as_bytes());
                    }
                } else {
                    let resp = "HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n";
                    let _ = s.write_all(resp.as_bytes());
                }
            }
        }
    }
}
