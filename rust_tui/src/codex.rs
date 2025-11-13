//! Async Codex worker that mirrors the `VoiceJob` pattern so Codex calls never
//! block the UI thread. Also hosts shared Codex helpers (PTY sanitization, prompt
//! formatting) so the rest of the app can stay lean.

use crate::{
    config::AppConfig, log_debug, pty_session::PtyCodexSession, utf8_safe::window_by_columns,
};
use anyhow::{anyhow, Context, Result};
use std::{
    env, fmt,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Child, Command, Output, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, TryRecvError},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};
use strip_ansi_escapes::strip;
use unicode_width::UnicodeWidthStr;

/// Spinner frames used by the UI when a Codex request is inflight.
pub const CODEX_SPINNER_FRAMES: &[char] = &['-', '\\', '|', '/'];

/// Handle to an asynchronous Codex invocation.
pub struct CodexJob {
    pub receiver: mpsc::Receiver<CodexJobMessage>,
    pub handle: Option<thread::JoinHandle<()>>,
    cancel_token: CancelToken,
}

impl CodexJob {
    /// Request cancellation; the worker best-effort terminates subprocesses and
    /// emits `CodexJobMessage::Canceled`.
    pub fn cancel(&self) {
        self.cancel_token.cancel();
    }

    /// Whether cancellation has been requested.
    pub fn is_cancelled(&self) -> bool {
        self.cancel_token.is_cancelled()
    }
}

/// Worker → UI messages describing Codex outcomes.
pub enum CodexJobMessage {
    Completed {
        lines: Vec<String>,
        status: String,
        codex_session: Option<PtyCodexSession>,
    },
    Failed {
        error: String,
        codex_session: Option<PtyCodexSession>,
    },
    Canceled {
        codex_session: Option<PtyCodexSession>,
    },
}

impl fmt::Debug for CodexJobMessage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CodexJobMessage::Completed { status, .. } => {
                f.debug_struct("Completed").field("status", status).finish()
            }
            CodexJobMessage::Failed { error, .. } => {
                f.debug_struct("Failed").field("error", error).finish()
            }
            CodexJobMessage::Canceled { .. } => f.debug_struct("Canceled").finish(),
        }
    }
}

/// Spawn a Codex worker thread and return a handle the UI can poll.
pub fn start_codex_job(
    prompt: String,
    config: AppConfig,
    codex_session: Option<PtyCodexSession>,
) -> CodexJob {
    let (tx, rx) = mpsc::channel();
    let cancel_token = CancelToken::new();
    let cancel_for_worker = cancel_token.clone();
    let handle = thread::spawn(move || {
        let message = run_codex_job(prompt, config, codex_session, cancel_for_worker);
        let _ = tx.send(message);
    });

    CodexJob {
        receiver: rx,
        handle: Some(handle),
        cancel_token,
    }
}

#[derive(Clone)]
struct CancelToken {
    flag: Arc<AtomicBool>,
}

impl CancelToken {
    fn new() -> Self {
        Self {
            flag: Arc::new(AtomicBool::new(false)),
        }
    }

    fn cancel(&self) {
        self.flag.store(true, Ordering::SeqCst);
    }

    pub(crate) fn is_cancelled(&self) -> bool {
        self.flag.load(Ordering::SeqCst)
    }
}

#[derive(Debug)]
enum CodexCallError {
    Cancelled,
    Failure(anyhow::Error),
}

impl From<anyhow::Error> for CodexCallError {
    fn from(err: anyhow::Error) -> Self {
        Self::Failure(err)
    }
}

impl From<std::io::Error> for CodexCallError {
    fn from(err: std::io::Error) -> Self {
        Self::Failure(err.into())
    }
}

