#!/bin/bash
# Test voice capture to see if Rust or Python path is used

export RUST_LOG=debug
export CODEX_LOG=1

echo "Starting TUI with debug logging..."
echo "Press 'v' to start voice capture when TUI opens"
echo "Check debug.log for which path is used"

./target/release/rust_tui --log-file debug.log 2>&1 | tee tui_output.log &
TUI_PID=$!

echo "TUI started with PID $TUI_PID"
echo "Log will be in debug.log"
echo "Press Ctrl+C to stop monitoring"

wait $TUI_PID
