//! Command-line parsing and validation helpers.

use anyhow::{anyhow, bail, Context, Result};
use clap::{ArgAction, Parser};
use std::{
    env, fs,
    path::{Path, PathBuf},
};

const MAX_CODEX_ARGS: usize = 64;
const MAX_CODEX_ARG_BYTES: usize = 8 * 1024;
const ISO_639_1_CODES: &[&str] = &[
    "af", "am", "ar", "az", "be", "bg", "bn", "bs", "ca", "cs", "cy", "da", "de", "el", "en", "es",
    "et", "eu", "fa", "fi", "fil", "fr", "ga", "gl", "gu", "he", "hi", "hr", "hu", "hy", "id",
    "is", "it", "ja", "jv", "ka", "kk", "km", "kn", "ko", "lo", "lt", "lv", "mk", "ml", "mn", "mr",
    "ms", "my", "ne", "nl", "no", "pa", "pl", "pt", "ro", "ru", "si", "sk", "sl", "sq", "sr", "sv",
    "sw", "ta", "te", "th", "tr", "uk", "ur", "vi", "zh",
];
// FFmpeg devices are passed to the shell, so strip characters that would let users sneak commands in.
const FORBIDDEN_DEVICE_CHARS: &[char] = &[';', '|', '&', '$', '`', '<', '>', '\\', '\'', '"'];
const DEFAULT_PIPELINE_SCRIPT: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../codex_voice.py");
const DEFAULT_PTY_HELPER: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/run_in_pty.py");

/// CLI options for the Codex Voice TUI. Validated values keep downstream subprocesses safe.
#[derive(Debug, Parser, Clone)]
#[command(about = "Codex Voice TUI", author, version)]
pub struct AppConfig {
    /// Path to the Codex CLI binary
    #[arg(long, default_value = "codex")]
    pub codex_cmd: String,

    /// Extra arguments to pass to the Codex CLI (repeatable)
    #[arg(long = "codex-arg", action = ArgAction::Append, value_name = "ARG")]
    pub codex_args: Vec<String>,

    /// Path to the python interpreter used for helper scripts
    #[arg(long, default_value = "python3")]
    pub python_cmd: String,

    /// Pipeline script location
    #[arg(long, default_value = DEFAULT_PIPELINE_SCRIPT)]
    pub pipeline_script: PathBuf,

    /// TERM value exported to Codex
    #[arg(long = "term", default_value_t = default_term())]
    pub term_value: String,

    /// PTY helper script path
    #[arg(long, default_value = DEFAULT_PTY_HELPER)]
    pub pty_helper: PathBuf,

    /// Preferred audio input device name
    #[arg(long)]
    pub input_device: Option<String>,

    /// Print detected audio input devices and exit
    #[arg(long = "list-input-devices", default_value_t = false)]
    pub list_input_devices: bool,

    /// Disable persistent Codex session
    #[arg(long = "no-persistent-codex", action = ArgAction::SetFalse, default_value_t = true)]
    pub persistent_codex: bool,

    /// Enable verbose timing logs
    #[arg(long)]
    pub log_timings: bool,

    /// Path to whisper executable
    #[arg(long, default_value = "whisper")]
    pub whisper_cmd: String,

    /// Whisper model name
    #[arg(long, default_value = "small")]
    pub whisper_model: String,

    /// Whisper model path (required for whisper.cpp)
    #[arg(long)]
    pub whisper_model_path: Option<String>,

    /// FFmpeg binary location
    #[arg(long, default_value = "ffmpeg")]
    pub ffmpeg_cmd: String,

    /// FFmpeg audio device override
    #[arg(long)]
    pub ffmpeg_device: Option<String>,

    /// Recording duration in seconds
    #[arg(long, default_value_t = 5)]
    pub seconds: u64,

    /// Language passed to Whisper
    #[arg(long, default_value = "en")]
    pub lang: String,

    /// Fail instead of using the python STT fallback
    #[arg(long = "no-python-fallback")]
    pub no_python_fallback: bool,
}

impl AppConfig {
    /// Parse CLI arguments and validate them right away.
    pub fn parse_args() -> Result<Self> {
        let mut config = Self::parse();
        config.validate()?;
        Ok(config)
    }