fn run_codex_job(
    prompt: String,
    config: AppConfig,
    mut codex_session: Option<PtyCodexSession>,
    cancel: CancelToken,
) -> CodexJobMessage {
    #[cfg(test)]
    if let Some(message) = try_job_hook(&prompt, &cancel) {
        return message;
    }

    if prompt.trim().is_empty() {
        return CodexJobMessage::Failed {
            error: "Prompt is empty.".into(),
            codex_session,
        };
    }

    let codex_start = Instant::now();
    let mut used_persistent = false;
    let mut codex_output: Option<String> = None;

    if config.persistent_codex {
        if let Some(mut session) = codex_session.take() {
            log_debug("CodexJob: Trying persistent Codex session");
            match call_codex_via_session(&mut session, &prompt, &cancel) {
                Ok(text) => {
                    used_persistent = true;
                    codex_session = Some(session);
                    codex_output = Some(text);
                }
                Err(CodexCallError::Cancelled) => {
                    return CodexJobMessage::Canceled {
                        codex_session: Some(session),
                    };
                }
                Err(err) => {
                    log_debug(&format!(
                        "Persistent Codex session failed, falling back: {err:?}"
                    ));
                }
            }
        }
    }

    if cancel.is_cancelled() {
        return CodexJobMessage::Canceled { codex_session };
    }

    let output_text = match codex_output {
        Some(text) => text,
        None => match call_codex_cli(&config, &prompt, &cancel) {
            Ok(text) => text,
            Err(CodexCallError::Cancelled) => {
                return CodexJobMessage::Canceled { codex_session };
            }
            Err(CodexCallError::Failure(err)) => {
                return CodexJobMessage::Failed {
                    error: format!("{err:#}"),
                    codex_session,
                };
            }
        },
    };

    let elapsed = codex_start.elapsed().as_secs_f64();
    let line_count = output_text.lines().count();
    if config.log_timings {
        log_debug(&format!(
            "timing|phase=codex_job|persistent_used={}|elapsed_s={:.3}|lines={}|chars={}",
            used_persistent,
            elapsed,
            line_count,
            output_text.len()
        ));
    }

    let sanitized_output = if used_persistent {
        output_text
    } else {
        sanitize_pty_output(output_text.as_bytes())
    };
    let sanitized_lines = prepare_for_display(&sanitized_output);

    let mut lines = Vec::with_capacity(sanitized_lines.len() + 4);
    let prompt_line = format!("> {}", prompt.trim());
    log_debug(&format!(
        "CodexJob: Adding prompt line to output: {}",
        preview_for_log(&prompt_line, 50)
    ));
    lines.push(prompt_line);
    lines.push(String::new());

    for (idx, line) in sanitized_lines.iter().enumerate() {
        if !line.is_empty() {
            log_debug(&format!(
                "CodexJob: sanitized_line[{idx}] {}",
                preview_for_log(line, 50)
            ));
        }
    }

    lines.extend(sanitized_lines);
    lines.push(String::new());
    CodexJobMessage::Completed {
        lines,
        status: format!("Codex returned {line_count} lines."),
        codex_session,
    }
}

fn call_codex_cli(
    config: &AppConfig,
    prompt: &str,
    cancel: &CancelToken,
) -> Result<String, CodexCallError> {
    let codex_working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if let Some(result) = try_python_pty(config, prompt, &codex_working_dir, cancel)? {
        match result {
            PtyResult::Success(text) => return Ok(text),
            PtyResult::Failure(msg) => {
                log_debug(&format!("Codex PTY helper failed; falling back: {msg}"));
            }
        }
    }

    // Primary fallback: interactive Codex invocation
    let mut interactive_cmd = Command::new(&config.codex_cmd);
    interactive_cmd
        .args(&config.codex_args)
        .arg("-C")
        .arg(&codex_working_dir)
        .env("TERM", &config.term_value)
        .env("CODEX_NONINTERACTIVE", "1")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let interactive_output =
        spawn_with_cancel(interactive_cmd, Some(prompt), cancel).map_err(|err| match err {
            CodexCallError::Cancelled => CodexCallError::Cancelled,
            CodexCallError::Failure(e) => CodexCallError::Failure(
                e.context(format!("failed to spawn interactive {}", config.codex_cmd)),
            ),
        })?;

    if interactive_output.status.success() {
        return Ok(String::from_utf8_lossy(&interactive_output.stdout).to_string());
    }

    let interactive_stderr = String::from_utf8_lossy(&interactive_output.stderr).to_string();

    // Last resort: codex exec -
    let mut exec_cmd = Command::new(&config.codex_cmd);
    exec_cmd
        .arg("exec")
        .arg("-C")
        .arg(&codex_working_dir)
        .args(&config.codex_args)
        .env("TERM", &config.term_value)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let exec_output =
        spawn_with_cancel(exec_cmd, Some(prompt), cancel).map_err(|err| match err {
            CodexCallError::Cancelled => CodexCallError::Cancelled,
            CodexCallError::Failure(e) => CodexCallError::Failure(
                e.context(format!("failed to spawn {} exec", config.codex_cmd)),
            ),
        })?;

    if exec_output.status.success() {
        return Ok(String::from_utf8_lossy(&exec_output.stdout).to_string());
    }

    Err(CodexCallError::Failure(anyhow!(
        "All codex invocation methods failed:\n\
           PTY Helper: Check if scripts/run_in_pty.py exists\n\
           Interactive: {}\n\
           Exec mode: {}",
        interactive_stderr.trim(),
        String::from_utf8_lossy(&exec_output.stderr).trim()
    )))
}

