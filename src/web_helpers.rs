use crate::conversation::Conversation;
use std::io::Read as IoRead;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct PendingReview {
    pub id: String,
    pub conv: Conversation,
    pub user_response: String,
    pub ai_response: String,
}

/// A small reader that wraps an mpsc::Receiver<String> and implements Read
/// by returning SSE-framed 'data: <json>\n\n' chunks when available.
pub struct ReceiverReader {
    pub rx: mpsc::Receiver<String>,
    pub buf: Vec<u8>,
}

impl IoRead for ReceiverReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.buf.is_empty() {
            match self.rx.recv() {
                Ok(s) => {
                    let ev = format!("data: {}\n\n", s);
                    self.buf = ev.into_bytes();
                }
                Err(_) => return Ok(0),
            }
        }
        let n = std::cmp::min(out.len(), self.buf.len());
        out[..n].copy_from_slice(&self.buf[..n]);
        self.buf.drain(..n);
        Ok(n)
    }
}

/// Helper to broadcast a JSON string to every sender in the broadcasters list.
pub fn broadcast_graph(broadcasters: &Arc<Mutex<Vec<std::sync::mpsc::Sender<String>>>>, s: String) {
    let mut guard = broadcasters.lock().unwrap();
    guard.retain(|tx| tx.send(s.clone()).is_ok());
}
