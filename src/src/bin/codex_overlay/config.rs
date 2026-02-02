use clap::{Parser, ValueEnum};
use std::path::{Path, PathBuf};
use voxterm::backend::BackendRegistry;
use voxterm::config::AppConfig;

use crate::color_mode::ColorMode;
use crate::theme::Theme;

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub(crate) enum VoiceSendMode {
    #[default]
    Auto,
    Insert,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub(crate) enum HudRightPanel {
    #[default]
    Ribbon,
    Dots,
    Chips,
    Off,
}

/// HUD display style - controls overall banner visibility.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum, Default)]
pub(crate) enum HudStyle {
    /// Full 4-row banner with borders and shortcuts (default)
    #[default]
    Full,
    /// Single-line minimal indicator (just colored text, no borders)
    Minimal,
    /// Hidden unless recording
    Hidden,
}

impl std::fmt::Display for HudStyle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            HudStyle::Full => "Full",
            HudStyle::Minimal => "Minimal",
            HudStyle::Hidden => "Hidden",
        };
        write!(f, "{label}")
    }
}

impl std::fmt::Display for HudRightPanel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            HudRightPanel::Off => "Off",
            HudRightPanel::Ribbon => "Ribbon",
            HudRightPanel::Dots => "Dots",
            HudRightPanel::Chips => "Chips",
        };
        write!(f, "{label}")
    }
}

#[derive(Debug, Parser, Clone)]
#[command(about = "VoxTerm", author, version)]
pub(crate) struct OverlayConfig {
    #[command(flatten)]
    pub(crate) app: AppConfig,

    /// Regex used to detect the AI prompt line (overrides auto-detection)
    #[arg(long = "prompt-regex")]
    pub(crate) prompt_regex: Option<String>,

    /// Log file path for prompt detection diagnostics
    #[arg(long = "prompt-log")]
    pub(crate) prompt_log: Option<PathBuf>,

    /// Start in auto-voice mode
    #[arg(long = "auto-voice", default_value_t = false)]
    pub(crate) auto_voice: bool,

    /// Idle time before auto-voice triggers when prompt detection is unknown (ms)
    #[arg(long = "auto-voice-idle-ms", default_value_t = 1200)]
    pub(crate) auto_voice_idle_ms: u64,

    /// Idle time before transcripts auto-send when a prompt has not been detected (ms)
    #[arg(long = "transcript-idle-ms", default_value_t = 250)]
    pub(crate) transcript_idle_ms: u64,

    /// Voice transcript handling (auto = send newline, insert = leave for editing)
    #[arg(long = "voice-send-mode", value_enum, default_value_t = VoiceSendMode::Auto)]
    pub(crate) voice_send_mode: VoiceSendMode,

    /// Color theme for status line (coral, catppuccin, dracula, nord, ansi, none)
    #[arg(long = "theme", default_value = "coral")]
    pub(crate) theme_name: Option<String>,

    /// Disable colors in status line output
    #[arg(long = "no-color", default_value_t = false)]
    pub(crate) no_color: bool,

    /// Right-side HUD panel (off, ribbon, dots, chips)
    #[arg(long = "hud-right-panel", value_enum, default_value_t = HudRightPanel::Ribbon)]
    pub(crate) hud_right_panel: HudRightPanel,

    /// Only animate the right-side panel while recording
    #[arg(long = "hud-right-panel-recording-only", default_value_t = true)]
    pub(crate) hud_right_panel_recording_only: bool,

    /// HUD display style (full, minimal, hidden)
    #[arg(long = "hud-style", value_enum, default_value_t = HudStyle::Full)]
    pub(crate) hud_style: HudStyle,

    /// Shorthand for --hud-style minimal
    #[arg(long = "minimal-hud", default_value_t = false)]
    pub(crate) minimal_hud: bool,

    /// Backend CLI to run (codex, claude, gemini, or custom command)
    ///
    /// Use a preset name or provide a custom command string.
    /// Examples:
    ///   --backend codex
    ///   --backend claude
    ///   --backend "my-tool --flag"
    #[arg(long = "backend", default_value = "codex")]
    pub(crate) backend: String,

    /// Shorthand for --backend codex
    #[arg(long = "codex", default_value_t = false)]
    pub(crate) codex: bool,