fn call_codex_via_session(
    session: &mut PtyCodexSession,
    prompt: &str,
    cancel: &CancelToken,
) -> Result<String, CodexCallError> {
    session
        .send(prompt)
        .context("failed to write prompt to persistent Codex session")?;

    let mut combined_raw = Vec::new();
    let start_time = Instant::now();
    let overall_timeout = Duration::from_secs(30);
    let quiet_grace = Duration::from_millis(350);
    let mut last_progress = Instant::now();
    let mut last_len = 0usize;

    loop {
        if cancel.is_cancelled() {
            return Err(CodexCallError::Cancelled);
        }

        let output_chunks = session.read_output_timeout(Duration::from_millis(500));
        for chunk in output_chunks {
            log_debug(&format!(
                "pty_raw_chunk[{}] {}",
                combined_raw.len(),
                format_bytes_for_log(&chunk)
            ));
            combined_raw.extend_from_slice(&chunk);
            last_progress = Instant::now();
        }

        if !combined_raw.is_empty() {
            let sanitized = sanitize_pty_output(&combined_raw);
            if sanitized.len() != last_len {
                last_len = sanitized.len();
                last_progress = Instant::now();
            }
            log_debug(&format!(
                "pty_sanitized_output {} chars",
                sanitized.chars().count()
            ));
            let idle = Instant::now().duration_since(last_progress);
            if !sanitized.trim().is_empty() && idle >= quiet_grace {
                return Ok(sanitized);
            }
            if idle >= overall_timeout {
                if sanitized.trim().is_empty() {
                    break;
                }
                return Ok(sanitized);
            }
        } else if Instant::now().duration_since(start_time) >= overall_timeout {
            break;
        }
    }

    log_debug("Persistent Codex session yielded no printable output; falling back");
    Err(CodexCallError::Failure(anyhow!(
        "persistent Codex session returned no text"
    )))
}

/// Outcome of the optional Python PTY helper invocation.
enum PtyResult {
    Success(String),
    Failure(String),
}

fn try_python_pty(
    config: &AppConfig,
    prompt: &str,
    working_dir: &Path,
    cancel: &CancelToken,
) -> Result<Option<PtyResult>, CodexCallError> {
    if !config.pty_helper.exists() {
        return Ok(None);
    }

    let mut cmd = Command::new(&config.python_cmd);
    cmd.arg(&config.pty_helper);
    cmd.arg("--stdin");
    cmd.arg(&config.codex_cmd);
    cmd.arg("-C");
    cmd.arg(working_dir);
    cmd.args(&config.codex_args);
    cmd.env("TERM", &config.term_value);
    cmd.stdin(Stdio::piped());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    let output = spawn_with_cancel(cmd, Some(prompt), cancel).map_err(|err| match err {
        CodexCallError::Cancelled => CodexCallError::Cancelled,
        CodexCallError::Failure(e) => CodexCallError::Failure(e.context(format!(
            "failed to run PTY helper {}",
            config.pty_helper.display()
        ))),
    })?;

    if output.status.success() {
        return Ok(Some(PtyResult::Success(
            String::from_utf8_lossy(&output.stdout).to_string(),
        )));
    }

    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let msg = if stderr.is_empty() {
        format!(
            "PTY helper exit {}: {}",
            output.status,
            config.pty_helper.display()
        )
    } else {
        format!("PTY helper exit {}: {}", output.status, stderr)
    };
    Ok(Some(PtyResult::Failure(msg)))
}

