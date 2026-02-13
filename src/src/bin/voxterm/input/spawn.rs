//! Input-thread bootstrap so stdin capture stays isolated from render/event logic.

use crossbeam_channel::Sender;
use std::io::{self, Read};
use std::thread;
use std::time::{Duration, Instant};
use voxterm::log_debug;

use crate::arrow_keys::parse_arrow_keys_only;
use crate::input::event::InputEvent;
use crate::input::parser::InputParser;

const INPUT_DEBUG_ENV: &str = "VOXTERM_DEBUG_INPUT";
const INPUT_DEBUG_MAX_BYTES: usize = 64;
const STARTUP_ESCAPE_DROP_WINDOW_MS: u64 = 700;

fn input_debug_enabled() -> bool {
    std::env::var(INPUT_DEBUG_ENV).is_ok()
}

fn format_debug_bytes(bytes: &[u8]) -> String {
    let sample_len = bytes.len().min(INPUT_DEBUG_MAX_BYTES);
    let mut out = String::new();
    for (idx, byte) in bytes.iter().take(sample_len).enumerate() {
        if idx > 0 {
            out.push(' ');
        }
        out.push_str(&format!("{byte:02x}"));
    }
    if bytes.len() > sample_len {
        out.push_str(" ...");
    }
    out
}

fn should_drop_startup_escape_noise(bytes: &[u8]) -> bool {
    if bytes.is_empty() {
        return false;
    }

    if parse_arrow_keys_only(bytes).is_some() {
        return true;
    }

    let mut saw_escape = false;
    for &byte in bytes {
        match byte {
            0x1b => saw_escape = true,
            b'[' | b';' | b'0'..=b'9' | b'A' | b'B' | b'C' | b'D' => {}
            _ => return false,
        }
    }
    saw_escape
}

pub(crate) fn spawn_input_thread(tx: Sender<InputEvent>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut stdin = io::stdin();
        let mut buf = [0u8; 1024];
        let mut parser = InputParser::new();
        let debug_input = input_debug_enabled();
        let startup_drop_deadline =
            Instant::now() + Duration::from_millis(STARTUP_ESCAPE_DROP_WINDOW_MS);
        loop {
            let n = match stdin.read(&mut buf) {
                Ok(0) => break,
                Ok(n) => n,
                Err(err) => {
                    log_debug(&format!("stdin read error: {err}"));
                    break;
                }
            };
            if Instant::now() < startup_drop_deadline && should_drop_startup_escape_noise(&buf[..n])
            {
                if debug_input {
                    log_debug(&format!(
                        "dropped startup escape noise ({}): {}",
                        n,
                        format_debug_bytes(&buf[..n])
                    ));
                }
                continue;
            }
            if debug_input {
                log_debug(&format!(
                    "input bytes ({}): {}",
                    n,
                    format_debug_bytes(&buf[..n])
                ));
            }
            let mut events = Vec::new();
            parser.consume_bytes(&buf[..n], &mut events);
            parser.flush_pending(&mut events);
            if debug_input && !events.is_empty() {
                log_debug(&format!("input events: {events:?}"));
            }
            for event in events {
                if tx.send(event).is_err() {
                    return;
                }
            }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::should_drop_startup_escape_noise;

    #[test]
    fn startup_escape_noise_drops_arrow_sequences() {
        assert!(should_drop_startup_escape_noise(b"\x1b[A"));
        assert!(should_drop_startup_escape_noise(b"\x1b[B\x1b[C"));
        assert!(should_drop_startup_escape_noise(b"\x1b[1;2A"));
    }

    #[test]
    fn startup_escape_noise_drops_partial_arrow_fragments() {
        assert!(should_drop_startup_escape_noise(b"\x1b["));
        assert!(should_drop_startup_escape_noise(b"\x1b[1;"));
    }

    #[test]
    fn startup_escape_noise_keeps_normal_input() {
        assert!(!should_drop_startup_escape_noise(b"hello"));
        assert!(!should_drop_startup_escape_noise(b"\x1b[31mred\x1b[0m"));
        assert!(!should_drop_startup_escape_noise(b"\n"));
    }
}
