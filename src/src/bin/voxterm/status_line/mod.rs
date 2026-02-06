//! Enhanced status line layout with sections.
//!
//! Provides a structured status line with mode indicator, pipeline tag,
//! sensitivity level, status message, and keyboard shortcuts.
//!
//! Now supports a multi-row banner layout with themed borders.
//! Buttons are clickable - click positions are tracked for mouse interaction.

mod animation;
mod buttons;
mod format;
mod layout;
mod state;
mod text;

pub use buttons::get_button_positions;
pub use format::format_status_banner;
pub use layout::status_banner_height;
#[allow(unused_imports)]
pub use state::ButtonPosition;
pub use state::{METER_HISTORY_MAX, Pipeline, RecordingState, StatusBanner, StatusLineState, VoiceMode};