fn spawn_with_cancel(
    mut cmd: Command,
    prompt: Option<&str>,
    cancel: &CancelToken,
) -> Result<Output, CodexCallError> {
    let mut child = cmd.spawn()?;
    if let Some(text) = prompt {
        if let Some(mut stdin) = child.stdin.take() {
            write_prompt_with_newline(&mut stdin, text)?;
        }
    }
    wait_child_with_cancel(child, cancel)
}

fn wait_child_with_cancel(child: Child, cancel: &CancelToken) -> Result<Output, CodexCallError> {
    let pid = child.id();
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let result = child.wait_with_output();
        let _ = tx.send(result);
    });

    let mut cancel_requested_at: Option<Instant> = None;
    let mut sigkill_sent = false;

    loop {
        match rx.try_recv() {
            Ok(result) => {
                return match result {
                    Ok(output) => {
                        if cancel_requested_at.is_some() {
                            Err(CodexCallError::Cancelled)
                        } else {
                            Ok(output)
                        }
                    }
                    Err(err) => Err(CodexCallError::Failure(err.into())),
                };
            }
            Err(TryRecvError::Disconnected) => {
                return Err(CodexCallError::Failure(anyhow!(
                    "Codex child waiter disconnected unexpectedly"
                )));
            }
            Err(TryRecvError::Empty) => {}
        }

        if cancel.is_cancelled() {
            if cancel_requested_at.is_none() {
                log_debug("CodexJob: cancellation requested; sending SIGTERM");
                send_signal(pid, Signal::Term);
                cancel_requested_at = Some(Instant::now());
            } else if !sigkill_sent
                && cancel_requested_at
                    .map(|start| start.elapsed() >= Duration::from_millis(500))
                    .unwrap_or(false)
            {
                log_debug("CodexJob: escalation to SIGKILL");
                send_signal(pid, Signal::Kill);
                sigkill_sent = true;
            }
        }

        thread::sleep(Duration::from_millis(50));
    }
}

enum Signal {
    Term,
    Kill,
}

fn send_signal(pid: u32, signal: Signal) {
    #[cfg(unix)]
    unsafe {
        let signo = match signal {
            Signal::Term => libc::SIGTERM,
            Signal::Kill => libc::SIGKILL,
        };
        if libc::kill(pid as i32, signo) != 0 {
            log_debug(&format!(
                "CodexJob: failed to send signal {signo} to pid {pid}: {}",
                io::Error::last_os_error()
            ));
        }
    }

    #[cfg(not(unix))]
    {
        let _ = pid;
        let _ = signal;
        log_debug("CodexJob: cancellation requested, but signals unsupported on this platform");
    }
}

/// Normalize CR/LF pairs, strip ANSI, and guarantee clean UTF-8 suitable for the TUI.
pub fn sanitize_pty_output(raw: &[u8]) -> String {
    if raw.is_empty() {
        return String::new();
    }

    let normalized = normalize_control_bytes(raw);
    let ansi_free = strip(&normalized);
    let mut text = String::from_utf8_lossy(&ansi_free).to_string();
    if raw.last() == Some(&b'\n') && !text.ends_with('\n') {
        text.push('\n');
    }
    text
}

pub fn prepare_for_display(text: &str) -> Vec<String> {
    text.lines().map(|line| line.to_string()).collect()
}

fn preview_for_log(text: &str, max_cols: usize) -> String {
    if max_cols == 0 || text.is_empty() {
        return String::new();
    }
    let slice = window_by_columns(text, 0, max_cols);
    let needs_ellipsis = UnicodeWidthStr::width(text) > UnicodeWidthStr::width(slice);
    if needs_ellipsis {
        format!("{slice}…")
    } else {
        slice.to_string()
    }
}

