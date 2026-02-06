pub mod audio;
pub mod auth;
pub mod backend;
pub mod codex;
pub mod config;
pub mod doctor;
pub mod ipc;
mod lock;
pub mod mic_meter;
pub mod pty_session;
pub mod stt;
pub mod terminal_restore;
mod telemetry;
pub mod legacy_ui;
pub mod utf8_safe;
#[cfg(feature = "vad_earshot")]
pub mod vad_earshot;
pub mod voice;

mod legacy_tui;

pub(crate) use lock::lock_or_recover;
pub use legacy_tui::*;
pub use voice::{VoiceCaptureSource, VoiceCaptureTrigger, VoiceJob, VoiceJobMessage};
