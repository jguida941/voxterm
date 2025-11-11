use anyhow::Result;
use rust_tui::{
    audio::Recorder, config::AppConfig, init_debug_log_file, log_debug, log_file_path, ui, App,
};

fn main() -> Result<()> {
    init_debug_log_file();
    let log_path = log_file_path();
    log_debug("=== Codex Voice TUI Started ===");
    log_debug(&format!("Log file: {log_path:?}"));

    let config = AppConfig::parse_args()?;
    if config.list_input_devices {
        list_input_devices()?;
        return Ok(());
    }
    let mut app = App::new(config);
    let result = ui::run_app(&mut app);

    log_debug("=== Codex Voice TUI Exiting ===");
    if let Err(ref e) = result {
        log_debug(&format!("Exit with error: {e:#}"));
    }

    result
}

fn list_input_devices() -> Result<()> {
    let devices = Recorder::list_devices()?;
    if devices.is_empty() {
        println!("No audio input devices detected.");
    } else {
        println!("Available audio input devices:");
        for name in devices {
            println!("  - {name}");
        }
    }
    Ok(())
}