fn format_bytes_for_log(bytes: &[u8]) -> String {
    const MAX_BYTES: usize = 64;
    let mut parts = Vec::new();
    for (idx, b) in bytes.iter().take(MAX_BYTES).enumerate() {
        parts.push(format!("{b:02X}"));
        if idx >= MAX_BYTES - 1 {
            break;
        }
    }
    if bytes.len() > MAX_BYTES {
        parts.push("...".into());
    }
    parts.join(" ")
}

fn normalize_control_bytes(raw: &[u8]) -> Vec<u8> {
    let mut output = Vec::with_capacity(raw.len());
    let mut idx = 0;
    let mut line_start = 0usize;

    while idx < raw.len() {
        match raw[idx] {
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
                idx += 1;
                let removed_newline = pop_last_codepoint(&mut output);
                if removed_newline {
                    line_start = current_line_start(&output);
                }
            }
            0 => {
                idx += 1;
            }
            0x1B => {
                if let Some(next) = raw.get(idx + 1) {
                    if *next == b']' {
                        idx = skip_osc_sequence(raw, idx + 2);
                        continue;
                    } else if *next == b'[' {
                        if let Some((end, final_byte)) = find_csi_sequence(raw, idx) {
                            if final_byte == b'm' {
                                idx = end + 1;
                                continue;
                            }
                            output.extend_from_slice(raw.get(idx..=end).unwrap_or(&[]));
                            idx = end + 1;
                            continue;
                        }
                    }
                }
                output.push(raw[idx]);
                idx += 1;
            }
            byte => {
                output.push(byte);
                idx += 1;
            }
        }
        if line_start > output.len() {
            line_start = current_line_start(&output);
        }
    }

    output
}

fn pop_last_codepoint(buf: &mut Vec<u8>) -> bool {
    if buf.is_empty() {
        return false;
    }
    if buf.last() == Some(&b'\n') {
        buf.pop();
        return true;
    }
    while let Some(byte) = buf.pop() {
        if (byte & 0b1100_0000) != 0b1000_0000 {
            break;
        }
    }
    false
}

fn current_line_start(buf: &[u8]) -> usize {
    buf.iter()
        .rposition(|&b| b == b'\n')
        .map(|pos| pos + 1)
        .unwrap_or(0)
}

fn skip_osc_sequence(bytes: &[u8], mut cursor: usize) -> usize {
    while cursor < bytes.len() {
        match bytes[cursor] {
            0x07 => return cursor + 1,
            0x1B if cursor + 1 < bytes.len() && bytes[cursor + 1] == b'\\' => {
                return cursor + 2;
            }
            _ => cursor += 1,
        }
    }
    cursor
}

fn find_csi_sequence(bytes: &[u8], start: usize) -> Option<(usize, u8)> {
    if bytes.get(start)? != &0x1B || bytes.get(start + 1)? != &b'[' {
        return None;
    }
    let mut idx = start + 2;
    while idx < bytes.len() {
        let b = bytes[idx];
        if (0x40..=0x7E).contains(&b) {
            return Some((idx, b));
        }
        idx += 1;
    }
    None
}

fn write_prompt_with_newline<W: Write>(writer: &mut W, prompt: &str) -> io::Result<()> {
    writer.write_all(prompt.as_bytes())?;
    if !prompt.ends_with('\n') {
        writer.write_all(b"\n")?;
    }
    Ok(())
}

#[cfg(test)]
use std::sync::{Mutex, MutexGuard, OnceLock};

#[cfg(test)]
#[derive(Clone)]
pub(crate) struct CancelProbe(CancelToken);

#[cfg(test)]
impl CancelProbe {
    pub fn is_cancelled(&self) -> bool {
        self.0.is_cancelled()
    }
}

#[cfg(test)]
type CodexJobHook = Box<dyn Fn(&str, CancelProbe) -> CodexJobMessage + Send + Sync + 'static>;

#[cfg(test)]
static CODEX_JOB_HOOK: OnceLock<Mutex<Option<CodexJobHook>>> = OnceLock::new();
#[cfg(test)]
static JOB_HOOK_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(test)]
fn try_job_hook(prompt: &str, cancel: &CancelToken) -> Option<CodexJobMessage> {
    let storage = CODEX_JOB_HOOK.get_or_init(|| Mutex::new(None));
    let guard = storage.lock().unwrap_or_else(|e| e.into_inner());
    guard
        .as_ref()
        .map(|hook| hook(prompt, CancelProbe(cancel.clone())))
}

