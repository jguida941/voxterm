use crossbeam_channel::Sender;
use std::io::{self, Read};
use std::thread;
use voxterm::log_debug;

use crate::input::event::InputEvent;
use crate::input::parser::InputParser;

pub(crate) fn spawn_input_thread(tx: Sender<InputEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut buf = [0u8; 1024];
        let mut parser = InputParser::new();
        loop {
            let n = match stdin.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) => {
                    log_debug(&format!("stdin read error: {err}"));
                    break;
                }
            };
            let mut events = Vec::new();
            parser.consume_bytes(&buf[..n], &mut events);
            parser.flush_pending(&mut events);
            for event in events {
                if tx.send(event).is_err() {
                    return;
                }
            }
        }
    })
}
