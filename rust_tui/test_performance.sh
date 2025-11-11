#!/bin/bash
# Test performance of voice capture

echo "=== Codex Voice TUI Performance Test ==="
echo

# Check if model exists
if [ ! -f "models/ggml-tiny.en.bin" ]; then
    echo "⚠️  No Whisper model found. Will use Python fallback (slower)."
    echo "   Download a model with:"
    echo "   curl -L 'https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.en.bin' -o models/ggml-tiny.en.bin"
    echo
fi

# Build in release mode for best performance
echo "Building release binary..."
cargo build --release --features high-quality-audio

echo
echo "Running TUI with performance logging..."
echo "Press 'v' to test voice capture, 'q' to quit"
echo

# Run with timing logs
./target/release/rust_tui \
    --log-file perf.log \
    --log-timings \
    --input-device "MacBook Pro Microphone" \
    --seconds 3

echo
echo "=== Performance Summary ==="
if [ -f perf.log ]; then
    echo "Voice capture timings:"
    grep "timing|phase=voice_capture" perf.log | tail -5
    echo
    echo "Pipeline used:"
    grep -E "(Rust pipeline|Python fallback)" perf.log | tail -5
fi