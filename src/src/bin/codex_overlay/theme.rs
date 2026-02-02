//! Color themes for the overlay status line.
//!
//! Provides predefined color palettes that can be selected via CLI flags.

use std::env;

/// Border character set for drawing boxes.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BorderSet {
    pub top_left: char,
    pub top_right: char,
    pub bottom_left: char,
    pub bottom_right: char,
    pub horizontal: char,
    pub vertical: char,
    pub t_left: char,   // ├
    pub t_right: char,  // ┤
    pub t_top: char,    // ┬
    pub t_bottom: char, // ┴
}

/// Standard single-line borders
pub const BORDER_SINGLE: BorderSet = BorderSet {
    top_left: '┌',
    top_right: '┐',
    bottom_left: '└',
    bottom_right: '┘',
    horizontal: '─',
    vertical: '│',
    t_left: '├',
    t_right: '┤',
    t_top: '┬',
    t_bottom: '┴',
};

/// Double-line borders (elegant)
pub const BORDER_DOUBLE: BorderSet = BorderSet {
    top_left: '╔',
    top_right: '╗',
    bottom_left: '╚',
    bottom_right: '╝',
    horizontal: '═',
    vertical: '║',
    t_left: '╠',
    t_right: '╣',
    t_top: '╦',
    t_bottom: '╩',
};

/// Heavy/bold borders
pub const BORDER_HEAVY: BorderSet = BorderSet {
    top_left: '┏',
    top_right: '┓',
    bottom_left: '┗',
    bottom_right: '┛',
    horizontal: '━',
    vertical: '┃',
    t_left: '┣',
    t_right: '┫',
    t_top: '┳',
    t_bottom: '┻',
};

/// Rounded corners (modern)
pub const BORDER_ROUNDED: BorderSet = BorderSet {
    top_left: '╭',
    top_right: '╮',
    bottom_left: '╰',
    bottom_right: '╯',
    horizontal: '─',
    vertical: '│',
    t_left: '├',
    t_right: '┤',
    t_top: '┬',
    t_bottom: '┴',
};

/// Minimal dotted borders (reserved for future themes)
#[allow(dead_code)]
pub const BORDER_DOTTED: BorderSet = BorderSet {
    top_left: '·',
    top_right: '·',
    bottom_left: '·',
    bottom_right: '·',
    horizontal: '·',
    vertical: '·',
    t_left: '·',
    t_right: '·',
    t_top: '·',
    t_bottom: '·',
};

/// No borders (spaces) (reserved for future themes)
#[allow(dead_code)]
pub const BORDER_NONE: BorderSet = BorderSet {
    top_left: ' ',
    top_right: ' ',
    bottom_left: ' ',
    bottom_right: ' ',
    horizontal: ' ',
    vertical: ' ',
    t_left: ' ',
    t_right: ' ',
    t_top: ' ',
    t_bottom: ' ',
};

/// ANSI color codes for a theme.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ThemeColors {
    /// Color for recording/active states
    pub recording: &'static str,
    /// Color for processing/working states
    pub processing: &'static str,
    /// Color for success states
    pub success: &'static str,
    /// Color for warning states
    pub warning: &'static str,
    /// Color for error states
    pub error: &'static str,
    /// Color for info states
    pub info: &'static str,
    /// Reset code
    pub reset: &'static str,
    /// Dim/muted text for secondary info
    pub dim: &'static str,
    /// Primary background color (for main status area)
    pub bg_primary: &'static str,
    /// Secondary background color (for shortcuts row)
    pub bg_secondary: &'static str,
    /// Border/frame color
    pub border: &'static str,
    /// Border character set
    pub borders: BorderSet,
    /// Mode indicator symbol
    pub indicator_rec: &'static str,
    pub indicator_auto: &'static str,
    pub indicator_manual: &'static str,
    pub indicator_idle: &'static str,
}

/// Available color themes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Theme {
    /// Default coral/red theme (matches existing TUI)
    #[default]
    Coral,
    /// Claude warm neutral theme (Anthropic-inspired)
    Claude,
    /// Codex cool blue theme (neutral, OpenAI-style)
    Codex,
    /// Catppuccin Mocha - pastel dark theme
    Catppuccin,
    /// Dracula - high contrast dark theme
    Dracula,
    /// Nord - arctic blue-gray theme
    Nord,
    /// ANSI 16-color fallback for older terminals
    Ansi,
    /// No colors - plain text
    None,
}