    /// Shorthand for --backend claude
    #[arg(long = "claude", default_value_t = false)]
    pub(crate) claude: bool,

    /// Shorthand for --backend gemini
    #[arg(long = "gemini", default_value_t = false)]
    pub(crate) gemini: bool,
}

pub(crate) struct ResolvedBackend {
    pub(crate) command: String,
    pub(crate) args: Vec<String>,
    pub(crate) label: String,
    pub(crate) prompt_pattern: Option<String>,
}

impl OverlayConfig {
    /// Get the resolved theme, respecting --no-color/NO_COLOR and backend defaults.
    pub(crate) fn theme_for_backend(&self, backend_label: &str) -> Theme {
        if self.no_color || std::env::var("NO_COLOR").is_ok() {
            return Theme::None;
        }
        let requested = self
            .theme_name
            .as_deref()
            .and_then(Theme::from_name)
            .unwrap_or_else(|| default_theme_for_backend(backend_label));
        let mode = self.color_mode();
        if !mode.supports_color() {
            Theme::None
        } else if !mode.supports_truecolor() {
            requested.fallback_for_ansi()
        } else {
            requested
        }
    }

    /// Get the detected color mode for the terminal.
    pub(crate) fn color_mode(&self) -> ColorMode {
        if self.no_color {
            ColorMode::None
        } else {
            ColorMode::detect()
        }
    }

    /// Resolve the backend command, arguments, and prompt patterns.
    pub(crate) fn resolve_backend(&self) -> ResolvedBackend {
        // Check shorthand flags first
        let backend_raw = if self.claude {
            "claude"
        } else if self.gemini {
            "gemini"
        } else if self.codex {
            "codex"
        } else {
            self.backend.trim()
        };
        let (primary, extra_args) = split_backend_command(backend_raw);
        let primary_label = extract_binary_label(&primary);

        if primary.is_empty() || primary_label.eq_ignore_ascii_case("codex") {
            let mut args = self.app.codex_args.clone();
            args.extend(extra_args);
            let command = if is_path_like(&primary) {
                primary
            } else {
                self.app.codex_cmd.clone()
            };
            return ResolvedBackend {
                command,
                args,
                label: "codex".to_string(),
                prompt_pattern: None,
            };
        }

        let registry = BackendRegistry::new();
        if let Some(backend) = registry.get(&primary_label) {
            let mut command_parts = backend.command();
            let default_cmd = command_parts
                .first()
                .cloned()
                .unwrap_or_else(|| primary.clone());
            let mut args: Vec<String> = if command_parts.len() > 1 {
                command_parts.drain(1..).collect()
            } else {
                Vec::new()
            };
            args.extend(extra_args);
            let command = if is_path_like(&primary) {
                primary
            } else {
                default_cmd
            };
            let prompt_pattern = if backend.prompt_pattern().trim().is_empty() {
                None
            } else {
                Some(backend.prompt_pattern().to_string())
            };
            return ResolvedBackend {
                command,
                args,
                label: backend.name().to_string(),
                prompt_pattern,
            };
        }

        ResolvedBackend {
            command: primary.clone(),
            args: extra_args,
            label: primary_label.to_lowercase(),
            prompt_pattern: None,
        }
    }
}

fn default_theme_for_backend(backend_label: &str) -> Theme {
    match backend_label.to_lowercase().as_str() {
        "claude" => Theme::Claude,
        "codex" => Theme::Codex,
        _ => Theme::Coral,
    }
}

fn split_backend_command(raw: &str) -> (String, Vec<String>) {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return (String::new(), Vec::new());
    }
    let parts = shell_words::split(trimmed)
        .unwrap_or_else(|_| trimmed.split_whitespace().map(|s| s.to_string()).collect());
    if parts.is_empty() {
        return (String::new(), Vec::new());
    }
    let command = parts[0].to_string();
    let args = parts[1..].to_vec();
    (command, args)
}

fn extract_binary_label(command: &str) -> String {
    if command.trim().is_empty() {
        return String::new();
    }
    Path::new(command)
        .file_name()
        .and_then(|s| s.to_str())
        .unwrap_or(command)
        .to_string()
}

