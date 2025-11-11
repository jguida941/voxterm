//! Terminal UI shell for the `codex_voice` pipeline. It mirrors the Python
//! prototype but wraps it in a full-screen experience driven by `ratatui`.

use std::{
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{mpsc::TryRecvError, Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use crate::config::AppConfig;
use crate::utf8_safe::window_by_columns;
use crate::voice::{self, VoiceCaptureTrigger, VoiceJob, VoiceJobMessage};
use crate::{audio, pty_session, stt};
use anyhow::{anyhow, bail, Context, Result};
use serde::Deserialize;
use strip_ansi_escapes::strip;
use unicode_width::UnicodeWidthStr;

/// Maximum number of lines to retain in the scrollback buffer.
const OUTPUT_MAX_LINES: usize = 500;

/// Path to the temp log file we rotate between runs.
pub fn log_file_path() -> PathBuf {
    env::temp_dir().join("codex_voice_tui.log")
}

/// Write debug messages to a temp file so we can troubleshoot without corrupting the TUI.
pub fn log_debug(msg: &str) {
    use std::fs::OpenOptions;

    let log_path = log_file_path();
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open(log_path) {
        let _ = writeln!(file, "[{timestamp}] {msg}");
    }
}

/// Remove the log file if it grows past 5 MB between runs.
pub fn init_debug_log_file() {
    let log_path = log_file_path();
    if let Ok(metadata) = fs::metadata(&log_path) {
        const MAX_BYTES: u64 = 5 * 1024 * 1024;
        if metadata.len() > MAX_BYTES {
            let _ = fs::remove_file(&log_path);
        }
    }
}

/// Return a UTF-8 safe preview string for debug logging.
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

/// Dispatch the current prompt to the persistent Codex session.
fn send_prompt(app: &mut App) -> Result<Option<Vec<String>>> {
    let prompt = app.input.trim().to_string();
    if prompt.is_empty() {
        app.status = "Nothing to send; prompt is empty.".into();
        return Ok(None);
    }

    app.status = "Calling Codex...".into();

    let codex_start = Instant::now();
    let mut used_persistent = false;
    let mut codex_result: Option<String> = None;

    // Try the persistent PTY session first so replies show up faster.
    if app.config.persistent_codex {
        log_debug("Attempting persistent Codex session");
        match call_codex_via_session(app, &prompt) {
            Ok(text) => {
                used_persistent = true;
                codex_result = Some(text);
            }
            Err(err) => {
                log_debug(&format!(
                    "Persistent Codex session failed: {err:#}. Falling back to one-shot CLI."
                ));
                app.codex_session = None;
                app.status =
                    "Persistent Codex session failed; falling back to single invocation.".into();
            }
        }
    }

    let codex_output = match codex_result {
        Some(text) => text,
        None => call_codex(&app.config, &prompt)?,
    };

    if app.config.log_timings {
        let elapsed = codex_start.elapsed().as_secs_f64();
        log_debug(&format!(
            "timing|phase=codex_cli|persistent={}|elapsed_s={:.3}|lines={}|chars={}",
            used_persistent,
            elapsed,
            codex_output.lines().count(),
            codex_output.len()
        ));
    }

    // Count lines before consuming codex_output
    let line_count = codex_output.lines().count();

    // The output from call_codex_via_session is already sanitized,
    // but call_codex might not be, so apply sanitization conditionally
    let sanitized_output = if used_persistent {
        // Already sanitized in call_codex_via_session
        codex_output
    } else {
        // Need to sanitize output from direct call_codex
        sanitize_pty_output(codex_output.as_bytes())
    };
    let sanitized_lines = prepare_for_display(&sanitized_output);

    let mut lines = Vec::new();
    // Ensure the prompt line is clean
    let prompt_line = format!("> {}", prompt.trim());
    log_debug(&format!(
        "send_prompt: Adding prompt line: {:?}",
        prompt_line
    ));
    lines.push(prompt_line);
    lines.push(String::new()); // Add blank line after prompt

    // Log the sanitized lines we're about to add
    for (idx, line) in sanitized_lines.iter().enumerate() {
        if !line.is_empty() {
            log_debug(&format!(
                "send_prompt: sanitized_line[{}] = {:?}",
                idx,
                preview_for_log(line, 50)
            ));
        }
    }

    lines.extend(sanitized_lines);
    lines.push(String::new()); // Add blank line after Codex response
    app.status = format!("Codex returned {} lines.", line_count);
    app.input.clear();

    if app.voice_enabled {
        if let Err(err) = app.start_voice_capture(VoiceCaptureTrigger::Auto) {
            app.status = format!("Voice capture failed after Codex call: {err:#}");
        }
    }

    Ok(Some(lines))
}

/// Run the Codex CLI with the same fallbacks the Python pipeline uses (PTY helper ->
/// direct interactive -> exec `-`) so tool calls behave consistently.
fn call_codex(config: &AppConfig, prompt: &str) -> Result<String> {
    // Prefer the PTY helper for full interactive support (streaming, tools, etc.).

    // Use the current working directory for Codex (where user actually is)
    let codex_working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    if config.pty_helper.exists() {
        match try_python_pty(config, prompt, &codex_working_dir)? {
            Some(PtyResult::Success(text)) => return Ok(text),
            Some(PtyResult::Failure(_msg)) => {
                // PTY failed, but continue to fallbacks
                // Don't use eprintln in TUI mode - it corrupts the display
            }
            None => {
                // PTY helper not available
            }
        }
    }

    // Fallback 1: spawn Codex directly; this keeps PTY-like behavior for most cases.
    let mut interactive_cmd = Command::new(&config.codex_cmd);
    interactive_cmd
        .args(&config.codex_args)
        .arg("-C")
        .arg(&codex_working_dir)
        .env("TERM", &config.term_value)
        .env("CODEX_NONINTERACTIVE", "1") // Hint to Codex that we're piping
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut interactive_child = interactive_cmd
        .spawn()
        .with_context(|| format!("failed to spawn interactive {}", config.codex_cmd))?;

    if let Some(mut stdin) = interactive_child.stdin.take() {
        write_prompt_with_newline(&mut stdin, prompt)
            .context("failed to write prompt to codex stdin")?;
    }

    let interactive_output = interactive_child
        .wait_with_output()
        .context("failed to wait for interactive codex process")?;

    if interactive_output.status.success() {
        return Ok(String::from_utf8_lossy(&interactive_output.stdout).to_string());
    }

    let interactive_stderr = String::from_utf8_lossy(&interactive_output.stderr).to_string();

    // Fallback 2: last resort is `codex exec -` which mirrors how the Python pipeline works.
    let mut exec_cmd = Command::new(&config.codex_cmd);
    exec_cmd
        .arg("exec")
        .arg("-C")
        .arg(&codex_working_dir)
        .args(&config.codex_args)
        .env("TERM", &config.term_value)
        .arg("-") // Read from stdin
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());

    let mut exec_child = exec_cmd
        .spawn()
        .with_context(|| format!("failed to spawn {} exec", config.codex_cmd))?;

    if let Some(mut stdin) = exec_child.stdin.take() {
        write_prompt_with_newline(&mut stdin, prompt)
            .context("failed to write prompt to codex exec stdin")?;
    }

    let exec_output = exec_child
        .wait_with_output()
        .context("failed to wait for codex exec process")?;

    if exec_output.status.success() {
        // Using codex exec mode as fallback
        return Ok(String::from_utf8_lossy(&exec_output.stdout).to_string());
    }

    bail!(
        "All codex invocation methods failed:\n\
           PTY Helper: Check if scripts/run_in_pty.py exists\n\
           Interactive: {}\n\
           Exec mode: {}",
        interactive_stderr.trim(),
        String::from_utf8_lossy(&exec_output.stderr).trim()
    )
}

/// Send the prompt through the persistent PTY session so Codex keeps its shell
/// state (tools, cwd, env vars) between prompts.
fn call_codex_via_session(app: &mut App, prompt: &str) -> Result<String> {
    app.ensure_codex_session()?;
    let session = app
        .codex_session
        .as_mut()
        .ok_or_else(|| anyhow!("Codex session failed to initialize"))?;
    session
        .send(prompt)
        .context("failed to write prompt to persistent Codex session")?;
    let mut combined_raw = Vec::new();
    let start_time = Instant::now();
    let overall_timeout = Duration::from_secs(30);
    // Treat 350 ms of silence as the end of the response.
    let quiet_grace = Duration::from_millis(350);
    let mut last_progress = Instant::now();
    let mut last_len = 0usize;

    loop {
        let output_chunks = session.read_output_timeout(Duration::from_millis(500));
        for chunk in output_chunks {
            // Log raw PTY bytes plus a short hexdump so we can trace stuck ANSI queries or prompts.
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
    bail!("persistent Codex session returned no text");
}

/// Render a small chunk of bytes as hex so the log shows what the PTY emitted (handy for CSI quirks).
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

/// Normalize CR/LF pairs and strip ANSI escape sequences so the scrollback only shows readable text.
fn sanitize_pty_output(raw: &[u8]) -> String {
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

fn prepare_for_display(text: &str) -> Vec<String> {
    text.lines().map(|line| line.to_string()).collect()
}

/// Helper for the various Codex invocations to guarantee the prompt ends in a newline.
fn write_prompt_with_newline<W: Write>(writer: &mut W, prompt: &str) -> io::Result<()> {
    writer.write_all(prompt.as_bytes())?;
    if !prompt.ends_with('\n') {
        writer.write_all(b"\n")?;
    }
    Ok(())
}

/// JSON payload emitted by the Python fallback pipeline; we parse it to reuse the
/// verified Python flow when native capture fails.
#[derive(Debug, Deserialize)]
pub(crate) struct PipelineJsonResult {
    pub(crate) transcript: String,
    #[allow(dead_code)]
    pub(crate) prompt: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub(crate) codex_output: Option<String>,
    #[serde(default)]
    pub(crate) metrics: PipelineMetrics,
}

/// Optional timing metadata emitted by the python helper, used for log summaries.
#[derive(Debug, Deserialize, Default, Clone, Copy)]
pub(crate) struct PipelineMetrics {
    #[serde(default)]
    pub(crate) record_s: f64,
    #[serde(default)]
    pub(crate) stt_s: f64,
    #[serde(default)]
    pub(crate) codex_s: f64,
    #[serde(default)]
    pub(crate) total_s: f64,
}

/// Execute the original python pipeline and parse its JSON result for STT fallback.
pub(crate) fn run_python_transcription(config: &AppConfig) -> Result<PipelineJsonResult> {
    let mut cmd = Command::new(&config.python_cmd);
    cmd.arg(&config.pipeline_script);
    cmd.args(["--seconds", &config.seconds.to_string()]);
    cmd.args(["--lang", &config.lang]);
    cmd.args(["--ffmpeg-cmd", &config.ffmpeg_cmd]);
    if let Some(device) = &config.ffmpeg_device {
        cmd.args(["--ffmpeg-device", device]);
    }
    cmd.args(["--whisper-cmd", &config.whisper_cmd]);
    cmd.args(["--whisper-model", &config.whisper_model]);
    if let Some(model_path) = &config.whisper_model_path {
        cmd.args(["--whisper-model-path", model_path]);
    }
    cmd.args(["--codex-cmd", &config.codex_cmd]);
    for arg in &config.codex_args {
        cmd.arg(format!("--codex-arg={arg}"));
    }
    cmd.arg("--no-codex");
    cmd.arg("--emit-json");
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());

    log_debug("Invoking python fallback for transcription");
    let call_started = Instant::now();
    let output = cmd
        .output()
        .context("failed to run python fallback pipeline")?;
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    if !output.status.success() {
        return Err(anyhow!(
            "python fallback failed with status {}.\nstdout:\n{}\nstderr:\n{}",
            output.status,
            stdout.trim(),
            stderr.trim()
        ));
    }

    let mut parsed: Option<PipelineJsonResult> = None;
    let mut last_parse_error: Option<(String, serde_json::Error)> = None;

    let stdout_trimmed = stdout.trim();
    if !stdout_trimmed.is_empty() {
        match serde_json::from_str::<PipelineJsonResult>(stdout_trimmed) {
            Ok(json) => parsed = Some(json),
            Err(err) => last_parse_error = Some((stdout_trimmed.to_string(), err)),
        }
    }

    if parsed.is_none() {
        for line in stdout.lines().rev() {
            let mut trimmed = line.trim();
            if let Some(rest) = trimmed.strip_prefix("JSON:") {
                trimmed = rest.trim();
            }
            if !(trimmed.starts_with('{') && trimmed.ends_with('}')) {
                continue;
            }
            match serde_json::from_str::<PipelineJsonResult>(trimmed) {
                Ok(json) => {
                    parsed = Some(json);
                    break;
                }
                Err(err) => {
                    last_parse_error = Some((trimmed.to_string(), err));
                }
            }
        }
    }

    let parsed = match parsed {
        Some(json) => {
            if let Some((line, err)) = last_parse_error {
                log_debug(&format!(
                    "Python fallback JSON parse warnings (last error: {err} on `{line}`)"
                ));
            }
            json
        }
        None => {
            let mut error = anyhow!(
                "python fallback did not emit JSON.\nstdout:\n{}\nstderr:\n{}",
                stdout.trim(),
                stderr.trim()
            );
            if let Some((line, parse_err)) = last_parse_error {
                error = error.context(format!("last JSON parse failure `{line}`: {parse_err}"));
            }
            return Err(error);
        }
    };
    if config.log_timings {
        let elapsed = call_started.elapsed().as_secs_f64();
        log_debug(&format!(
            "timing|phase=python_pipeline|record_s={:.3}|stt_s={:.3}|codex_s={:.3}|total_s={:.3}|rust_elapsed_s={:.3}",
            parsed.metrics.record_s,
            parsed.metrics.stt_s,
            parsed.metrics.codex_s,
            parsed.metrics.total_s,
            elapsed,
        ));
    }
    Ok(parsed)
}

/// Outcome of the Python PTY helper invocation.
enum PtyResult {
    Success(String),
    Failure(String),
}

/// Invoke the Python helper that wraps Codex in a pseudo-terminal, returning any output.
fn try_python_pty(
    config: &AppConfig,
    prompt: &str,
    working_dir: &Path,
) -> Result<Option<PtyResult>> {
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

    let mut child = cmd
        .spawn()
        .with_context(|| format!("failed to run PTY helper {}", config.pty_helper.display()))?;

    if let Some(mut stdin) = child.stdin.take() {
        let mut payload = prompt.as_bytes().to_vec();
        if !prompt.ends_with('\n') {
            payload.push(b'\n');
        }
        stdin
            .write_all(&payload)
            .context("failed writing prompt to PTY helper")?;
    }

    let output = child
        .wait_with_output()
        .context("failed waiting for PTY helper")?;

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

/// Central application state shared between the event loop, renderer, and voice worker.
pub struct App {
    config: AppConfig,
    input: String,
    output: Vec<String>,
    status: String,
    voice_enabled: bool,
    scroll_offset: u16,
    codex_session: Option<pty_session::PtyCodexSession>,
    audio_recorder: Option<Arc<Mutex<audio::Recorder>>>,
    transcriber: Option<Arc<Mutex<stt::Transcriber>>>,
    voice_job: Option<VoiceJob>,
}

impl App {
    /// Create the application state with default status text.
    pub fn new(config: AppConfig) -> Self {
        Self {
            config,
            input: String::new(),
            output: Vec::new(),
            status: "Ready. Press Ctrl+R for voice capture.".into(),
            voice_enabled: false,
            scroll_offset: 0,
            codex_session: None,
            audio_recorder: None,
            transcriber: None,
            voice_job: None,
        }
    }

    /// Create the audio recorder on first use so we only query the OS once.
    fn get_recorder(&mut self) -> Result<Arc<Mutex<audio::Recorder>>> {
        if self.audio_recorder.is_none() {
            let recorder = audio::Recorder::new(self.config.input_device.as_deref())?;
            self.audio_recorder = Some(Arc::new(Mutex::new(recorder)));
        }
        Ok(self
            .audio_recorder
            .as_ref()
            .expect("recorder initialized")
            .clone())
    }

    /// Load the Whisper model lazily because it is heavy and can take seconds.
    fn get_transcriber(&mut self) -> Result<Option<Arc<Mutex<stt::Transcriber>>>> {
        if self.transcriber.is_none() {
            let Some(model_path) = self.config.whisper_model_path.clone() else {
                return Ok(None);
            };
            let transcriber = stt::Transcriber::new(&model_path)?;
            self.transcriber = Some(Arc::new(Mutex::new(transcriber)));
        }
        Ok(self.transcriber.as_ref().cloned())
    }

    /// Ensure a long-lived PTY session exists so repeated prompts avoid Codex cold-start costs.
    fn ensure_codex_session(&mut self) -> Result<()> {
        if !self.config.persistent_codex {
            bail!("persistent Codex disabled via CLI flag");
        }
        if self.codex_session.is_none() {
            self.status = "Starting persistent Codex session...".into();
            let working_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

            match pty_session::PtyCodexSession::new(
                &self.config.codex_cmd,
                working_dir.to_str().unwrap_or("."),
                &self.config.codex_args,
                &self.config.term_value,
            ) {
                Ok(session) => {
                    self.codex_session = Some(session);
                    self.status = "Codex session ready (persistent).".into();
                    log_debug("PTY Codex session started successfully");
                }
                Err(e) => {
                    let msg = format!("Failed to start Codex PTY: {e}");
                    self.status = msg.clone();
                    log_debug(&msg);
                    bail!(msg);
                }
            }
        } else {
            // Check if session is still alive
            if let Some(ref session) = self.codex_session {
                if !session.is_alive() {
                    self.status = "Codex session died, restarting...".into();
                    self.codex_session = None;
                    self.ensure_codex_session()?;
                }
            }
        }
        Ok(())
    }

    /// Append output lines while trimming scrollback so the terminal stays snappy on long sessions.
    fn append_output(&mut self, lines: Vec<String>) {
        self.output.extend(lines);
        if self.output.len() > OUTPUT_MAX_LINES {
            let excess = self.output.len().saturating_sub(OUTPUT_MAX_LINES);
            self.output.drain(0..excess);
        }
        // Scroll near the bottom when new content arrives.
        let offset = self.output.len().saturating_sub(10).min(u16::MAX as usize);
        self.scroll_offset = offset as u16;
    }

    /// Start the background worker that records speech and pipes it through STT without blocking the UI.
    pub(crate) fn start_voice_capture(&mut self, trigger: VoiceCaptureTrigger) -> Result<bool> {
        if self.voice_job.is_some() {
            if trigger == VoiceCaptureTrigger::Manual {
                self.status = "Voice capture already running; please wait...".into();
            }
            return Ok(false);
        }

        let transcriber = self.get_transcriber()?;
        if transcriber.is_none() {
            log_debug(
                "No native Whisper model configured; using python fallback for voice capture.",
            );
            if self.config.no_python_fallback {
                let msg = "Native Whisper model not configured and --no-python-fallback is set.";
                self.status = msg.into();
                bail!(msg);
            }
        }
        let mut fallback_note: Option<String> = None;
        let recorder = if transcriber.is_some() {
            match self.get_recorder() {
                Ok(recorder) => Some(recorder),
                Err(err) => {
                    if self.config.no_python_fallback {
                        let msg = format!(
                            "Audio recorder unavailable and --no-python-fallback is set: {err:#}"
                        );
                        self.status = msg.clone();
                        bail!(msg);
                    }
                    log_debug(&format!(
                        "Audio recorder unavailable ({err:#}); falling back to python pipeline."
                    ));
                    fallback_note =
                        Some("Recorder unavailable; falling back to python pipeline.".into());
                    None
                }
            }
        } else {
            None
        };
        let using_native = transcriber.is_some() && recorder.is_some();
        let job = voice::start_voice_job(recorder, transcriber.clone(), self.config.clone());
        self.voice_job = Some(job);

        let pipeline_label = if using_native {
            "Rust pipeline"
        } else {
            "Python pipeline"
        };
        self.status = match trigger {
            VoiceCaptureTrigger::Manual => format!(
                "Recording voice for {} seconds... ({pipeline_label})",
                self.config.seconds,
                pipeline_label = pipeline_label
            ),
            VoiceCaptureTrigger::Auto => format!(
                "Recording voice for {} seconds... ({pipeline_label}, auto mode)",
                self.config.seconds,
                pipeline_label = pipeline_label
            ),
        };
        if let Some(note) = fallback_note {
            self.status.push(' ');
            self.status.push_str(&note);
        }
        Ok(true)
    }

    pub(crate) fn toggle_voice_mode(&mut self) -> Result<()> {
        self.voice_enabled = !self.voice_enabled;
        if self.voice_enabled {
            if !self.start_voice_capture(VoiceCaptureTrigger::Auto)? {
                self.voice_enabled = false;
                self.status = "Voice mode failed to start; disabled voice mode.".into();
            }
        } else {
            self.status = "Voice mode disabled.".into();
        }
        Ok(())
    }

    pub(crate) fn send_current_input(&mut self) -> Result<()> {
        if let Some(output) = send_prompt(self)? {
            self.append_output(output);
        }
        Ok(())
    }

    pub(crate) fn poll_voice_job(&mut self) -> Result<()> {
        // Check the worker channel without blocking the UI thread.
        let mut finished = false;
        let mut message_to_handle: Option<VoiceJobMessage> = None;
        if let Some(job) = self.voice_job.as_mut() {
            match job.receiver.try_recv() {
                Ok(message) => {
                    message_to_handle = Some(message);
                    finished = true;
                }
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => {
                    self.status = "Voice capture worker disconnected unexpectedly.".into();
                    finished = true;
                }
            }
            if finished {
                // Join the worker thread once it signals completion to avoid lingering handles.
                if let Some(handle) = job.handle.take() {
                    let _ = handle.join();
                }
            }
        }
        if let Some(message) = message_to_handle {
            self.handle_voice_job_message(message);
        }
        if finished {
            self.voice_job = None;
        }
        Ok(())
    }

    /// Update UI state based on whatever the voice worker reported (transcript, silence, or error).
    fn handle_voice_job_message(&mut self, message: VoiceJobMessage) {
        match message {
            VoiceJobMessage::Transcript { text, source } => {
                log_debug("Voice capture completed successfully");
                self.input = text;
                self.status = format!(
                    "Transcript captured ({}); edit and press Enter.",
                    source.label()
                );
            }
            VoiceJobMessage::Empty { source } => {
                log_debug("Voice capture detected no speech");
                self.input.clear();
                self.status = format!("No speech detected ({}). Try again.", source.label());
            }
            VoiceJobMessage::Error(err) => {
                log_debug(&format!("Voice capture worker error: {err}"));
                self.status = format!("Voice capture failed: {err}");
            }
        }
    }

    pub(crate) fn scroll_up(&mut self) {
        if self.scroll_offset > 0 {
            self.scroll_offset = self.scroll_offset.saturating_sub(1);
        }
    }

    pub(crate) fn scroll_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(1);
    }

    pub(crate) fn page_up(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_sub(10);
    }

    pub(crate) fn page_down(&mut self) {
        self.scroll_offset = self.scroll_offset.saturating_add(10);
    }

    pub(crate) fn scroll_to_top(&mut self) {
        self.scroll_offset = 0;
    }

    pub(crate) fn scroll_to_bottom(&mut self) {
        let offset = self.output.len().saturating_sub(10).min(u16::MAX as usize);
        self.scroll_offset = offset as u16;
    }

    pub(crate) fn input_text(&self) -> &str {
        &self.input
    }

    /// Returns the current input text for rendering in the UI.
    pub(crate) fn sanitized_input_text(&self) -> String {
        self.input.clone()
    }

    pub(crate) fn status_text(&self) -> &str {
        &self.status
    }

    pub(crate) fn output_lines(&self) -> &[String] {
        &self.output
    }

    pub(crate) fn get_scroll_offset(&self) -> u16 {
        self.scroll_offset
    }

    pub(crate) fn push_input_char(&mut self, ch: char) {
        self.input.push(ch);
    }

    pub(crate) fn backspace_input(&mut self) {
        self.input.pop();
    }

    pub(crate) fn clear_input(&mut self) {
        self.input.clear();
    }

    /// Keep the UI in sync with the persistent Codex PTY without blocking.
    pub(crate) fn drain_persistent_output(&mut self) {
        if let Some(ref session) = self.codex_session {
            // Collect PTY bytes and convert them into clean lines.
            let chunks = session.read_output();
            if !chunks.is_empty() {
                let mut raw = Vec::new();
                for chunk in chunks {
                    raw.extend_from_slice(&chunk);
                }
                let sanitized = sanitize_pty_output(&raw);
                if !sanitized.trim().is_empty() {
                    let lines = prepare_for_display(&sanitized);
                    if !lines.is_empty() {
                        self.append_output(lines);
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    fn test_config() -> AppConfig {
        AppConfig::parse_from(["codex-voice-tests"])
    }

    #[test]
    fn append_output_trims_history() {
        let config = test_config();
        let mut app = App::new(config);
        let lines = (0..600).map(|i| format!("line {i}")).collect();
        app.append_output(lines);
        assert!(app.output_lines().len() <= OUTPUT_MAX_LINES);
    }

    #[test]
    fn scroll_helpers_update_offset() {
        let config = test_config();
        let mut app = App::new(config);
        app.append_output(vec!["one".into(), "two".into(), "three".into()]);
        app.page_down();
        assert_eq!(app.get_scroll_offset(), 10);
        app.scroll_to_top();
        assert_eq!(app.get_scroll_offset(), 0);
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
}
