#!/usr/bin/env bash
# Test script for audio recording functionality

set -euo pipefail

echo "Testing audio recording in Rust TUI..."
echo "This will record 3 seconds of audio and transcribe it."
echo "Please say something when prompted!"
echo ""

cd /Users/jguida941/new_github_projects/codex_voice/rust_tui

# Run a simple test that records and transcribes
cargo run -- \
    --whisper-model-path ../models/ggml-base.en.bin \
    --seconds 3 \
    --list-input-devices

echo ""
echo "Now testing actual recording..."
echo "Say something in the next 3 seconds!"

# Create a simple Rust test program
cat > test_audio.rs << 'EOF'
use anyhow::Result;
mod audio;
mod stt;

fn main() -> Result<()> {
    println!("Initializing audio recorder...");
    let recorder = audio::Recorder::new(None)?;

    println!("Recording for 3 seconds - please speak now!");
    let samples = recorder.record(3)?;
    println!("Recorded {} samples", samples.len());

    println!("Initializing transcriber...");
    let transcriber = stt::Transcriber::new("../models/ggml-base.en.bin")?;

    println!("Transcribing...");
    let transcript = transcriber.transcribe(&samples, "en")?;

    println!("\nTranscript: {}", transcript);

    Ok(())
}
EOF

# Compile and run the test
rustc --edition 2021 test_audio.rs -L target/debug/deps -L target/debug \
    --extern anyhow=target/debug/deps/libanyhow*.rlib \
    --extern whisper_rs=target/debug/deps/libwhisper_rs*.rlib \
    --extern cpal=target/debug/deps/libcpal*.rlib \
    --extern num_cpus=target/debug/deps/libnum_cpus*.rlib \
    -o test_audio 2>/dev/null || {
    echo "Direct compilation failed, using cargo instead..."

    # Create a minimal test binary in the project
    cat > src/bin/test_audio.rs << 'EOF'
use anyhow::Result;
use rust_tui::{audio, stt};

fn main() -> Result<()> {
    println!("Initializing audio recorder...");
    let recorder = audio::Recorder::new(None)?;

    println!("Recording for 3 seconds - please speak now!");
    let samples = recorder.record(3)?;
    println!("Recorded {} samples", samples.len());

    println!("Initializing transcriber...");
    let transcriber = stt::Transcriber::new("../models/ggml-base.en.bin")?;

    println!("Transcribing...");
    let transcript = transcriber.transcribe(&samples, "en")?;

    println!("\nTranscript: {}", transcript);

    Ok(())
}
EOF

    cargo build --bin test_audio 2>&1 | tail -5
    cargo run --bin test_audio
}

echo ""
echo "Test complete!"