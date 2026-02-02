//! Terminal UI shell for the voxterm pipeline. It mirrors the Python
//! prototype but wraps it in a full-screen experience driven by ratatui.

mod logging;
mod state;
#[cfg(test)]
mod tests;

#[cfg(test)]
pub(crate) use logging::set_logging_for_tests;
pub use logging::{crash_log_path, init_logging, log_debug, log_debug_content, log_file_path, log_panic};
pub use state::App;
#[allow(unused_imports)]
pub(crate) use state::{run_python_transcription, PipelineJsonResult, PipelineMetrics};
