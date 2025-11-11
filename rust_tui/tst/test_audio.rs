use anyhow::Result;

fn main() -> Result<()> {
    println!("Testing audio recording...");
    println!("This will record 3 seconds of audio");
    println!("Please grant microphone access if prompted!");
    println!();

    // Try to create recorder
    println!("Creating audio recorder...");
    let recorder = rust_tui::audio::Recorder::new(None)?;

    println!("Press Enter to start recording (will record for 3 seconds)...");
    let mut input = String::new();
    std::io::stdin().read_line(&mut input)?;

    println!("\n[RECORDING] Speak now...");
    let samples = recorder.record(3)?;

    println!("\n[OK] Recording complete!");
    println!("Captured {} samples", samples.len());

    // Check if we actually got audio
    let max_amplitude = samples.iter().map(|s| s.abs()).fold(0.0f32, f32::max);
    println!("Max amplitude: {max_amplitude}");

    if max_amplitude < 0.001 {
        println!("[WARN] Audio seems to be silent! Check microphone permissions.");
    } else {
        println!("[OK] Audio detected!");
    }

    Ok(())
}
