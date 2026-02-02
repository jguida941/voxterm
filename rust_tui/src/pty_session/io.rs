use crate::log_debug;
use anyhow::{anyhow, Result};
use crossbeam_channel::Sender;
use std::io::{self, ErrorKind};
use std::os::unix::io::RawFd;
use std::thread;
use std::time::Duration;
#[cfg(any(test, feature = "mutants"))]
use std::time::Instant;

#[cfg(any(test, feature = "mutants"))]
use super::counters::guard_loop;
use super::counters::write_all_limit;
use super::osc::{respond_to_terminal_queries, respond_to_terminal_queries_passthrough};

pub(super) fn should_retry_read_error(err: &io::Error) -> bool {
    err.kind() == ErrorKind::Interrupted || err.kind() == ErrorKind::WouldBlock
}

/// Continuously read from the PTY and forward chunks to the main thread.
pub(super) fn spawn_reader_thread(master_fd: RawFd, tx: Sender<Vec<u8>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        #[cfg(any(test, feature = "mutants"))]
        let guard_start = Instant::now();
        #[cfg(any(test, feature = "mutants"))]
        let mut guard_iters: usize = 0;
        loop {
            #[cfg(any(test, feature = "mutants"))]
            {
                let prev = guard_iters;
                guard_iters += 1;
                assert!(guard_iters > prev);
                guard_loop(guard_start, guard_iters, 10_000, "spawn_reader_thread");
            }
            let n = unsafe {
                libc::read(
                    master_fd,
                    buffer.as_mut_ptr() as *mut libc::c_void,
                    buffer.len(),
                )
            };
            if n > 0 {
                let mut data = buffer.get(..n as usize).unwrap_or(&[]).to_vec();
                // Answer simple terminal capability queries so Codex doesn't hang waiting.
                respond_to_terminal_queries(&mut data, master_fd);
                if data.is_empty() {
                    continue;
                }
                if tx.send(data).is_err() {
                    break;
                }
                continue;
            }
            if n == 0 {
                break;
            }
            let err = io::Error::last_os_error();
            if should_retry_read_error(&err) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            log_debug(&format!("PTY read error: {err}"));
            break;
        }
    })
}

/// Continuously read from the PTY and forward raw chunks to the main thread.
pub(super) fn spawn_passthrough_reader_thread(
    master_fd: RawFd,
    tx: Sender<Vec<u8>>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut buffer = [0u8; 4096];
        #[cfg(any(test, feature = "mutants"))]
        let guard_start = Instant::now();
        #[cfg(any(test, feature = "mutants"))]
        let mut guard_iters: usize = 0;
        loop {
            #[cfg(any(test, feature = "mutants"))]
            {
                let prev = guard_iters;
                guard_iters += 1;
                assert!(guard_iters > prev);
                guard_loop(
                    guard_start,
                    guard_iters,
                    10_000,
                    "spawn_passthrough_reader_thread",
                );
            }
            let n = unsafe {
                libc::read(
                    master_fd,
                    buffer.as_mut_ptr() as *mut libc::c_void,
                    buffer.len(),
                )
            };
            if n > 0 {
                let mut data = buffer.get(..n as usize).unwrap_or(&[]).to_vec();
                respond_to_terminal_queries_passthrough(&mut data, master_fd);
                if data.is_empty() {
                    continue;
                }
                if tx.send(data).is_err() {
                    break;
                }
                continue;
            }
            if n == 0 {
                break;
            }
            let err = io::Error::last_os_error();
            if should_retry_read_error(&err) {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            log_debug(&format!("PTY read error: {err}"));
            break;
        }
    })
}

/// Write the entire buffer to the PTY master, retrying short writes.
pub(super) fn write_all(fd: RawFd, mut data: &[u8]) -> Result<()> {
    #[cfg(any(test, feature = "mutants"))]
    let guard_start = Instant::now();
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;
    while !data.is_empty() {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(guard_start, guard_iters, 10_000, "write_all");
        }
        let write_len = write_all_limit(data.len());
        let written = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, write_len) };
        if written < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted || err.kind() == ErrorKind::WouldBlock {
                thread::sleep(Duration::from_millis(1));
                continue;
            }
            return Err(anyhow!("write to PTY failed: {err}"));
        }
        if written == 0 {
            return Err(anyhow!("write to PTY returned 0"));
        }
        let written = written as usize;
        data = if written <= data.len() {
            &data[written..]
        } else {
            &[]
        };
    }
    Ok(())
}