fn is_path_like(command: &str) -> bool {
    let trimmed = command.trim();
    if trimmed.is_empty() {
        return false;
    }
    let path = Path::new(trimmed);
    path.is_absolute() || trimmed.contains(std::path::MAIN_SEPARATOR)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config_with_backend(backend: &str) -> OverlayConfig {
        OverlayConfig {
            app: AppConfig::parse_from(["test"]),
            prompt_regex: None,
            prompt_log: None,
            auto_voice: false,
            auto_voice_idle_ms: 1200,
            transcript_idle_ms: 250,
            voice_send_mode: VoiceSendMode::Auto,
            theme_name: None,
            no_color: false,
            hud_right_panel: HudRightPanel::Ribbon,
            hud_right_panel_recording_only: true,
            hud_style: HudStyle::Full,
            minimal_hud: false,
            backend: backend.to_string(),
            codex: false,
            claude: false,
            gemini: false,
        }
    }

    #[test]
    fn resolve_backend_preset_codex_uses_app_config() {
        let mut config = make_config_with_backend("codex");
        config.app.codex_cmd = "codex-bin".to_string();
        config.app.codex_args = vec!["--flag".to_string()];
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "codex-bin");
        assert_eq!(resolved.args, vec!["--flag"]);
        assert_eq!(resolved.label, "codex");
        assert!(resolved.prompt_pattern.is_none());
    }

    #[test]
    fn resolve_backend_codex_includes_extra_args() {
        let mut config = make_config_with_backend("codex --extra");
        config.app.codex_args = vec!["--flag".to_string()];
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "codex");
        assert_eq!(resolved.args, vec!["--flag", "--extra"]);
    }

    #[test]
    fn resolve_backend_preset_claude() {
        let config = make_config_with_backend("claude");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "claude");
        assert!(resolved.args.is_empty());
        assert_eq!(resolved.label, "claude");
        assert!(resolved.prompt_pattern.is_some());
    }

    #[test]
    fn resolve_backend_preset_gemini() {
        let config = make_config_with_backend("gemini");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "gemini");
        assert!(resolved.args.is_empty());
        assert_eq!(resolved.label, "gemini");
        assert!(resolved.prompt_pattern.is_some());
    }

    #[test]
    fn resolve_backend_preset_aider() {
        let config = make_config_with_backend("aider");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "aider");
        assert!(resolved.args.is_empty());
        assert_eq!(resolved.label, "aider");
        assert!(resolved.prompt_pattern.is_some());
    }

    #[test]
    fn resolve_backend_preset_opencode() {
        let config = make_config_with_backend("opencode");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "opencode");
        assert!(resolved.args.is_empty());
        assert_eq!(resolved.label, "opencode");
        assert!(resolved.prompt_pattern.is_some());
    }

    #[test]
    fn resolve_backend_preset_case_insensitive() {
        let config = make_config_with_backend("CLAUDE");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "claude");
        assert!(resolved.args.is_empty());

        let config = make_config_with_backend("Gemini");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "gemini");
        assert!(resolved.args.is_empty());
    }

    #[test]
    fn resolve_backend_custom_command() {
        let config = make_config_with_backend("my-tool");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "my-tool");
        assert!(resolved.args.is_empty());
        assert_eq!(resolved.label, "my-tool");
        assert!(resolved.prompt_pattern.is_none());
    }

    #[test]
    fn resolve_backend_custom_command_with_args() {
        let config = make_config_with_backend("my-tool --flag value");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "my-tool");
        assert_eq!(resolved.args, vec!["--flag", "value"]);
    }

    #[test]
    fn resolve_backend_custom_command_with_quoted_args() {
        let config = make_config_with_backend("my-tool --flag \"value with spaces\"");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "my-tool");
        assert_eq!(resolved.args, vec!["--flag", "value with spaces"]);
    }

    #[test]
    fn resolve_backend_custom_path() {
        let config = make_config_with_backend("/usr/local/bin/my-tool --verbose");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "/usr/local/bin/my-tool");
        assert_eq!(resolved.args, vec!["--verbose"]);
        assert_eq!(resolved.label, "my-tool");
    }

    #[test]
    fn resolve_backend_empty_fallback() {
        let config = make_config_with_backend("   ");
        let resolved = config.resolve_backend();
        assert_eq!(resolved.command, "codex");
        assert!(resolved.args.is_empty());
        assert_eq!(resolved.label, "codex");
    }
}