#[cfg(test)]
pub(crate) struct JobHookGuard {
    lock: Option<MutexGuard<'static, ()>>,
}

#[cfg(test)]
impl Drop for JobHookGuard {
    fn drop(&mut self) {
        if let Some(storage) = CODEX_JOB_HOOK.get() {
            *storage.lock().unwrap_or_else(|e| e.into_inner()) = None;
        }
        self.lock.take();
    }
}

#[cfg(test)]
pub(crate) fn with_job_hook<R>(hook: CodexJobHook, f: impl FnOnce() -> R) -> (R, JobHookGuard) {
    let storage = CODEX_JOB_HOOK.get_or_init(|| Mutex::new(None));
    let lock = JOB_HOOK_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(|e| e.into_inner());
    *storage.lock().unwrap_or_else(|e| e.into_inner()) = Some(hook);
    let result = f();
    (result, JobHookGuard { lock: Some(lock) })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use clap::Parser;

    fn test_config() -> AppConfig {
        let mut config = AppConfig::parse_from(["codex-job-test"]);
        config.no_python_fallback = true;
        config
    }

    #[test]
    fn sanitize_handles_backspace() {
        let cleaned = sanitize_pty_output(b"a\x08b\n");
        assert_eq!(cleaned, "b\n");
    }

    #[test]
    fn sanitize_strips_cursor_query_bytes() {
        let cleaned = sanitize_pty_output(b"Hello\x1b[6nWorld\n");
        assert_eq!(cleaned, "HelloWorld\n");
    }

    #[test]
    fn sanitize_preserves_numeric_lines() {
        let sample = "2024\n0;1;2\n";
        let cleaned = sanitize_pty_output(sample.as_bytes());
        assert_eq!(cleaned, sample);
    }

    #[test]
    fn sanitize_keeps_wide_glyphs() {
        let sample = "│> Test 😊 你好\n";
        let cleaned = sanitize_pty_output(sample.as_bytes());
        assert_eq!(cleaned, sample);
    }

    #[test]
    fn sanitize_strips_escape_wrapped_cursor_report() {
        let cleaned = sanitize_pty_output(b"\x1b[>0;0;0uHello");
        assert_eq!(cleaned, "Hello");
    }

    #[test]
    fn codex_job_reports_success() {
        let config = test_config();
        let (job, hook_guard) = with_job_hook(
            Box::new(|prompt, _| CodexJobMessage::Completed {
                lines: vec![prompt.to_string()],
                status: "ok".into(),
                codex_session: None,
            }),
            || start_codex_job("hello".into(), config, None),
        );
        let message = job.receiver.recv().expect("message");
        drop(hook_guard);
        match message {
            CodexJobMessage::Completed { lines, status, .. } => {
                assert_eq!(lines, vec!["hello"]);
                assert_eq!(status, "ok");
            }
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn codex_job_reports_failure() {
        let config = test_config();
        let (job, hook_guard) = with_job_hook(
            Box::new(|_, _| CodexJobMessage::Failed {
                error: "boom".into(),
                codex_session: None,
            }),
            || start_codex_job("hello".into(), config, None),
        );
        let message = job.receiver.recv().expect("message");
        drop(hook_guard);
        match message {
            CodexJobMessage::Failed { error, .. } => assert_eq!(error, "boom"),
            other => panic!("unexpected message: {other:?}"),
        }
    }

    #[test]
    fn codex_job_can_cancel() {
        let config = test_config();
        let (job, hook_guard) = with_job_hook(
            Box::new(|_, cancel| {
                while !cancel.is_cancelled() {
                    thread::sleep(Duration::from_millis(10));
                }
                CodexJobMessage::Canceled {
                    codex_session: None,
                }
            }),
            || start_codex_job("hello".into(), config, None),
        );
        job.cancel();
        let message = job.receiver.recv().expect("message");
        drop(hook_guard);
        match message {
            CodexJobMessage::Canceled { .. } => {}
            other => panic!("unexpected message: {other:?}"),
        }
    }
}