impl Theme {
    /// Parse theme name from string.
    pub fn from_name(name: &str) -> Option<Self> {
        match name.to_lowercase().as_str() {
            "coral" | "default" => Some(Self::Coral),
            "claude" | "anthropic" => Some(Self::Claude),
            "codex" => Some(Self::Codex),
            "catppuccin" | "mocha" => Some(Self::Catppuccin),
            "dracula" => Some(Self::Dracula),
            "nord" => Some(Self::Nord),
            "ansi" | "ansi16" | "basic" => Some(Self::Ansi),
            "none" | "plain" => Some(Self::None),
            _ => None,
        }
    }

    /// Get the color palette for this theme.
    pub fn colors(&self) -> ThemeColors {
        let mut colors = match self {
            Self::Coral => THEME_CORAL,
            Self::Claude => THEME_CLAUDE,
            Self::Codex => THEME_CODEX,
            Self::Catppuccin => THEME_CATPPUCCIN,
            Self::Dracula => THEME_DRACULA,
            Self::Nord => THEME_NORD,
            Self::Ansi => THEME_ANSI,
            Self::None => THEME_NONE,
        };
        if is_warp_terminal() {
            colors.bg_primary = "";
            colors.bg_secondary = "";
        }
        colors
    }

    /// List all available theme names.
    #[allow(dead_code)]
    pub fn available() -> &'static [&'static str] {
        &[
            "claude",
            "codex",
            "coral",
            "catppuccin",
            "dracula",
            "nord",
            "ansi",
            "none",
        ]
    }

    /// Check if this theme uses truecolor (24-bit RGB).
    pub fn is_truecolor(&self) -> bool {
        matches!(
            self,
            Self::Claude | Self::Codex | Self::Catppuccin | Self::Dracula | Self::Nord
        )
    }

    /// Get a fallback theme for terminals without truecolor support.
    pub fn fallback_for_ansi(&self) -> Self {
        if self.is_truecolor() {
            Self::Ansi
        } else {
            *self
        }
    }
}

fn is_warp_terminal() -> bool {
    env::var("TERM_PROGRAM")
        .map(|value| value.to_lowercase().contains("warp"))
        .unwrap_or(false)
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Coral => write!(f, "coral"),
            Self::Claude => write!(f, "claude"),
            Self::Codex => write!(f, "codex"),
            Self::Catppuccin => write!(f, "catppuccin"),
            Self::Dracula => write!(f, "dracula"),
            Self::Nord => write!(f, "nord"),
            Self::Ansi => write!(f, "ansi"),
            Self::None => write!(f, "none"),
        }
    }
}

/// Coral theme - warm red/coral accents (default)
/// Uses transparent backgrounds for best compatibility across terminals
pub const THEME_CORAL: ThemeColors = ThemeColors {
    recording: "\x1b[91m",  // Bright red
    processing: "\x1b[93m", // Bright yellow
    success: "\x1b[92m",    // Bright green
    warning: "\x1b[93m",    // Bright yellow
    error: "\x1b[91m",      // Bright red
    info: "\x1b[94m",       // Bright blue
    reset: "\x1b[0m",
    dim: "\x1b[90m",    // Dark gray (not dim attribute - cleaner look)
    bg_primary: "",     // Transparent
    bg_secondary: "",   // Transparent
    border: "\x1b[91m", // Coral/red borders
    borders: BORDER_SINGLE,
    indicator_rec: "●",
    indicator_auto: "◉",
    indicator_manual: "●",
    indicator_idle: "○",
};

/// Claude theme - warm neutrals (Anthropic-inspired palette)
/// Uses transparent backgrounds for best compatibility across terminals
pub const THEME_CLAUDE: ThemeColors = ThemeColors {
    recording: "\x1b[38;2;217;119;87m",  // Orange #d97757
    processing: "\x1b[38;2;106;155;204m", // Blue #6a9bcc
    success: "\x1b[38;2;120;140;93m",     // Green #788c5d
    warning: "\x1b[38;2;217;119;87m",     // Orange #d97757
    error: "\x1b[38;2;217;119;87m",       // Orange #d97757
    info: "\x1b[38;2;106;155;204m",       // Blue #6a9bcc
    reset: "\x1b[0m",
    dim: "\x1b[38;2;176;174;165m",    // Mid gray #b0aea5
    bg_primary: "",                   // Transparent
    bg_secondary: "",                 // Transparent
    border: "\x1b[38;2;176;174;165m", // Mid gray #b0aea5
    borders: BORDER_ROUNDED,
    indicator_rec: "●",
    indicator_auto: "◉",
    indicator_manual: "●",
    indicator_idle: "○",
};

