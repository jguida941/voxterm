use super::VadEngineKind;
use std::env;

pub const DEFAULT_VOICE_SAMPLE_RATE: u32 = 16_000;
pub const DEFAULT_VOICE_MAX_CAPTURE_MS: u64 = 30_000;
pub const DEFAULT_VOICE_SILENCE_TAIL_MS: u64 = 1000;
pub const DEFAULT_VOICE_MIN_SPEECH_MS: u64 = 300;
pub const DEFAULT_VOICE_LOOKBACK_MS: u64 = 500;
pub const DEFAULT_VOICE_BUFFER_MS: u64 = 30_000;
pub const DEFAULT_VOICE_CHANNEL_CAPACITY: usize = 100;
pub const DEFAULT_VOICE_STT_TIMEOUT_MS: u64 = 60_000;
pub const DEFAULT_VOICE_VAD_THRESHOLD_DB: f32 = -55.0;
pub const DEFAULT_VOICE_VAD_FRAME_MS: u64 = 20;
pub const DEFAULT_VOICE_VAD_SMOOTHING_FRAMES: usize = 3;
pub const DEFAULT_MIC_METER_AMBIENT_MS: u64 = 3000;
pub const DEFAULT_MIC_METER_SPEECH_MS: u64 = 3000;
pub const MIN_MIC_METER_SAMPLE_MS: u64 = 500;
pub const MAX_MIC_METER_SAMPLE_MS: u64 = 30_000;

pub(super) const MAX_CODEX_ARGS: usize = 64;
pub(super) const MAX_CODEX_ARG_BYTES: usize = 8 * 1024;
pub(super) const MAX_CAPTURE_HARD_LIMIT_MS: u64 = 60_000;
pub(super) const ISO_639_1_CODES: &[&str] = &[
    "af", "am", "ar", "az", "be", "bg", "bn", "bs", "ca", "cs", "cy", "da", "de", "el", "en", "es",
    "et", "eu", "fa", "fi", "fil", "fr", "ga", "gl", "gu", "he", "hi", "hr", "hu", "hy", "id",
    "is", "it", "ja", "jv", "ka", "kk", "km", "kn", "ko", "lo", "lt", "lv", "mk", "ml", "mn", "mr",
    "ms", "my", "ne", "nl", "no", "pa", "pl", "pt", "ro", "ru", "si", "sk", "sl", "sq", "sr", "sv",
    "sw", "ta", "te", "th", "tr", "uk", "ur", "vi", "zh",
];
// FFmpeg devices are passed to the shell, so strip characters that would let users sneak commands in.
pub(super) const FORBIDDEN_DEVICE_CHARS: &[char] =
    &[';', '|', '&', '$', '`', '<', '>', '\\', '\'', '"'];
pub(super) const DEFAULT_PIPELINE_SCRIPT: &str =
    concat!(env!("CARGO_MANIFEST_DIR"), "/../scripts/voxterm.py");
// PTY helper removed - using native Rust PtyCodexSession instead

pub const fn default_vad_engine() -> VadEngineKind {
    #[cfg(feature = "vad_earshot")]
    {
        VadEngineKind::Earshot
    }
    #[cfg(not(feature = "vad_earshot"))]
    {
        VadEngineKind::Simple
    }
}

pub(super) fn default_term() -> String {
    env::var("TERM").unwrap_or_else(|_| "xterm-256color".to_string())
}