    /// Check CLI values and normalize paths.
    pub(crate) fn validate(&mut self) -> Result<()> {
        const MIN_RECORD_SECONDS: u64 = 1;
        const MAX_RECORD_SECONDS: u64 = 60;
        let repo_root = canonical_repo_root()?;

        if !(MIN_RECORD_SECONDS..=MAX_RECORD_SECONDS).contains(&self.seconds) {
            bail!(
                "--seconds must be between {MIN_RECORD_SECONDS} and {MAX_RECORD_SECONDS}, got {}",
                self.seconds
            );
        }

        self.codex_cmd = sanitize_binary(&self.codex_cmd, "--codex-cmd", &["codex"])?;
        self.python_cmd =
            sanitize_binary(&self.python_cmd, "--python-cmd", &["python3", "python"])?;
        self.ffmpeg_cmd = sanitize_binary(&self.ffmpeg_cmd, "--ffmpeg-cmd", &["ffmpeg"])?;
        self.whisper_cmd = sanitize_binary(
            &self.whisper_cmd,
            "--whisper-cmd",
            &["whisper", "whisper.cpp"],
        )?;

        // Keep helper scripts inside this repo.
        self.pipeline_script =
            canonicalize_within_repo(&self.pipeline_script, "pipeline script", &repo_root)?;
        self.pty_helper = canonicalize_within_repo(&self.pty_helper, "pty helper", &repo_root)?;

        if self.whisper_model_path.is_none() {
            if let Some(auto_model) =
                discover_default_whisper_model(&repo_root, &self.whisper_model)
            {
                self.whisper_model_path = Some(auto_model.to_string_lossy().to_string());
            }
        }

        // If a model path was supplied (explicitly or via auto-detect), make sure it exists.
        if let Some(model) = &self.whisper_model_path {
            let model_path = Path::new(model);
            if !model_path.exists() {
                bail!(
                    "whisper model path '{}' does not exist",
                    model_path.display()
                );
            }
        }

        if let Some(model) = &mut self.whisper_model_path {
            // Store a canonical absolute path for subprocesses.
            let canonical = Path::new(model)
                .canonicalize()
                .with_context(|| format!("failed to canonicalize whisper model path '{model}'"))?;
            *model = canonical
                .to_str()
                .map(|s| s.to_string())
                .ok_or_else(|| anyhow!("whisper model path must be valid UTF-8"))?;
        }

        if self.lang.trim().is_empty()
            || !self
                .lang
                .chars()
                .all(|ch| ch.is_ascii_alphabetic() || ch == '-' || ch == '_')
        {
            bail!("--lang must contain only alphabetic characters or '-'/'_' separators");
        }
        // Allow locale-style values but only check the leading ISO-639-1 code.
        let lang_primary = self
            .lang
            .split(['-', '_'])
            .next()
            .unwrap_or("")
            .to_ascii_lowercase();
        if !ISO_639_1_CODES.contains(&lang_primary.as_str()) {
            bail!(
                "--lang must start with a valid ISO-639-1 code, got '{}'",
                self.lang
            );
        }

        // Avoid huge argument lists when forwarding to Codex.
        if self.codex_args.len() > MAX_CODEX_ARGS {
            bail!(
                "--codex-arg repeated too many times (max {MAX_CODEX_ARGS}, got {})",
                self.codex_args.len()
            );
        }
        // Also limit the total byte length to keep argv small.
        let total_arg_bytes: usize = self.codex_args.iter().map(|arg| arg.len()).sum();
        if total_arg_bytes > MAX_CODEX_ARG_BYTES {
            bail!("combined --codex-arg length exceeds {MAX_CODEX_ARG_BYTES} bytes");
        }

        // The FFmpeg device string is passed straight to the shell, so keep it simple.
        if let Some(device) = &self.ffmpeg_device {
            if device.len() > 256
                || device.chars().any(|ch| matches!(ch, '\n' | '\r'))
                || device
                    .chars()
                    .any(|ch| FORBIDDEN_DEVICE_CHARS.contains(&ch))
            {
                bail!(
                    "--ffmpeg-device must be <=256 characters with no control or shell metacharacters"
                );
            }
        }

        Ok(())
    }
}

/// Use the user's TERM if set; default to xterm-256color otherwise.
fn default_term() -> String {
    env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string())
}

/// Resolve the repository root by walking up from the Cargo manifest.
fn canonical_repo_root() -> Result<PathBuf> {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.parent().unwrap_or(manifest_dir).to_path_buf();
    repo_root
        .canonicalize()
        .with_context(|| format!("failed to canonicalize repo root '{}'", repo_root.display()))
}