/// Codex theme - cool blue neutrals (OpenAI-style, neutral)
/// Uses transparent backgrounds for best compatibility across terminals
pub const THEME_CODEX: ThemeColors = ThemeColors {
    recording: "\x1b[38;2;111;177;255m",  // Blue #6fb1ff
    processing: "\x1b[38;2;154;215;255m", // Light blue #9ad7ff
    success: "\x1b[38;2;122;212;168m",    // Mint #7ad4a8
    warning: "\x1b[38;2;242;201;125m",    // Amber #f2c97d
    error: "\x1b[38;2;255;123;123m",      // Soft red #ff7b7b
    info: "\x1b[38;2;143;200;255m",       // Sky #8fc8ff
    reset: "\x1b[0m",
    dim: "\x1b[38;2;122;133;153m",    // Cool gray #7a8599
    bg_primary: "",                   // Transparent
    bg_secondary: "",                 // Transparent
    border: "\x1b[38;2;143;200;255m", // Sky #8fc8ff
    borders: BORDER_SINGLE,
    indicator_rec: "●",
    indicator_auto: "◉",
    indicator_manual: "●",
    indicator_idle: "○",
};

/// Catppuccin Mocha theme - pastel colors
/// https://github.com/catppuccin/catppuccin
/// Uses transparent backgrounds for best compatibility across terminals
pub const THEME_CATPPUCCIN: ThemeColors = ThemeColors {
    recording: "\x1b[38;2;243;139;168m",  // Red #f38ba8
    processing: "\x1b[38;2;249;226;175m", // Yellow #f9e2af
    success: "\x1b[38;2;166;227;161m",    // Green #a6e3a1
    warning: "\x1b[38;2;250;179;135m",    // Peach #fab387
    error: "\x1b[38;2;243;139;168m",      // Red #f38ba8
    info: "\x1b[38;2;137;180;250m",       // Blue #89b4fa
    reset: "\x1b[0m",
    dim: "\x1b[38;2;108;112;134m",    // Overlay0 #6c7086
    bg_primary: "",                   // Transparent
    bg_secondary: "",                 // Transparent
    border: "\x1b[38;2;180;190;254m", // Lavender #b4befe
    borders: BORDER_DOUBLE,
    indicator_rec: "◉",
    indicator_auto: "◈",
    indicator_manual: "◆",
    indicator_idle: "◇",
};

/// Dracula theme - high contrast
/// https://draculatheme.com
/// Uses transparent backgrounds for best compatibility across terminals
pub const THEME_DRACULA: ThemeColors = ThemeColors {
    recording: "\x1b[38;2;255;85;85m",    // Red #ff5555
    processing: "\x1b[38;2;241;250;140m", // Yellow #f1fa8c
    success: "\x1b[38;2;80;250;123m",     // Green #50fa7b
    warning: "\x1b[38;2;255;184;108m",    // Orange #ffb86c
    error: "\x1b[38;2;255;85;85m",        // Red #ff5555
    info: "\x1b[38;2;139;233;253m",       // Cyan #8be9fd
    reset: "\x1b[0m",
    dim: "\x1b[38;2;98;114;164m",     // Comment #6272a4
    bg_primary: "",                   // Transparent
    bg_secondary: "",                 // Transparent
    border: "\x1b[38;2;189;147;249m", // Purple #bd93f9
    borders: BORDER_HEAVY,
    indicator_rec: "⬤",
    indicator_auto: "⏺",
    indicator_manual: "⏵",
    indicator_idle: "○",
};

/// Nord theme - arctic blue-gray
/// https://www.nordtheme.com
pub const THEME_NORD: ThemeColors = ThemeColors {
    recording: "\x1b[38;2;191;97;106m",   // Aurora red #bf616a
    processing: "\x1b[38;2;235;203;139m", // Aurora yellow #ebcb8b
    success: "\x1b[38;2;163;190;140m",    // Aurora green #a3be8c
    warning: "\x1b[38;2;208;135;112m",    // Aurora orange #d08770
    error: "\x1b[38;2;191;97;106m",       // Aurora red #bf616a
    info: "\x1b[38;2;136;192;208m",       // Frost #88c0d0
    reset: "\x1b[0m",
    dim: "\x1b[38;2;76;86;106m",      // Polar Night #4c566a
    bg_primary: "",                   // Transparent to avoid wash-out on dark terminals
    bg_secondary: "",                 // Transparent to avoid wash-out on dark terminals
    border: "\x1b[38;2;136;192;208m", // Frost #88c0d0
    borders: BORDER_ROUNDED,
    indicator_rec: "◆",
    indicator_auto: "❄",
    indicator_manual: "▸",
    indicator_idle: "◇",
};

