//! Minimal PTY wrapper used to host Codex in a real terminal so persistent sessions
//! can keep state (tools, environment) between prompts.

use crate::log_debug;
use anyhow::{anyhow, Context, Result};
use crossbeam_channel::{bounded, Receiver, Sender};
#[cfg(any(test, feature = "mutants"))]
use std::cell::Cell;
use std::ffi::CString;
use std::io::{self, ErrorKind};
use std::mem;
use std::os::unix::io::RawFd;
use std::ptr;
#[cfg(any(test, feature = "mutants"))]
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
#[cfg(any(test, feature = "mutants"))]
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

#[cfg(any(test, feature = "mutants"))]
thread_local! {
    static PTY_SEND_COUNT: Cell<usize> = const { Cell::new(0) };
    static PTY_READ_COUNT: Cell<usize> = const { Cell::new(0) };
}
#[cfg(any(test, feature = "mutants"))]
static READ_OUTPUT_GRACE_OVERRIDE_MS: AtomicU64 = AtomicU64::new(u64::MAX);
#[cfg(any(test, feature = "mutants"))]
static READ_OUTPUT_ELAPSED_OVERRIDE_MS: AtomicU64 = AtomicU64::new(u64::MAX);
#[cfg(any(test, feature = "mutants"))]
static WAIT_FOR_EXIT_ELAPSED_OVERRIDE_MS: AtomicU64 = AtomicU64::new(u64::MAX);
#[cfg(any(test, feature = "mutants"))]
static WAIT_FOR_EXIT_POLL_COUNT: AtomicUsize = AtomicUsize::new(0);
#[cfg(any(test, feature = "mutants"))]
static WAIT_FOR_EXIT_REAP_COUNT: AtomicUsize = AtomicUsize::new(0);
#[cfg(any(test, feature = "mutants"))]
static WAIT_FOR_EXIT_ERROR_COUNT: AtomicUsize = AtomicUsize::new(0);
#[cfg(any(test, feature = "mutants"))]
thread_local! {
    static RESPOND_OSC_START: Cell<usize> = const { Cell::new(usize::MAX) };
    static RESPOND_OSC_HITS: Cell<usize> = const { Cell::new(0) };
    static APPLY_OSC_START: Cell<usize> = const { Cell::new(usize::MAX) };
    static APPLY_OSC_HITS: Cell<usize> = const { Cell::new(0) };
    static APPLY_LINESTART_RECALC_COUNT: Cell<usize> = const { Cell::new(0) };
}
#[cfg(any(test, feature = "mutants"))]
thread_local! {
    static WRITE_ALL_LIMIT: Cell<usize> = const { Cell::new(usize::MAX) };
}
#[cfg(any(test, feature = "mutants"))]
static TERMINAL_SIZE_OVERRIDE: OnceLock<Mutex<Option<(bool, u16, u16)>>> = OnceLock::new();

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn reset_pty_session_counters() {
    PTY_SEND_COUNT.with(|count| count.set(0));
    PTY_READ_COUNT.with(|count| count.set(0));
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn pty_session_send_count() -> usize {
    PTY_SEND_COUNT.with(|count| count.get())
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn pty_session_read_count() -> usize {
    PTY_READ_COUNT.with(|count| count.get())
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn set_read_output_grace_override(ms: Option<u64>) {
    READ_OUTPUT_GRACE_OVERRIDE_MS.store(ms.unwrap_or(u64::MAX), Ordering::SeqCst);
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn set_read_output_elapsed_override(ms: Option<u64>) {
    READ_OUTPUT_ELAPSED_OVERRIDE_MS.store(ms.unwrap_or(u64::MAX), Ordering::SeqCst);
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn set_write_all_limit(limit: Option<usize>) {
    WRITE_ALL_LIMIT.with(|value| value.set(limit.unwrap_or(usize::MAX)));
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn set_terminal_size_override(value: Option<(bool, u16, u16)>) {
    let lock = TERMINAL_SIZE_OVERRIDE.get_or_init(|| Mutex::new(None));
    *lock.lock().unwrap() = value;
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn set_wait_for_exit_elapsed_override(ms: Option<u64>) {
    WAIT_FOR_EXIT_ELAPSED_OVERRIDE_MS.store(ms.unwrap_or(u64::MAX), Ordering::SeqCst);
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn reset_wait_for_exit_counters() {
    WAIT_FOR_EXIT_POLL_COUNT.store(0, Ordering::SeqCst);
    WAIT_FOR_EXIT_REAP_COUNT.store(0, Ordering::SeqCst);
    WAIT_FOR_EXIT_ERROR_COUNT.store(0, Ordering::SeqCst);
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn wait_for_exit_poll_count() -> usize {
    WAIT_FOR_EXIT_POLL_COUNT.load(Ordering::SeqCst)
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn wait_for_exit_reap_count() -> usize {
    WAIT_FOR_EXIT_REAP_COUNT.load(Ordering::SeqCst)
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn wait_for_exit_error_count() -> usize {
    WAIT_FOR_EXIT_ERROR_COUNT.load(Ordering::SeqCst)
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn reset_respond_osc_counters() {
    RESPOND_OSC_START.with(|val| val.set(usize::MAX));
    RESPOND_OSC_HITS.with(|val| val.set(0));
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn respond_osc_start() -> usize {
    RESPOND_OSC_START.with(|val| val.get())
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn respond_osc_hits() -> usize {
    RESPOND_OSC_HITS.with(|val| val.get())
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn reset_apply_osc_counters() {
    APPLY_OSC_START.with(|val| val.set(usize::MAX));
    APPLY_OSC_HITS.with(|val| val.set(0));
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn apply_osc_start() -> usize {
    APPLY_OSC_START.with(|val| val.get())
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn apply_osc_hits() -> usize {
    APPLY_OSC_HITS.with(|val| val.get())
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn reset_apply_linestart_recalc_count() {
    APPLY_LINESTART_RECALC_COUNT.with(|val| val.set(0));
}

#[cfg(any(test, feature = "mutants"))]
#[allow(dead_code)]
pub(crate) fn apply_linestart_recalc_count() -> usize {
    APPLY_LINESTART_RECALC_COUNT.with(|val| val.get())
}

#[cfg(any(test, feature = "mutants"))]
fn record_respond_osc_start(start: usize) {
    RESPOND_OSC_START.with(|val| val.set(start));
    RESPOND_OSC_HITS.with(|val| val.set(val.get().saturating_add(1)));
}

#[cfg(any(test, feature = "mutants"))]
fn record_apply_osc_start(start: usize) {
    APPLY_OSC_START.with(|val| val.set(start));
    APPLY_OSC_HITS.with(|val| val.set(val.get().saturating_add(1)));
}

#[cfg(any(test, feature = "mutants"))]
fn record_apply_linestart_recalc() {
    APPLY_LINESTART_RECALC_COUNT.with(|val| val.set(val.get().saturating_add(1)));
}

#[cfg(any(test, feature = "mutants"))]
fn guard_elapsed_exceeded(elapsed: Duration, iterations: usize, limit: usize) -> bool {
    elapsed > Duration::from_secs(2) || iterations > limit
}

#[cfg(any(test, feature = "mutants"))]
fn guard_loop(start: Instant, iterations: usize, limit: usize, label: &str) {
    if guard_elapsed_exceeded(start.elapsed(), iterations, limit) {
        panic!("{label} loop guard exceeded");
    }
}

fn read_output_elapsed(start: Instant) -> Duration {
    #[cfg(any(test, feature = "mutants"))]
    {
        let override_ms = READ_OUTPUT_ELAPSED_OVERRIDE_MS.load(Ordering::SeqCst);
        if override_ms != u64::MAX {
            return Duration::from_millis(override_ms);
        }
    }
    start.elapsed()
}

/// Uses PTY to run Codex in a proper terminal environment
pub struct PtyCodexSession {
    master_fd: RawFd,
    child_pid: i32,
    pub output_rx: Receiver<Vec<u8>>,
    _output_thread: thread::JoinHandle<()>,
}

impl PtyCodexSession {
    /// Start Codex under a pseudo-terminal so it behaves like an interactive shell.
    pub fn new(
        codex_cmd: &str,
        working_dir: &str,
        args: &[String],
        term_value: &str,
    ) -> Result<Self> {
        let cwd = CString::new(working_dir)
            .with_context(|| format!("working directory contains NUL byte: {working_dir}"))?;
        let term_value_cstr = CString::new(term_value).unwrap_or_else(|_| {
            CString::new("xterm-256color").expect("static TERM fallback should be valid")
        });
        let mut argv: Vec<CString> = Vec::with_capacity(args.len() + 1);
        argv.push(
            CString::new(codex_cmd)
                .with_context(|| format!("codex_cmd contains NUL byte: {codex_cmd}"))?,
        );
        for arg in args {
            argv.push(
                CString::new(arg.as_str())
                    .with_context(|| format!("codex arg contains NUL byte: {arg}"))?,
            );
        }

        unsafe {
            let (master_fd, child_pid) = spawn_codex_child(&argv, &cwd, &term_value_cstr)?;
            set_nonblocking(master_fd)?;

            let (tx, rx) = bounded(100);
            let output_thread = spawn_reader_thread(master_fd, tx);

            let session = Self {
                master_fd,
                child_pid,
                output_rx: rx,
                _output_thread: output_thread,
            };

            Ok(session)
        }
    }

    /// Write text to the PTY, automatically ensuring prompts end with a newline.
    pub fn send(&mut self, text: &str) -> Result<()> {
        #[cfg(any(test, feature = "mutants"))]
        PTY_SEND_COUNT.with(|count| count.set(count.get().saturating_add(1)));
        write_all(self.master_fd, text.as_bytes())?;
        if !text.ends_with('\n') {
            write_all(self.master_fd, b"\n")?;
        }
        Ok(())
    }

    /// Drain any waiting output without blocking.
    pub fn read_output(&self) -> Vec<Vec<u8>> {
        let mut output = Vec::new();
        while let Ok(line) = self.output_rx.try_recv() {
            output.push(line);
        }
        output
    }

    /// Block for a short window until output arrives or the timeout expires.
    pub fn read_output_timeout(&self, timeout: Duration) -> Vec<Vec<u8>> {
        #[cfg(any(test, feature = "mutants"))]
        PTY_READ_COUNT.with(|count| count.set(count.get().saturating_add(1)));
        let mut output = Vec::new();
        let start = Instant::now();
        let mut last_chunk: Option<Instant> = None;

        while read_output_elapsed(start) < timeout {
            match self.output_rx.recv_timeout(Duration::from_millis(100)) {
                Ok(line) => {
                    last_chunk = Some(Instant::now());
                    output.push(line);
                }
                Err(_) => {
                    if output.is_empty() {
                        break;
                    }
                    if let Some(last) = last_chunk {
                        #[cfg(any(test, feature = "mutants"))]
                        let elapsed = {
                            let override_ms = READ_OUTPUT_GRACE_OVERRIDE_MS.load(Ordering::SeqCst);
                            if override_ms != u64::MAX {
                                Duration::from_millis(override_ms)
                            } else {
                                last.elapsed()
                            }
                        };
                        #[cfg(not(any(test, feature = "mutants")))]
                        let elapsed = last.elapsed();
                        if elapsed < Duration::from_millis(300) {
                            continue;
                        }
                    }
                    break;
                }
            }
        }
        output
    }

    /// Check if the PTY session is responsive by verifying the process is alive.
    /// Note: Codex doesn't output anything until you send a prompt, so we can't
    /// require output for health check - just verify the process started.
    pub fn is_responsive(&mut self, _timeout: Duration) -> bool {
        // Drain any startup output (banner, prompts, etc.) without blocking
        let _ = self.read_output();

        if !self.is_alive() {
            log_debug("PTY health check: process not alive");
            return false;
        }

        log_debug("PTY health check: process alive, assuming responsive");
        true
    }

    /// Wait up to `timeout` for at least one output chunk, then drain any remaining bytes.
    pub fn wait_for_output(&self, timeout: Duration) -> Vec<Vec<u8>> {
        let mut output = Vec::new();
        if let Ok(chunk) = self.output_rx.recv_timeout(timeout) {
            output.push(chunk);
            output.extend(self.read_output());
        }
        output
    }

    /// Peek whether the child is still running (without reaping it).
    pub fn is_alive(&self) -> bool {
        unsafe {
            let mut status = 0;
            let ret = libc::waitpid(self.child_pid, &mut status, libc::WNOHANG);
            ret == 0 // 0 means still running
        }
    }
}

impl Drop for PtyCodexSession {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = self.send("exit\n") {
                log_debug(&format!("failed to send PTY exit command: {err:#}"));
            }
            if !wait_for_exit(self.child_pid, Duration::from_millis(500)) {
                if libc::kill(self.child_pid, libc::SIGTERM) != 0 {
                    log_debug(&format!(
                        "SIGTERM to Codex session failed: {}",
                        io::Error::last_os_error()
                    ));
                }
                if !wait_for_exit(self.child_pid, Duration::from_millis(500)) {
                    if libc::kill(self.child_pid, libc::SIGKILL) != 0 {
                        log_debug(&format!(
                            "SIGKILL to Codex session failed: {}",
                            io::Error::last_os_error()
                        ));
                    }
                    #[cfg(any(test, feature = "mutants"))]
                    {
                        let mut status = 0;
                        let _ = libc::waitpid(self.child_pid, &mut status, libc::WNOHANG);
                    }
                    #[cfg(not(any(test, feature = "mutants")))]
                    {
                        let mut status = 0;
                        let ret = libc::waitpid(self.child_pid, &mut status, 0);
                        if waitpid_failed(ret) {
                            log_debug(&format!(
                                "waitpid after SIGKILL failed: {}",
                                io::Error::last_os_error()
                            ));
                        }
                    }
                }
            }
            close_fd(self.master_fd);
        }
    }
}

/// PTY session that forwards raw output (ANSI intact) while answering terminal queries.
pub struct PtyOverlaySession {
    master_fd: RawFd,
    child_pid: i32,
    pub output_rx: Receiver<Vec<u8>>,
    _output_thread: thread::JoinHandle<()>,
}

impl PtyOverlaySession {
    /// Start Codex under a pseudo-terminal but keep output raw for overlay rendering.
    pub fn new(
        codex_cmd: &str,
        working_dir: &str,
        args: &[String],
        term_value: &str,
    ) -> Result<Self> {
        let cwd = CString::new(working_dir)
            .with_context(|| format!("working directory contains NUL byte: {working_dir}"))?;
        let term_value_cstr = CString::new(term_value).unwrap_or_else(|_| {
            CString::new("xterm-256color").expect("static TERM fallback should be valid")
        });
        let mut argv: Vec<CString> = Vec::with_capacity(args.len() + 1);
        argv.push(
            CString::new(codex_cmd)
                .with_context(|| format!("codex_cmd contains NUL byte: {codex_cmd}"))?,
        );
        for arg in args {
            argv.push(
                CString::new(arg.as_str())
                    .with_context(|| format!("codex arg contains NUL byte: {arg}"))?,
            );
        }

        unsafe {
            let (master_fd, child_pid) = spawn_codex_child(&argv, &cwd, &term_value_cstr)?;
            set_nonblocking(master_fd)?;

            let (tx, rx) = bounded(100);
            let output_thread = spawn_passthrough_reader_thread(master_fd, tx);

            Ok(Self {
                master_fd,
                child_pid,
                output_rx: rx,
                _output_thread: output_thread,
            })
        }
    }

    /// Write raw bytes to the PTY master.
    pub fn send_bytes(&mut self, bytes: &[u8]) -> Result<()> {
        write_all(self.master_fd, bytes)
    }

    /// Write text to the PTY master.
    pub fn send_text(&mut self, text: &str) -> Result<()> {
        write_all(self.master_fd, text.as_bytes())
    }

    /// Write text to the PTY master and ensure it ends with a newline.
    pub fn send_text_with_newline(&mut self, text: &str) -> Result<()> {
        write_all(self.master_fd, text.as_bytes())?;
        if !text.ends_with('\n') {
            write_all(self.master_fd, b"\n")?;
        }
        Ok(())
    }

    /// Update the PTY window size and notify the child.
    pub fn set_winsize(&self, rows: u16, cols: u16) -> Result<()> {
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        ws.ws_row = rows.max(1);
        ws.ws_col = cols.max(1);
        ws.ws_xpixel = 0;
        ws.ws_ypixel = 0;
        let result = unsafe { libc::ioctl(self.master_fd, libc::TIOCSWINSZ, &ws) };
        if result != 0 {
            return Err(errno_error("ioctl(TIOCSWINSZ) failed"));
        }
        let _ = unsafe { libc::kill(self.child_pid, libc::SIGWINCH) };
        Ok(())
    }

    /// Peek whether the child is still running (without reaping it).
    pub fn is_alive(&self) -> bool {
        unsafe {
            let mut status = 0;
            let ret = libc::waitpid(self.child_pid, &mut status, libc::WNOHANG);
            ret == 0 // 0 means still running
        }
    }
}

impl Drop for PtyOverlaySession {
    fn drop(&mut self) {
        unsafe {
            if let Err(err) = self.send_text_with_newline("exit") {
                log_debug(&format!("failed to send PTY exit command: {err:#}"));
            }
            if !wait_for_exit(self.child_pid, Duration::from_millis(500)) {
                if libc::kill(self.child_pid, libc::SIGTERM) != 0 {
                    log_debug(&format!(
                        "SIGTERM to Codex session failed: {}",
                        io::Error::last_os_error()
                    ));
                }
                if !wait_for_exit(self.child_pid, Duration::from_millis(500)) {
                    if libc::kill(self.child_pid, libc::SIGKILL) != 0 {
                        log_debug(&format!(
                            "SIGKILL to Codex session failed: {}",
                            io::Error::last_os_error()
                        ));
                    }
                    #[cfg(any(test, feature = "mutants"))]
                    {
                        let mut status = 0;
                        let _ = libc::waitpid(self.child_pid, &mut status, libc::WNOHANG);
                    }
                    #[cfg(not(any(test, feature = "mutants")))]
                    {
                        let mut status = 0;
                        let ret = libc::waitpid(self.child_pid, &mut status, 0);
                        if waitpid_failed(ret) {
                            log_debug(&format!(
                                "waitpid after SIGKILL failed: {}",
                                io::Error::last_os_error()
                            ));
                        }
                    }
                }
            }
            close_fd(self.master_fd);
        }
    }
}

/// Fork and exec the Codex binary under a newly allocated PTY pair.
unsafe fn spawn_codex_child(
    argv: &[CString],
    working_dir: &CString,
    term_value: &CString,
) -> Result<(RawFd, i32)> {
    let mut master_fd: RawFd = -1;
    let mut slave_fd: RawFd = -1;

    // Set a proper terminal size - codex checks this for terminal detection
    let mut winsize: libc::winsize = mem::zeroed();
    winsize.ws_row = 24;
    winsize.ws_col = 80;
    winsize.ws_xpixel = 0;
    winsize.ws_ypixel = 0;

    #[allow(clippy::unnecessary_mut_passed)]
    if libc::openpty(
        &mut master_fd,
        &mut slave_fd,
        ptr::null_mut(),
        ptr::null_mut(),
        &mut winsize,
    ) != 0
    {
        return Err(errno_error("openpty failed"));
    }

    let pid = libc::fork();
    if pid < 0 {
        close_fd(master_fd);
        close_fd(slave_fd);
        return Err(errno_error("fork failed"));
    }

    if pid == 0 {
        child_exec(slave_fd, argv, working_dir, term_value);
    }

    close_fd(slave_fd);
    Ok((master_fd, pid))
}

/// Child-side setup: hook the PTY to stdin/stdout/stderr and exec Codex.
unsafe fn child_exec(
    slave_fd: RawFd,
    argv: &[CString],
    working_dir: &CString,
    term_value: &CString,
) -> ! {
    let fail = || -> ! {
        libc::_exit(1);
    };

    if libc::setsid() == -1 {
        fail();
    }
    if libc::ioctl(slave_fd, libc::TIOCSCTTY as libc::c_ulong, 0) == -1 {
        fail();
    }
    if libc::dup2(slave_fd, libc::STDIN_FILENO) < 0
        || libc::dup2(slave_fd, libc::STDOUT_FILENO) < 0
        || libc::dup2(slave_fd, libc::STDERR_FILENO) < 0
    {
        fail();
    }
    close_fd(slave_fd);

    if libc::chdir(working_dir.as_ptr()) != 0 {
        fail();
    }

    let term_key = CString::new("TERM").expect("TERM constant is valid");
    if libc::setenv(term_key.as_ptr(), term_value.as_ptr(), 1) != 0 {
        fail();
    }

    let mut argv_ptrs: Vec<*const libc::c_char> = argv.iter().map(|s| s.as_ptr()).collect();
    argv_ptrs.push(ptr::null());

    libc::execvp(argv_ptrs[0], argv_ptrs.as_ptr());
    fail();
}

/// Configure the PTY master for non-blocking reads.
unsafe fn set_nonblocking(fd: RawFd) -> Result<()> {
    let flags = libc::fcntl(fd, libc::F_GETFL, 0);
    if flags < 0 {
        return Err(errno_error("fcntl(F_GETFL) failed"));
    }
    if libc::fcntl(fd, libc::F_SETFL, flags | libc::O_NONBLOCK) < 0 {
        return Err(errno_error("fcntl(F_SETFL) failed"));
    }
    Ok(())
}

fn should_retry_read_error(err: &io::Error) -> bool {
    err.kind() == ErrorKind::Interrupted || err.kind() == ErrorKind::WouldBlock
}

/// Continuously read from the PTY and forward chunks to the main thread.
fn spawn_reader_thread(master_fd: RawFd, tx: Sender<Vec<u8>>) -> thread::JoinHandle<()> {
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
fn spawn_passthrough_reader_thread(
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

/// Helper that formats OS errors with additional context.
fn errno_error(context: &str) -> anyhow::Error {
    anyhow!("{context}: {}", io::Error::last_os_error())
}

/// Close a file descriptor while ignoring errors.
unsafe fn close_fd(fd: RawFd) {
    if fd >= 0 {
        let _ = libc::close(fd);
    }
}

/// Write the entire buffer to the PTY master, retrying short writes.
fn write_all(fd: RawFd, mut data: &[u8]) -> Result<()> {
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
        #[cfg(any(test, feature = "mutants"))]
        let write_len = WRITE_ALL_LIMIT.with(|limit| data.len().min(limit.get()));
        #[cfg(not(any(test, feature = "mutants")))]
        let write_len = data.len();
        let written = unsafe { libc::write(fd, data.as_ptr() as *const libc::c_void, write_len) };
        if written < 0 {
            let err = io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
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

fn wait_for_exit_elapsed(start: Instant) -> Duration {
    #[cfg(any(test, feature = "mutants"))]
    {
        let override_ms = WAIT_FOR_EXIT_ELAPSED_OVERRIDE_MS.load(Ordering::SeqCst);
        if override_ms != u64::MAX {
            return Duration::from_millis(override_ms);
        }
    }
    start.elapsed()
}

#[cfg_attr(any(test, feature = "mutants"), allow(dead_code))]
fn waitpid_failed(ret: i32) -> bool {
    ret < 0
}

/// Wait for the child process to terminate, but bail out after a short timeout.
fn wait_for_exit(child_pid: i32, timeout: Duration) -> bool {
    let start = Instant::now();
    let mut status = 0;
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;
    while wait_for_exit_elapsed(start) < timeout {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(start, guard_iters, 10_000, "wait_for_exit");
        }
        #[cfg(any(test, feature = "mutants"))]
        WAIT_FOR_EXIT_POLL_COUNT.fetch_add(1, Ordering::SeqCst);
        let result = unsafe { libc::waitpid(child_pid, &mut status, libc::WNOHANG) };
        if result > 0 {
            #[cfg(any(test, feature = "mutants"))]
            WAIT_FOR_EXIT_REAP_COUNT.fetch_add(1, Ordering::SeqCst);
            return true;
        }
        if result < 0 {
            #[cfg(any(test, feature = "mutants"))]
            WAIT_FOR_EXIT_ERROR_COUNT.fetch_add(1, Ordering::SeqCst);
            log_debug(&format!(
                "waitpid({}) failed: {}",
                child_pid,
                io::Error::last_os_error()
            ));
            return true;
        }
        thread::sleep(Duration::from_millis(50));
    }
    false
}

/// Codex occasionally probes terminal capabilities; strip those control sequences
/// and apply inline edits (CR/BS) before rendering so the UI only sees printable text.
fn respond_to_terminal_queries(buffer: &mut Vec<u8>, master_fd: RawFd) {
    let (rows, cols) = current_terminal_size(master_fd);
    let mut idx = 0;
    #[cfg(any(test, feature = "mutants"))]
    let guard_start = Instant::now();
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;
    while idx < buffer.len() {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(
                guard_start,
                guard_iters,
                buffer.len().saturating_mul(4).max(64),
                "respond_to_terminal_queries",
            );
        }
        if buffer[idx] != 0x1B {
            idx += 1;
            continue;
        }
        if idx + 1 >= buffer.len() {
            break;
        }
        match buffer[idx + 1] {
            b'[' => {
                match find_csi_sequence(buffer, idx + 2) {
                    Some((params_end, final_byte)) => {
                        let seq_end = params_end + 1;
                        let params_start = idx + 2;
                        let params: Vec<u8> =
                            buffer.get(params_start..params_end).unwrap_or(&[]).to_vec();
                        if should_strip_without_reply(&params, final_byte) {
                            buffer.drain(idx..seq_end);
                            continue;
                        }
                        if let Some(reply) = csi_reply(&params, final_byte, rows, cols) {
                            buffer.drain(idx..seq_end);
                            if let Err(err) = write_all(master_fd, &reply) {
                                log_debug(&format!(
                                    "Failed to answer terminal query (CSI {}{}): {err:#}",
                                    String::from_utf8_lossy(&params),
                                    final_byte as char
                                ));
                            }
                            continue;
                        }
                        idx = seq_end;
                        continue;
                    }
                    None => break, // Incomplete CSI sequence; wait for more data
                }
            }
            b']' => {
                #[cfg(any(test, feature = "mutants"))]
                record_respond_osc_start(idx + 2);
                if let Some(end) = find_osc_terminator(buffer, idx + 2) {
                    buffer.drain(idx..end);
                    continue;
                } else {
                    // Unterminated OSC; drop the rest to avoid leaking into output
                    buffer.drain(idx..);
                    break;
                }
            }
            _ => {}
        }
        idx += 1;
    }

    apply_control_edits(buffer);
}

/// Answer terminal queries but keep all other ANSI sequences intact.
fn respond_to_terminal_queries_passthrough(buffer: &mut Vec<u8>, master_fd: RawFd) {
    let (rows, cols) = current_terminal_size(master_fd);
    let mut idx = 0;
    #[cfg(any(test, feature = "mutants"))]
    let guard_start = Instant::now();
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;
    while idx < buffer.len() {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(
                guard_start,
                guard_iters,
                buffer.len().saturating_mul(4).max(64),
                "respond_to_terminal_queries_passthrough",
            );
        }
        if buffer[idx] != 0x1B {
            idx += 1;
            continue;
        }
        if idx + 1 >= buffer.len() {
            break;
        }
        if buffer[idx + 1] == b'[' {
            match find_csi_sequence(buffer, idx + 2) {
                Some((params_end, final_byte)) => {
                    let seq_end = params_end + 1;
                    let params_start = idx + 2;
                    let params: Vec<u8> =
                        buffer.get(params_start..params_end).unwrap_or(&[]).to_vec();
                    if let Some(reply) = csi_reply(&params, final_byte, rows, cols) {
                        buffer.drain(idx..seq_end);
                        if let Err(err) = write_all(master_fd, &reply) {
                            log_debug(&format!(
                                "Failed to answer terminal query (CSI {}{}): {err:#}",
                                String::from_utf8_lossy(&params),
                                final_byte as char
                            ));
                        }
                        continue;
                    }
                    idx = seq_end;
                    continue;
                }
                None => break,
            }
        }
        idx += 1;
    }
}

fn should_strip_without_reply(params: &[u8], final_byte: u8) -> bool {
    // Strip keyboard protocol queries (Kitty keyboard protocol)
    if final_byte == b'u' && (params.starts_with(b"?") || params.starts_with(b">")) {
        return true;
    }
    // Strip mode set/reset sequences (h/l) - these don't need responses
    // Examples: ?2004h (bracketed paste), ?1004h (focus events), ?25h (cursor visible)
    if (final_byte == b'h' || final_byte == b'l') && params.starts_with(b"?") {
        return true;
    }
    // Strip cursor movement and styling sequences
    if matches!(
        final_byte,
        b'A' | b'B' | b'C' | b'D' | b'H' | b'J' | b'K' | b'm' | b'r'
    ) {
        return true;
    }
    false
}

fn csi_reply(params: &[u8], final_b: u8, rows: u16, cols: u16) -> Option<Vec<u8>> {
    // Normalize params: drop leading '?', '>' and spaces.
    let p: Vec<u8> = params
        .iter()
        .copied()
        .filter(|b| *b != b' ')
        .skip_while(|b| *b == b'?' || *b == b'>')
        .collect();

    match final_b {
        // DSR: status report → ESC[0n
        b'n' if p == b"5" => Some(b"\x1b[0n".to_vec()),

        // DSR: cursor position request → ESC[row;colR
        b'n' if p == b"6" => {
            let (r, c) = (rows.max(1), cols.max(1));
            Some(format!("\x1b[{r};{c}R").into_bytes())
        }

        // DA: primary device attributes → safe VT220-ish reply
        b'c' => Some(b"\x1b[?1;2c".to_vec()),

        _ => None,
    }
}

fn current_terminal_size(master_fd: RawFd) -> (u16, u16) {
    #[cfg(any(test, feature = "mutants"))]
    if let Some((ok, row, col)) = *TERMINAL_SIZE_OVERRIDE
        .get_or_init(|| Mutex::new(None))
        .lock()
        .unwrap()
    {
        return if ok && row > 0 && col > 0 {
            (row, col)
        } else {
            (24, 80)
        };
    }
    let mut ws: libc::winsize = unsafe { mem::zeroed() };
    unsafe {
        if libc::ioctl(master_fd, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_row > 0 && ws.ws_col > 0
        {
            (ws.ws_row, ws.ws_col)
        } else {
            (24, 80)
        }
    }
}

fn find_csi_sequence(bytes: &[u8], start: usize) -> Option<(usize, u8)> {
    let mut idx = start;
    #[cfg(any(test, feature = "mutants"))]
    let guard_start = Instant::now();
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;
    while idx < bytes.len() {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(
                guard_start,
                guard_iters,
                bytes.len().saturating_add(16),
                "find_csi_sequence",
            );
        }
        let byte = bytes[idx];
        if (0x40..=0x7E).contains(&byte) {
            return Some((idx, byte));
        }
        idx += 1;
    }
    None
}

fn find_osc_terminator(bytes: &[u8], mut cursor: usize) -> Option<usize> {
    #[cfg(any(test, feature = "mutants"))]
    let guard_start = Instant::now();
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;
    while cursor < bytes.len() {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(
                guard_start,
                guard_iters,
                bytes.len().saturating_add(16),
                "find_osc_terminator",
            );
        }
        match bytes[cursor] {
            0x07 => return Some(cursor + 1),
            0x1B if cursor + 1 < bytes.len() && bytes[cursor + 1] == b'\\' => {
                return Some(cursor + 2)
            }
            _ => cursor += 1,
        }
    }
    None
}

fn apply_control_edits(buffer: &mut Vec<u8>) {
    let mut output = Vec::with_capacity(buffer.len());
    let mut line_start = 0usize;
    let mut idx = 0;
    #[cfg(any(test, feature = "mutants"))]
    let guard_start = Instant::now();
    #[cfg(any(test, feature = "mutants"))]
    let mut guard_iters: usize = 0;

    while idx < buffer.len() {
        #[cfg(any(test, feature = "mutants"))]
        {
            let prev = guard_iters;
            guard_iters += 1;
            assert!(guard_iters > prev);
            guard_loop(
                guard_start,
                guard_iters,
                buffer.len().saturating_mul(4).max(64),
                "apply_control_edits",
            );
        }
        match buffer[idx] {
            b'\r' => {
                output.truncate(line_start);
                idx += 1;
            }
            b'\n' => {
                output.push(b'\n');
                idx += 1;
                line_start = output.len();
            }
            b'\x08' => {
                pop_last_codepoint(&mut output);
                idx += 1;
                if line_start > output.len() {
                    #[cfg(any(test, feature = "mutants"))]
                    record_apply_linestart_recalc();
                    line_start = current_line_start(&output);
                }
            }
            0x1B => {
                if idx + 1 < buffer.len() && buffer[idx + 1] == b']' {
                    #[cfg(any(test, feature = "mutants"))]
                    record_apply_osc_start(idx + 2);
                    if let Some(end) = find_osc_terminator(buffer, idx + 2) {
                        idx = end;
                        continue;
                    } else {
                        break;
                    }
                }
                output.push(buffer[idx]);
                idx += 1;
            }
            byte => {
                output.push(byte);
                idx += 1;
            }
        }
    }

    buffer.clear();
    buffer.extend_from_slice(&output);
}

fn pop_last_codepoint(buf: &mut Vec<u8>) {
    if buf.is_empty() {
        return;
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
        return;
    }
    while let Some(byte) = buf.pop() {
        if (byte & 0b1100_0000) != 0b1000_0000 {
            break;
        }
    }
}

fn current_line_start(buf: &[u8]) -> usize {
    buf.iter()
        .rposition(|&b| b == b'\n')
        .map(|pos| pos + 1)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::mem::ManuallyDrop;
    use std::os::unix::io::RawFd;
    use std::sync::{Mutex, OnceLock};

    fn pipe_pair() -> (RawFd, RawFd) {
        let mut fds = [0; 2];
        let result = unsafe { libc::pipe(fds.as_mut_ptr()) };
        assert_eq!(
            result,
            0,
            "pipe() failed with errno {}",
            io::Error::last_os_error()
        );
        (fds[0], fds[1])
    }

    fn close_fd_pair(read_fd: RawFd, write_fd: RawFd) {
        unsafe {
            libc::close(read_fd);
            libc::close(write_fd);
        }
    }

    fn read_all(fd: RawFd) -> Vec<u8> {
        let mut out = Vec::new();
        let mut buf = [0u8; 64];
        loop {
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
            if n <= 0 {
                break;
            }
            out.extend_from_slice(&buf[..n as usize]);
        }
        out
    }

    fn set_nonblocking_fd(fd: RawFd) {
        unsafe { set_nonblocking(fd).unwrap() };
    }

    fn open_pty_pair() -> (RawFd, RawFd) {
        let mut master = -1;
        let mut slave = -1;
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        let result = unsafe {
            libc::openpty(
                &mut master,
                &mut slave,
                ptr::null_mut(),
                ptr::null_mut(),
                &mut ws,
            )
        };
        assert_eq!(
            result,
            0,
            "openpty() failed with errno {}",
            io::Error::last_os_error()
        );
        (master, slave)
    }

    fn log_lock() -> &'static Mutex<()> {
        static LOG_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        LOG_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn read_output_timeout_lock() -> &'static Mutex<()> {
        static READ_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        READ_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn terminal_size_override_lock() -> &'static Mutex<()> {
        static SIZE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        SIZE_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn wait_for_exit_lock() -> &'static Mutex<()> {
        static EXIT_LOCK: OnceLock<Mutex<()>> = OnceLock::new();
        EXIT_LOCK.get_or_init(|| Mutex::new(()))
    }

    fn capture_new_log<F: FnOnce()>(f: F) -> String {
        let _guard = log_lock().lock().unwrap();
        let log_path = crate::log_file_path();
        let before = fs::read(&log_path).unwrap_or_default();
        f();
        let after = fs::read(&log_path).unwrap_or_default();
        let new_bytes = after.get(before.len()..).unwrap_or(&[]);
        String::from_utf8_lossy(new_bytes).to_string()
    }

    #[test]
    fn guard_loop_enforces_iteration_limit() {
        let start = Instant::now();
        let ok = std::panic::catch_unwind(|| guard_loop(start, 10, 10, "guard"));
        assert!(ok.is_ok());
        let err = std::panic::catch_unwind(|| guard_loop(start, 11, 10, "guard"));
        assert!(err.is_err());
    }

    #[test]
    fn guard_loop_enforces_elapsed_limit() {
        let start = Instant::now() - Duration::from_secs(3);
        let err = std::panic::catch_unwind(|| guard_loop(start, 0, 10, "guard"));
        assert!(err.is_err());
    }

    #[test]
    fn guard_elapsed_exceeded_allows_exact_limit() {
        assert!(!guard_elapsed_exceeded(Duration::from_secs(2), 0, 10));
        assert!(guard_elapsed_exceeded(Duration::from_secs(3), 0, 10));
    }

    #[test]
    fn respond_to_terminal_queries_handles_multiple_sequences() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"start\x1b[6nmid\x1b[cend".to_vec();

        respond_to_terminal_queries(&mut buffer, write_fd);

        assert_eq!(buffer, b"startmidend".to_vec());
        let reply = read_reply(read_fd);
        assert!(
            reply_contains(&reply, b"\x1b[24;80R"),
            "reply missing cursor report: {reply:?}"
        );
        assert!(
            reply_contains(&reply, b"\x1b[?1;2c"),
            "reply missing device attributes: {reply:?}"
        );
        close_fd_pair(read_fd, write_fd);
    }

    fn read_reply(fd: RawFd) -> Vec<u8> {
        set_nonblocking_fd(fd);
        let start = Instant::now();
        let mut buf = [0u8; 64];
        loop {
            let n = unsafe { libc::read(fd, buf.as_mut_ptr() as *mut _, buf.len()) };
            if n > 0 {
                return buf[..n as usize].to_vec();
            }
            if n == 0 {
                return Vec::new();
            }
            let err = io::Error::last_os_error();
            if err.kind() == ErrorKind::Interrupted {
                continue;
            }
            if err.kind() == ErrorKind::WouldBlock {
                if start.elapsed() > Duration::from_millis(200) {
                    return Vec::new();
                }
                thread::sleep(Duration::from_millis(5));
                continue;
            }
            return Vec::new();
        }
    }

    #[test]
    fn respond_to_terminal_queries_replies_to_cursor_request() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"Hello\x1b[6nWorld".to_vec();

        respond_to_terminal_queries(&mut buffer, write_fd);

        assert_eq!(buffer, b"HelloWorld".to_vec());
        let reply = read_reply(read_fd);
        assert!(reply_contains(&reply, b"\x1b[24;80R"));
        close_fd_pair(read_fd, write_fd);
    }

    #[test]
    fn respond_to_terminal_queries_replies_to_status_request() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"\x1b[5nOK".to_vec();

        respond_to_terminal_queries(&mut buffer, write_fd);

        assert_eq!(buffer, b"OK".to_vec());
        let reply = read_reply(read_fd);
        assert_eq!(reply, b"\x1b[0n");
        close_fd_pair(read_fd, write_fd);
    }

    #[test]
    fn respond_to_terminal_queries_replies_to_extended_device_attr() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"\x1b[?1;0c".to_vec();

        respond_to_terminal_queries(&mut buffer, write_fd);

        assert!(buffer.is_empty());
        let reply = read_reply(read_fd);
        assert_eq!(reply, b"\x1b[?1;2c");
        close_fd_pair(read_fd, write_fd);
    }

    #[test]
    fn respond_to_terminal_queries_passthrough_keeps_ansi() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"\x1b[31mred\x1b[0m\x1b[6n".to_vec();

        respond_to_terminal_queries_passthrough(&mut buffer, write_fd);

        assert_eq!(buffer, b"\x1b[31mred\x1b[0m".to_vec());
        let reply = read_reply(read_fd);
        assert!(reply_contains(&reply, b"\x1b[24;80R"));
        close_fd_pair(read_fd, write_fd);
    }

    #[test]
    fn apply_control_edits_handles_cr_and_backspace() {
        let mut data = b"foo\rbar\x08z\nnext\x08!".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"baz\nnex!");
    }

    #[test]
    fn write_all_writes_bytes() {
        let (read_fd, write_fd) = pipe_pair();
        write_all(write_fd, b"hello").unwrap();
        unsafe { libc::close(write_fd) };
        let output = read_all(read_fd);
        assert_eq!(output, b"hello");
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn write_all_handles_short_writes() {
        struct LimitReset;
        impl Drop for LimitReset {
            fn drop(&mut self) {
                set_write_all_limit(None);
            }
        }

        set_write_all_limit(Some(1));
        let _reset = LimitReset;

        let (read_fd, write_fd) = pipe_pair();
        write_all(write_fd, b"hello").unwrap();
        unsafe { libc::close(write_fd) };
        let output = read_all(read_fd);
        assert_eq!(output, b"hello");
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn write_all_reports_zero_write() {
        struct LimitReset;
        impl Drop for LimitReset {
            fn drop(&mut self) {
                set_write_all_limit(None);
            }
        }

        set_write_all_limit(Some(0));
        let _reset = LimitReset;

        let (read_fd, write_fd) = pipe_pair();
        let err = write_all(write_fd, b"hi").unwrap_err();
        unsafe {
            libc::close(write_fd);
            libc::close(read_fd);
        }
        assert!(err.to_string().contains("returned 0"));
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_returns_true_for_exited_child() {
        let _guard = wait_for_exit_lock().lock().unwrap();
        reset_wait_for_exit_counters();
        set_wait_for_exit_elapsed_override(None);
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        assert!(wait_for_exit(pid, Duration::from_secs(1)));
        let _ = child.wait();
        assert_eq!(wait_for_exit_error_count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_returns_false_when_timeout() {
        let _guard = wait_for_exit_lock().lock().unwrap();
        reset_wait_for_exit_counters();
        set_wait_for_exit_elapsed_override(None);
        let mut child = std::process::Command::new("sleep")
            .arg("1")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        assert!(!wait_for_exit(pid, Duration::from_millis(10)));
        let _ = child.kill();
        let _ = child.wait();
        assert_eq!(wait_for_exit_reap_count(), 0);
        assert_eq!(wait_for_exit_error_count(), 0);
        assert!(wait_for_exit_poll_count() > 0);
    }

    #[test]
    fn find_csi_sequence_finds_final_byte() {
        let bytes = b"1;2R";
        assert_eq!(find_csi_sequence(bytes, 0), Some((3, b'R')));
        assert_eq!(find_csi_sequence(bytes, 4), None);
    }

    #[test]
    fn find_osc_terminator_handles_bel_and_st() {
        let bel = b"0;title\x07rest";
        assert_eq!(find_osc_terminator(bel, 0), Some(b"0;title\x07".len()));
        let st = b"0;title\x1b\\rest";
        assert_eq!(find_osc_terminator(st, 0), Some(b"0;title\x1b\\".len()));
    }

    #[test]
    fn should_strip_without_reply_matches_expected_sequences() {
        assert!(should_strip_without_reply(b"?2004", b'h'));
        assert!(should_strip_without_reply(b">1", b'u'));
        assert!(should_strip_without_reply(b"", b'm'));
        assert!(!should_strip_without_reply(b"6", b'n'));
    }

    #[test]
    fn csi_reply_returns_expected_responses() {
        assert_eq!(csi_reply(b"5", b'n', 24, 80), Some(b"\x1b[0n".to_vec()));
        assert_eq!(csi_reply(b"6", b'n', 2, 3), Some(b"\x1b[2;3R".to_vec()));
        assert_eq!(csi_reply(b"", b'c', 24, 80), Some(b"\x1b[?1;2c".to_vec()));
        assert_eq!(csi_reply(b"1", b'n', 24, 80), None);
    }

    #[test]
    fn current_terminal_size_falls_back_on_invalid_fd() {
        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(None);
        assert_eq!(current_terminal_size(-1), (24, 80));
    }

    #[test]
    fn pty_codex_session_send_appends_newline() {
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let mut session = ManuallyDrop::new(PtyCodexSession {
            master_fd: write_fd,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        session.send("hello").unwrap();
        unsafe { libc::close(write_fd) };
        let output = read_all(read_fd);
        assert_eq!(output, b"hello\n");
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn pty_overlay_session_send_text_with_newline_appends() {
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let mut session = ManuallyDrop::new(PtyOverlaySession {
            master_fd: write_fd,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        session.send_text_with_newline("overlay").unwrap();
        unsafe { libc::close(write_fd) };
        let output = read_all(read_fd);
        assert_eq!(output, b"overlay\n");
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn spawn_reader_thread_forwards_output() {
        let (read_fd, write_fd) = pipe_pair();
        let (tx, rx) = bounded(2);
        let handle = spawn_reader_thread(read_fd, tx);
        unsafe {
            libc::write(write_fd, b"hello".as_ptr() as *const libc::c_void, 5);
            libc::close(write_fd);
        }
        let data = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(data, b"hello");
        handle.join().unwrap();
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn spawn_passthrough_reader_thread_forwards_output() {
        let (read_fd, write_fd) = pipe_pair();
        let (tx, rx) = bounded(2);
        let handle = spawn_passthrough_reader_thread(read_fd, tx);
        unsafe {
            libc::write(write_fd, b"hello".as_ptr() as *const libc::c_void, 5);
            libc::close(write_fd);
        }
        let data = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(data, b"hello");
        handle.join().unwrap();
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn should_retry_read_error_reports_expected_kinds() {
        assert!(should_retry_read_error(&io::Error::from(
            ErrorKind::Interrupted
        )));
        assert!(should_retry_read_error(&io::Error::from(
            ErrorKind::WouldBlock
        )));
        assert!(!should_retry_read_error(&io::Error::from(ErrorKind::Other)));
    }

    #[test]
    fn pty_session_counters_track_send_and_read() {
        reset_pty_session_counters();
        assert_eq!(pty_session_send_count(), 0);
        assert_eq!(pty_session_read_count(), 0);

        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let mut session = ManuallyDrop::new(PtyCodexSession {
            master_fd: write_fd,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        session.send("ping").unwrap();
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
        assert_eq!(pty_session_send_count(), 1);

        let _guard = read_output_timeout_lock().lock().unwrap();
        let (_tx2, rx2) = bounded(1);
        let handle2 = thread::spawn(|| {});
        let session2 = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx2,
            _output_thread: handle2,
        });
        let _ = session2.read_output_timeout(Duration::from_millis(0));
        assert_eq!(pty_session_read_count(), 1);

        reset_pty_session_counters();
        assert_eq!(pty_session_send_count(), 0);
        assert_eq!(pty_session_read_count(), 0);
    }

    #[test]
    fn current_terminal_size_uses_override_values() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_terminal_size_override(None);
            }
        }

        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(Some((true, 10, 20)));
        let _reset = OverrideReset;
        assert_eq!(current_terminal_size(-1), (10, 20));
    }

    #[test]
    fn current_terminal_size_falls_back_when_override_disabled() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_terminal_size_override(None);
            }
        }

        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(Some((false, 10, 20)));
        let _reset = OverrideReset;
        assert_eq!(current_terminal_size(-1), (24, 80));
    }

    #[test]
    fn current_terminal_size_falls_back_when_ioctl_dimensions_zero() {
        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(None);
        let (master, slave) = open_pty_pair();
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        ws.ws_row = 0;
        ws.ws_col = 0;
        unsafe {
            assert_eq!(libc::ioctl(master, libc::TIOCSWINSZ, &ws), 0);
        }
        assert_eq!(current_terminal_size(master), (24, 80));
        unsafe {
            libc::close(master);
            libc::close(slave);
        }
    }

    #[test]
    fn current_terminal_size_falls_back_when_ioctl_row_zero() {
        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(None);
        let (master, slave) = open_pty_pair();
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        ws.ws_row = 0;
        ws.ws_col = 80;
        unsafe {
            assert_eq!(libc::ioctl(master, libc::TIOCSWINSZ, &ws), 0);
        }
        assert_eq!(current_terminal_size(master), (24, 80));
        unsafe {
            libc::close(master);
            libc::close(slave);
        }
    }

    #[test]
    fn current_terminal_size_falls_back_when_ioctl_col_zero() {
        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(None);
        let (master, slave) = open_pty_pair();
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        ws.ws_row = 40;
        ws.ws_col = 0;
        unsafe {
            assert_eq!(libc::ioctl(master, libc::TIOCSWINSZ, &ws), 0);
        }
        assert_eq!(current_terminal_size(master), (24, 80));
        unsafe {
            libc::close(master);
            libc::close(slave);
        }
    }

    #[test]
    fn respond_to_terminal_queries_handles_trailing_escape() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"end\x1b".to_vec();
        respond_to_terminal_queries(&mut buffer, write_fd);
        assert_eq!(buffer, b"end\x1b".to_vec());
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_to_terminal_queries_handles_unknown_escape() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"pre\x1bXpost".to_vec();
        respond_to_terminal_queries(&mut buffer, write_fd);
        assert_eq!(buffer, b"pre\x1bXpost".to_vec());
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_to_terminal_queries_osc_preserves_trailing_bytes() {
        reset_respond_osc_counters();
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"hello!\x1b]\x07X".to_vec();
        respond_to_terminal_queries(&mut buffer, write_fd);
        assert_eq!(buffer, b"hello!X".to_vec());
        assert_eq!(respond_osc_hits(), 1);
        assert_eq!(respond_osc_start(), 8);
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_to_terminal_queries_osc_with_prior_bel() {
        reset_respond_osc_counters();
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"A\x07B\x1b]0\x07Z".to_vec();
        respond_to_terminal_queries(&mut buffer, write_fd);
        assert_eq!(buffer, b"A\x07BZ".to_vec());
        assert_eq!(respond_osc_hits(), 1);
        assert_eq!(respond_osc_start(), 5);
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_osc_counters_reset_clears_hits() {
        reset_respond_osc_counters();
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"\x1b]0\x07".to_vec();
        respond_to_terminal_queries(&mut buffer, write_fd);
        assert_eq!(respond_osc_hits(), 1);
        reset_respond_osc_counters();
        assert_eq!(respond_osc_hits(), 0);
        assert_eq!(respond_osc_start(), usize::MAX);
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_osc_hits_zero_without_osc() {
        reset_respond_osc_counters();
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"plain text".to_vec();
        respond_to_terminal_queries(&mut buffer, write_fd);
        assert_eq!(respond_osc_hits(), 0);
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_to_terminal_queries_passthrough_handles_trailing_escape() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"tail\x1b".to_vec();
        respond_to_terminal_queries_passthrough(&mut buffer, write_fd);
        assert_eq!(buffer, b"tail\x1b".to_vec());
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_to_terminal_queries_passthrough_handles_unknown_escape() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"pre\x1bXpost".to_vec();
        respond_to_terminal_queries_passthrough(&mut buffer, write_fd);
        assert_eq!(buffer, b"pre\x1bXpost".to_vec());
        unsafe { libc::close(write_fd) };
        let _ = read_all(read_fd);
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn apply_control_edits_backspace_then_carriage_return_resets_line_start() {
        let mut data = b"abc\n\x08\rZ".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"Z");
    }

    #[test]
    fn apply_control_edits_preserves_non_osc_escape() {
        reset_apply_osc_counters();
        let mut data = b"\x1b[31mred".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"\x1b[31mred".to_vec());
        assert_eq!(apply_osc_hits(), 0);
    }

    #[test]
    fn apply_control_edits_handles_trailing_escape() {
        let mut data = b"hi\x1b".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"hi\x1b".to_vec());
    }

    #[test]
    fn apply_control_edits_records_osc_start() {
        reset_apply_osc_counters();
        let mut data = b"pre\x1b]0\x07post".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"prepost".to_vec());
        assert_eq!(apply_osc_hits(), 1);
        assert_eq!(apply_osc_start(), 5);
    }

    #[test]
    fn apply_osc_counters_reset_clears_hits() {
        reset_apply_osc_counters();
        let mut data = b"\x1b]0\x07".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(apply_osc_hits(), 1);
        reset_apply_osc_counters();
        assert_eq!(apply_osc_hits(), 0);
        assert_eq!(apply_osc_start(), usize::MAX);
    }

    #[test]
    fn apply_control_edits_handles_osc_near_end_preserves_trailing() {
        reset_apply_osc_counters();
        let mut data = b"hello!\x1b]\x07X".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"hello!X".to_vec());
        assert_eq!(apply_osc_hits(), 1);
        assert_eq!(apply_osc_start(), 8);
    }

    #[test]
    fn apply_control_edits_handles_osc_with_prior_bel() {
        reset_apply_osc_counters();
        let mut data = b"A\x07B\x1b]0\x07Z".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"A\x07BZ".to_vec());
        assert_eq!(apply_osc_hits(), 1);
        assert_eq!(apply_osc_start(), 5);
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_does_not_poll_when_elapsed_equals_timeout() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_wait_for_exit_elapsed_override(None);
            }
        }

        let _guard = wait_for_exit_lock().lock().unwrap();
        reset_wait_for_exit_counters();
        set_wait_for_exit_elapsed_override(Some(0));
        let _reset = OverrideReset;
        assert!(!wait_for_exit(99999, Duration::from_millis(0)));
        assert_eq!(wait_for_exit_poll_count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_elapsed_override_skips_polling() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_wait_for_exit_elapsed_override(None);
            }
        }

        let _guard = wait_for_exit_lock().lock().unwrap();
        reset_wait_for_exit_counters();
        set_wait_for_exit_elapsed_override(Some(10_000));
        let _reset = OverrideReset;
        assert!(!wait_for_exit(99999, Duration::from_millis(100)));
        assert_eq!(wait_for_exit_poll_count(), 0);
        assert_eq!(wait_for_exit_error_count(), 0);
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_records_reap_count() {
        let _guard = wait_for_exit_lock().lock().unwrap();
        reset_wait_for_exit_counters();
        set_wait_for_exit_elapsed_override(None);
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        assert!(wait_for_exit(pid, Duration::from_secs(1)));
        let _ = child.wait();
        assert!(wait_for_exit_reap_count() >= 1);
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_reports_error_for_invalid_pid() {
        let _guard = wait_for_exit_lock().lock().unwrap();
        reset_wait_for_exit_counters();
        set_wait_for_exit_elapsed_override(None);
        assert!(wait_for_exit(99999, Duration::from_millis(10)));
        assert_eq!(wait_for_exit_error_count(), 1);
    }

    #[test]
    fn waitpid_failed_flags_negative() {
        assert!(waitpid_failed(-1));
        assert!(!waitpid_failed(0));
        assert!(!waitpid_failed(1));
    }

    #[test]
    fn respond_to_terminal_queries_strips_osc_and_modes() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"hello\x1b]0;title\x07world\x1b[?2004h!".to_vec();

        respond_to_terminal_queries(&mut buffer, write_fd);

        assert_eq!(buffer, b"helloworld!".to_vec());
        unsafe { libc::close(write_fd) };
        let reply = read_all(read_fd);
        assert!(reply.is_empty());
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn respond_to_terminal_queries_passthrough_handles_multiple_queries() {
        let (read_fd, write_fd) = pipe_pair();
        let mut buffer = b"\x1b[31mred\x1b[6nmid\x1b[5n\x1b[0m".to_vec();

        respond_to_terminal_queries_passthrough(&mut buffer, write_fd);

        assert_eq!(buffer, b"\x1b[31mredmid\x1b[0m".to_vec());
        unsafe { libc::close(write_fd) };
        let reply = read_all(read_fd);
        assert!(reply_contains(&reply, b"\x1b[24;80R"));
        assert!(reply_contains(&reply, b"\x1b[0n"));
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn apply_control_edits_strips_osc_sequences() {
        let mut data = b"hi\x1b]0;title\x07there".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"hithere");
    }

    #[test]
    fn apply_control_edits_backspace_before_line_start_rewinds() {
        reset_apply_linestart_recalc_count();
        let mut data = b"abc\n\x08Z".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"abcZ");
        assert_eq!(apply_linestart_recalc_count(), 1);
    }

    #[test]
    fn apply_control_edits_backspace_at_line_end_does_not_recalc() {
        reset_apply_linestart_recalc_count();
        let mut data = b"ab\nc\x08".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(data, b"ab\n".to_vec());
        assert_eq!(apply_linestart_recalc_count(), 0);
    }

    #[test]
    fn reset_apply_linestart_recalc_count_clears() {
        reset_apply_linestart_recalc_count();
        let mut data = b"abc\n\x08Z".to_vec();
        apply_control_edits(&mut data);
        assert_eq!(apply_linestart_recalc_count(), 1);
        reset_apply_linestart_recalc_count();
        assert_eq!(apply_linestart_recalc_count(), 0);
    }

    #[test]
    fn pop_last_codepoint_handles_multibyte() {
        let mut data = b"a\xC3\xA9".to_vec();
        pop_last_codepoint(&mut data);
        assert_eq!(data, b"a");
    }

    #[test]
    fn pop_last_codepoint_removes_trailing_newline() {
        let mut data = b"line\n".to_vec();
        pop_last_codepoint(&mut data);
        assert_eq!(data, b"line");
    }

    #[test]
    fn current_line_start_handles_lines() {
        assert_eq!(current_line_start(b"abc"), 0);
        assert_eq!(current_line_start(b"abc\n"), 4);
        assert_eq!(current_line_start(b"abc\nxyz"), 4);
    }

    #[test]
    fn find_csi_sequence_returns_none_for_incomplete_sequence() {
        assert_eq!(find_csi_sequence(b"12;", 0), None);
    }

    #[test]
    fn find_osc_terminator_ignores_incomplete_escape() {
        assert_eq!(find_osc_terminator(b"no-term\x1b", 0), None);
        let with_bel = b"hi\x1bXmore\x07";
        assert_eq!(find_osc_terminator(with_bel, 0), Some(with_bel.len()));
    }

    #[test]
    fn should_strip_without_reply_additional_cases() {
        assert!(should_strip_without_reply(b"?25", b'h'));
        assert!(should_strip_without_reply(b"?1", b'u'));
        assert!(!should_strip_without_reply(b"2004", b'h'));
        assert!(!should_strip_without_reply(b">1", b'n'));
    }

    #[test]
    fn csi_reply_normalizes_leading_markers() {
        assert_eq!(csi_reply(b"?6", b'n', 4, 5), Some(b"\x1b[4;5R".to_vec()));
        assert_eq!(csi_reply(b"> 6", b'n', 3, 2), Some(b"\x1b[3;2R".to_vec()));
    }

    #[test]
    fn current_terminal_size_reads_winsize() {
        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(None);
        let (master, slave) = open_pty_pair();
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        ws.ws_row = 40;
        ws.ws_col = 120;
        unsafe {
            assert_eq!(libc::ioctl(master, libc::TIOCSWINSZ, &ws), 0);
        }
        assert_eq!(current_terminal_size(master), (40, 120));
        unsafe {
            libc::close(master);
            libc::close(slave);
        }
    }

    #[test]
    fn current_terminal_size_falls_back_when_dimension_zero() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_terminal_size_override(None);
            }
        }

        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(Some((true, 0, 80)));
        let _reset = OverrideReset;
        assert_eq!(current_terminal_size(0), (24, 80));
    }

    #[test]
    fn current_terminal_size_falls_back_when_override_col_zero() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_terminal_size_override(None);
            }
        }

        let _guard = terminal_size_override_lock().lock().unwrap();
        set_terminal_size_override(Some((true, 10, 0)));
        let _reset = OverrideReset;
        assert_eq!(current_terminal_size(0), (24, 80));
    }

    #[test]
    fn write_all_errors_on_invalid_fd() {
        assert!(write_all(-1, b"oops").is_err());
    }

    #[test]
    fn pty_codex_session_read_output_drains_channel() {
        let (tx, rx) = bounded(4);
        tx.send(b"one".to_vec()).unwrap();
        tx.send(b"two".to_vec()).unwrap();
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let output = session.read_output();
        assert_eq!(output, vec![b"one".to_vec(), b"two".to_vec()]);
        assert!(session.read_output().is_empty());
    }

    #[test]
    fn pty_codex_session_read_output_timeout_elapsed_override_prevents_loop() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_read_output_elapsed_override(None);
            }
        }

        let _guard = read_output_timeout_lock().lock().unwrap();
        set_read_output_elapsed_override(Some(5_000));
        let _reset = OverrideReset;

        let (tx, rx) = bounded(1);
        tx.send(b"ready".to_vec()).unwrap();
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let output = session.read_output_timeout(Duration::from_millis(10));
        assert!(output.is_empty());
    }

    #[test]
    fn pty_codex_session_read_output_timeout_respects_zero_timeout() {
        let _guard = read_output_timeout_lock().lock().unwrap();
        set_read_output_grace_override(None);
        let (tx, rx) = bounded(1);
        tx.send(b"ready".to_vec()).unwrap();
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let output = session.read_output_timeout(Duration::from_millis(0));
        assert!(output.is_empty());
    }

    #[test]
    fn pty_codex_session_read_output_timeout_elapsed_boundary() {
        struct OverrideReset;
        impl Drop for OverrideReset {
            fn drop(&mut self) {
                set_read_output_elapsed_override(None);
            }
        }

        let _guard = read_output_timeout_lock().lock().unwrap();
        set_read_output_elapsed_override(Some(0));
        let _reset = OverrideReset;

        let (tx, rx) = bounded(1);
        tx.send(b"ready".to_vec()).unwrap();
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let output = session.read_output_timeout(Duration::from_millis(0));
        assert!(output.is_empty());
    }

    #[test]
    fn pty_codex_session_read_output_timeout_collects_recent_chunks() {
        let _guard = read_output_timeout_lock().lock().unwrap();
        set_read_output_grace_override(None);
        let (tx, rx) = bounded(4);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let sender = thread::spawn(move || {
            tx.send(b"first".to_vec()).unwrap();
            thread::sleep(Duration::from_millis(150));
            tx.send(b"second".to_vec()).unwrap();
        });
        let output = session.read_output_timeout(Duration::from_millis(800));
        sender.join().unwrap();
        assert_eq!(output, vec![b"first".to_vec(), b"second".to_vec()]);
    }

    #[test]
    fn pty_codex_session_read_output_timeout_breaks_on_grace_boundary() {
        struct GraceReset;
        impl Drop for GraceReset {
            fn drop(&mut self) {
                set_read_output_grace_override(None);
            }
        }

        let _guard = read_output_timeout_lock().lock().unwrap();
        set_read_output_grace_override(Some(300));
        let _reset = GraceReset;

        let (tx, rx) = bounded(4);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let sender = thread::spawn(move || {
            tx.send(b"first".to_vec()).unwrap();
            thread::sleep(Duration::from_millis(200));
            tx.send(b"second".to_vec()).unwrap();
        });
        let output = session.read_output_timeout(Duration::from_millis(700));
        sender.join().unwrap();
        assert_eq!(output, vec![b"first".to_vec()]);
    }

    #[test]
    fn pty_codex_session_wait_for_output_collects_and_drains() {
        let (tx, rx) = bounded(4);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        let sender = thread::spawn(move || {
            tx.send(b"first".to_vec()).unwrap();
            tx.send(b"second".to_vec()).unwrap();
        });
        let output = session.wait_for_output(Duration::from_millis(200));
        sender.join().unwrap();
        assert_eq!(output, vec![b"first".to_vec(), b"second".to_vec()]);
    }

    #[test]
    fn pty_codex_session_is_responsive_tracks_liveness() {
        let mut child = std::process::Command::new("sleep")
            .arg("1")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let mut session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: pid,
            output_rx: rx,
            _output_thread: handle,
        });
        assert!(session.is_responsive(Duration::from_millis(10)));
        let _ = child.kill();
        let _ = child.wait();
        assert!(!session.is_responsive(Duration::from_millis(10)));
    }

    #[test]
    fn pty_codex_session_is_alive_reflects_child() {
        let mut child = std::process::Command::new("sleep")
            .arg("1")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyCodexSession {
            master_fd: -1,
            child_pid: pid,
            output_rx: rx,
            _output_thread: handle,
        });
        assert!(session.is_alive());
        let _ = child.kill();
        let _ = child.wait();
        assert!(!session.is_alive());
    }

    #[test]
    fn pty_overlay_session_send_bytes_writes() {
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let mut session = ManuallyDrop::new(PtyOverlaySession {
            master_fd: write_fd,
            child_pid: -1,
            output_rx: rx,
            _output_thread: handle,
        });
        session.send_bytes(b"bytes").unwrap();
        unsafe { libc::close(write_fd) };
        let output = read_all(read_fd);
        assert_eq!(output, b"bytes");
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn pty_overlay_session_set_winsize_updates_and_minimums() {
        let (master, slave) = open_pty_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyOverlaySession {
            master_fd: master,
            child_pid: unsafe { libc::getpid() },
            output_rx: rx,
            _output_thread: handle,
        });
        session.set_winsize(0, 0).unwrap();
        let mut ws: libc::winsize = unsafe { mem::zeroed() };
        unsafe {
            assert_eq!(libc::ioctl(master, libc::TIOCGWINSZ, &mut ws), 0);
        }
        assert_eq!(ws.ws_row, 1);
        assert_eq!(ws.ws_col, 1);
        unsafe {
            libc::close(master);
            libc::close(slave);
        }
    }

    #[test]
    fn pty_overlay_session_set_winsize_errors_on_invalid_fd() {
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyOverlaySession {
            master_fd: -1,
            child_pid: unsafe { libc::getpid() },
            output_rx: rx,
            _output_thread: handle,
        });
        assert!(session.set_winsize(10, 10).is_err());
    }

    #[test]
    fn pty_overlay_session_is_alive_reflects_child() {
        let mut child = std::process::Command::new("sleep")
            .arg("1")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let session = ManuallyDrop::new(PtyOverlaySession {
            master_fd: -1,
            child_pid: pid,
            output_rx: rx,
            _output_thread: handle,
        });
        assert!(session.is_alive());
        let _ = child.kill();
        let _ = child.wait();
        assert!(!session.is_alive());
    }

    #[test]
    fn spawn_reader_thread_recovers_from_wouldblock() {
        let (read_fd, write_fd) = pipe_pair();
        set_nonblocking_fd(read_fd);
        let (tx, rx) = bounded(2);
        let handle = spawn_reader_thread(read_fd, tx);
        thread::sleep(Duration::from_millis(20));
        unsafe {
            libc::write(write_fd, b"ping".as_ptr() as *const libc::c_void, 4);
            libc::close(write_fd);
        }
        let data = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(data, b"ping");
        handle.join().unwrap();
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn spawn_reader_thread_does_not_log_on_eof() {
        let (read_fd, write_fd) = pipe_pair();
        let (tx, _rx) = bounded(1);
        let handle = spawn_reader_thread(read_fd, tx);
        let log = capture_new_log(|| unsafe {
            libc::close(write_fd);
            handle.join().unwrap();
        });
        assert!(!log.contains("PTY read error"));
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn spawn_passthrough_reader_thread_recovers_from_wouldblock() {
        let (read_fd, write_fd) = pipe_pair();
        set_nonblocking_fd(read_fd);
        let (tx, rx) = bounded(2);
        let handle = spawn_passthrough_reader_thread(read_fd, tx);
        thread::sleep(Duration::from_millis(20));
        unsafe {
            libc::write(write_fd, b"pong".as_ptr() as *const libc::c_void, 4);
            libc::close(write_fd);
        }
        let data = rx.recv_timeout(Duration::from_secs(1)).unwrap();
        assert_eq!(data, b"pong");
        handle.join().unwrap();
        unsafe { libc::close(read_fd) };
    }

    #[test]
    fn spawn_passthrough_reader_thread_does_not_log_on_eof() {
        let (read_fd, write_fd) = pipe_pair();
        let (tx, _rx) = bounded(1);
        let handle = spawn_passthrough_reader_thread(read_fd, tx);
        let log = capture_new_log(|| unsafe {
            libc::close(write_fd);
            handle.join().unwrap();
        });
        assert!(!log.contains("PTY read error"));
        unsafe { libc::close(read_fd) };
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_with_zero_timeout_does_not_poll() {
        let _guard = wait_for_exit_lock().lock().unwrap();
        set_wait_for_exit_elapsed_override(None);
        let mut child = std::process::Command::new("true")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        assert!(!wait_for_exit(pid, Duration::from_millis(0)));
        let _ = child.wait();
    }

    #[cfg(unix)]
    #[test]
    fn wait_for_exit_reaps_forked_child() {
        let _guard = wait_for_exit_lock().lock().unwrap();
        set_wait_for_exit_elapsed_override(None);
        unsafe {
            let pid = libc::fork();
            assert!(pid >= 0);
            if pid == 0 {
                libc::_exit(0);
            }
            let result = wait_for_exit(pid, Duration::from_secs(1));
            let mut status = 0;
            let _ = libc::waitpid(pid, &mut status, 0);
            assert!(result);
        }
    }

    #[cfg(unix)]
    #[test]
    fn pty_codex_session_drop_terminates_child() {
        let mut child = std::process::Command::new("sleep")
            .arg("5")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let log = capture_new_log(|| {
            let session = PtyCodexSession {
                master_fd: write_fd,
                child_pid: pid,
                output_rx: rx,
                _output_thread: handle,
            };
            drop(session);
        });
        assert!(!log.contains("SIGTERM to Codex session failed"));
        assert!(!log.contains("SIGKILL to Codex session failed"));
        unsafe { libc::close(read_fd) };
        let mut status = 0;
        let start = Instant::now();
        let mut alive = true;
        while start.elapsed() < Duration::from_millis(200) {
            let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
            if result == 0 {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            alive = false;
            break;
        }
        if alive {
            unsafe {
                libc::kill(pid, libc::SIGKILL);
                let _ = libc::waitpid(pid, &mut status, 0);
            }
            panic!("child still alive after PtyCodexSession drop");
        }
        let _ = child.wait();
    }

    #[cfg(unix)]
    #[test]
    fn pty_codex_session_drop_sigkill_for_ignored_sigterm() {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg("trap '' TERM; sleep 5")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let log = capture_new_log(|| {
            let session = PtyCodexSession {
                master_fd: write_fd,
                child_pid: pid,
                output_rx: rx,
                _output_thread: handle,
            };
            drop(session);
        });
        assert!(!log.contains("SIGKILL to Codex session failed"));
        unsafe { libc::close(read_fd) };
        let mut status = 0;
        let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if result == 0 {
            unsafe {
                libc::kill(pid, libc::SIGKILL);
                let result = libc::waitpid(pid, &mut status, 0);
                assert!(
                    result > 0 || result == -1,
                    "child still alive after SIGKILL in PtyCodexSession drop"
                );
            }
        }
        let _ = child.wait();
    }

    #[cfg(unix)]
    #[test]
    fn pty_overlay_session_drop_terminates_child() {
        let mut child = std::process::Command::new("sleep")
            .arg("5")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let log = capture_new_log(|| {
            let session = PtyOverlaySession {
                master_fd: write_fd,
                child_pid: pid,
                output_rx: rx,
                _output_thread: handle,
            };
            drop(session);
        });
        assert!(!log.contains("SIGTERM to Codex session failed"));
        assert!(!log.contains("SIGKILL to Codex session failed"));
        unsafe { libc::close(read_fd) };
        let mut status = 0;
        let start = Instant::now();
        let mut alive = true;
        while start.elapsed() < Duration::from_millis(200) {
            let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
            if result == 0 {
                thread::sleep(Duration::from_millis(10));
                continue;
            }
            alive = false;
            break;
        }
        if alive {
            unsafe {
                libc::kill(pid, libc::SIGKILL);
                let _ = libc::waitpid(pid, &mut status, 0);
            }
            panic!("child still alive after PtyOverlaySession drop");
        }
        let _ = child.wait();
    }

    #[cfg(unix)]
    #[test]
    fn pty_overlay_session_drop_sigkill_for_ignored_sigterm() {
        let mut child = std::process::Command::new("sh")
            .arg("-c")
            .arg("trap '' TERM; sleep 5")
            .spawn()
            .expect("spawned child");
        let pid = child.id() as i32;
        let (read_fd, write_fd) = pipe_pair();
        let (_tx, rx) = bounded(1);
        let handle = thread::spawn(|| {});
        let log = capture_new_log(|| {
            let session = PtyOverlaySession {
                master_fd: write_fd,
                child_pid: pid,
                output_rx: rx,
                _output_thread: handle,
            };
            drop(session);
        });
        assert!(!log.contains("SIGKILL to Codex session failed"));
        unsafe { libc::close(read_fd) };
        let mut status = 0;
        let result = unsafe { libc::waitpid(pid, &mut status, libc::WNOHANG) };
        if result == 0 {
            unsafe {
                libc::kill(pid, libc::SIGKILL);
                let result = libc::waitpid(pid, &mut status, 0);
                assert!(
                    result > 0 || result == -1,
                    "child still alive after SIGKILL in PtyOverlaySession drop"
                );
            }
        }
        let _ = child.wait();
    }

    fn reply_contains(haystack: &[u8], needle: &[u8]) -> bool {
        haystack
            .windows(needle.len())
            .any(|window| window == needle)
    }
}