/// Canonicalize a path and ensure it still lives under the repo root.
fn canonicalize_within_repo(path: &Path, label: &str, repo_root: &Path) -> Result<PathBuf> {
    let canonical = path
        .canonicalize()
        .with_context(|| format!("failed to canonicalize {label} '{}'", path.display()))?;
    if !canonical.starts_with(repo_root) {
        bail!(
            "{label} '{}' must reside within {}",
            canonical.display(),
            repo_root.display()
        );
    }
    Ok(canonical)
}

/// Try to locate a ggml model in the repo's `models/` directory so the Rust pipeline
/// works out-of-the-box when users haven't provided --whisper-model-path.
fn discover_default_whisper_model(repo_root: &Path, whisper_model: &str) -> Option<PathBuf> {
    let models_dir = repo_root.join("models");
    if !models_dir.exists() {
        return None;
    }

    let mut candidates = Vec::new();
    candidates.push(models_dir.join(format!("ggml-{whisper_model}.en.bin")));
    candidates.push(models_dir.join(format!("ggml-{whisper_model}.bin")));
    candidates.push(models_dir.join("ggml-base.en.bin"));
    candidates.push(models_dir.join("ggml-base.bin"));

    for candidate in candidates {
        if candidate.exists() {
            if let Ok(canonical) = candidate.canonicalize() {
                return Some(canonical);
            }
        }
    }

    None
}

/// Allow either a known binary name or an absolute path.
fn sanitize_binary(value: &str, flag: &str, allowlist: &[&str]) -> Result<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        bail!("{flag} cannot be empty");
    }
    if let Some(allowed) = allowlist
        .iter()
        .find(|candidate| candidate.eq_ignore_ascii_case(trimmed))
    {
        return Ok((*allowed).to_string());
    }

    let path = Path::new(trimmed);
    if path.is_absolute() || trimmed.contains(std::path::MAIN_SEPARATOR) {
        let canonical = path
            .canonicalize()
            .with_context(|| format!("failed to canonicalize {flag} '{trimmed}'"))?;
        let metadata = fs::metadata(&canonical)
            .with_context(|| format!("failed to inspect {flag} '{}'", canonical.display()))?;
        if !metadata.is_file() {
            bail!("{flag} '{}' is not a file", canonical.display());
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mode = metadata.permissions().mode();
            if mode & 0o111 == 0 {
                bail!(
                    "{flag} '{}' exists but is not executable (mode {:o})",
                    canonical.display(),
                    mode
                );
            }
        }
        return canonical
            .to_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow!("{flag} must be valid UTF-8"));
    }

    bail!("{flag} must be one of {allowlist:?} or an existing binary path");
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn rejects_seconds_out_of_bounds() {
        let mut cfg = AppConfig::parse_from(["test-app", "--seconds", "0"]);
        assert!(cfg.validate().is_err());

        let mut cfg = AppConfig::parse_from(["test-app", "--seconds", "61"]);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn rejects_invalid_language_code() {
        let mut cfg = AppConfig::parse_from(["test-app", "--lang", "en$"]);
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn accepts_valid_defaults() {
        let mut cfg = AppConfig::parse_from(["test-app"]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn rejects_ffmpeg_device_with_shell_metacharacters() {
        for dangerous in [
            "default;rm -rf /",
            "mix|pipe",
            "out & more",
            "name$VAR",
            "quote\"",
            "single'",
            "newline\nbreak",
            "back\\slash",
        ] {
            let mut cfg = AppConfig::parse_from(["test-app", "--ffmpeg-device", dangerous]);
            assert!(
                cfg.validate().is_err(),
                "device '{dangerous}' should be rejected"
            );
        }
    }

    #[cfg(unix)]
    #[test]
    fn codex_cmd_path_must_be_executable() {
        use std::os::unix::fs::PermissionsExt;

        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis();
        let temp_path = env::temp_dir().join(format!("codex_cmd_test_{unique}"));
        fs::write(&temp_path, "#!/bin/sh\necho test\n").unwrap();
        let mut perms = fs::metadata(&temp_path).unwrap().permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&temp_path, perms.clone()).unwrap();

        let mut cfg =
            AppConfig::parse_from(["test-app", "--codex-cmd", temp_path.to_str().unwrap()]);
        assert!(
            cfg.validate().is_err(),
            "non-executable binary path should be rejected"
        );

        perms.set_mode(0o700);
        fs::set_permissions(&temp_path, perms).unwrap();
        let mut cfg =
            AppConfig::parse_from(["test-app", "--codex-cmd", temp_path.to_str().unwrap()]);
        assert!(
            cfg.validate().is_ok(),
            "executable binary path should be accepted"
        );

        let _ = fs::remove_file(&temp_path);
    }
}