/// ANSI 16-color theme - works on all color terminals
/// Uses standard ANSI escape codes (30-37, 90-97)
/// Uses transparent backgrounds for best compatibility across terminals
pub const THEME_ANSI: ThemeColors = ThemeColors {
    recording: "\x1b[31m",  // Red
    processing: "\x1b[33m", // Yellow
    success: "\x1b[32m",    // Green
    warning: "\x1b[33m",    // Yellow
    error: "\x1b[31m",      // Red
    info: "\x1b[36m",       // Cyan
    reset: "\x1b[0m",
    dim: "\x1b[90m",    // Dark gray (bright black)
    bg_primary: "",     // Transparent
    bg_secondary: "",   // Transparent
    border: "\x1b[37m", // White
    borders: BORDER_SINGLE,
    indicator_rec: "*",
    indicator_auto: "@",
    indicator_manual: ">",
    indicator_idle: "-",
};

/// No colors - plain text output
pub const THEME_NONE: ThemeColors = ThemeColors {
    recording: "",
    processing: "",
    success: "",
    warning: "",
    error: "",
    info: "",
    reset: "",
    dim: "",
    bg_primary: "",
    bg_secondary: "",
    border: "",
    borders: BORDER_SINGLE,
    indicator_rec: "*",
    indicator_auto: "@",
    indicator_manual: ">",
    indicator_idle: "-",
};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_from_name_parses_valid() {
        assert_eq!(Theme::from_name("coral"), Some(Theme::Coral));
        assert_eq!(Theme::from_name("claude"), Some(Theme::Claude));
        assert_eq!(Theme::from_name("anthropic"), Some(Theme::Claude));
        assert_eq!(Theme::from_name("codex"), Some(Theme::Codex));
        assert_eq!(Theme::from_name("CATPPUCCIN"), Some(Theme::Catppuccin));
        assert_eq!(Theme::from_name("Dracula"), Some(Theme::Dracula));
        assert_eq!(Theme::from_name("nord"), Some(Theme::Nord));
        assert_eq!(Theme::from_name("ansi"), Some(Theme::Ansi));
        assert_eq!(Theme::from_name("ansi16"), Some(Theme::Ansi));
        assert_eq!(Theme::from_name("none"), Some(Theme::None));
        assert_eq!(Theme::from_name("default"), Some(Theme::Coral));
    }

    #[test]
    fn theme_is_truecolor() {
        assert!(!Theme::Coral.is_truecolor());
        assert!(Theme::Claude.is_truecolor());
        assert!(Theme::Codex.is_truecolor());
        assert!(Theme::Catppuccin.is_truecolor());
        assert!(Theme::Dracula.is_truecolor());
        assert!(Theme::Nord.is_truecolor());
        assert!(!Theme::Ansi.is_truecolor());
        assert!(!Theme::None.is_truecolor());
    }

    #[test]
    fn theme_fallback_for_ansi() {
        assert_eq!(Theme::Coral.fallback_for_ansi(), Theme::Coral);
        assert_eq!(Theme::Claude.fallback_for_ansi(), Theme::Ansi);
        assert_eq!(Theme::Codex.fallback_for_ansi(), Theme::Ansi);
        assert_eq!(Theme::Catppuccin.fallback_for_ansi(), Theme::Ansi);
        assert_eq!(Theme::Dracula.fallback_for_ansi(), Theme::Ansi);
        assert_eq!(Theme::None.fallback_for_ansi(), Theme::None);
    }

    #[test]
    fn theme_from_name_rejects_invalid() {
        assert_eq!(Theme::from_name("invalid"), None);
        assert_eq!(Theme::from_name(""), None);
    }

    #[test]
    fn theme_colors_returns_palette() {
        let colors = Theme::Coral.colors();
        assert!(colors.recording.contains("\x1b["));
        assert!(colors.reset.contains("\x1b[0m"));

        let none_colors = Theme::None.colors();
        assert!(none_colors.recording.is_empty());
    }

    #[test]
    fn theme_display_matches_name() {
        assert_eq!(format!("{}", Theme::Coral), "coral");
        assert_eq!(format!("{}", Theme::Claude), "claude");
        assert_eq!(format!("{}", Theme::Codex), "codex");
        assert_eq!(format!("{}", Theme::Catppuccin), "catppuccin");
    }

    #[test]
    fn theme_has_unique_borders() {
        // Each theme should have visually distinct borders
        assert_eq!(Theme::Coral.colors().borders.horizontal, '─');
        assert_eq!(Theme::Catppuccin.colors().borders.horizontal, '═');
        assert_eq!(Theme::Dracula.colors().borders.horizontal, '━');
        assert_eq!(Theme::Nord.colors().borders.top_left, '╭');
    }

    #[test]
    fn theme_has_indicators() {
        let colors = Theme::Coral.colors();
        assert!(!colors.indicator_rec.is_empty());
        assert!(!colors.indicator_auto.is_empty());
    }
}
