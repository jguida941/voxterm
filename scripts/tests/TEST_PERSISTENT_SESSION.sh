#!/bin/bash
# Test script for persistent Codex session

echo "=========================================="
echo "Testing Persistent Codex Session"
echo "=========================================="
echo ""
echo "This test verifies that:"
echo "1. Codex starts once and stays alive"
echo "2. Multiple messages use the same session"
echo "3. Context is maintained between messages"
echo "4. Streaming output works"
echo ""

# Clear the log
LOG_FILE="$TMPDIR/voxterm_tui.log"
echo "Clearing log at: $LOG_FILE"
> "$LOG_FILE"

echo ""
echo "=== MANUAL TEST INSTRUCTIONS ==="
echo "1. Type a message manually (don't use voice)"
echo "2. Press Enter - should respond quickly"
echo "3. Type another message"
echo "4. Press Enter - should respond quickly with context"
echo "5. Check if Codex remembers the first message"
echo "6. Press Ctrl+C to exit"
echo ""
echo "Expected: Fast responses, maintained context"
echo ""
read -p "Press Enter to start the test..."

# Launch the TUI with fake whisper to avoid audio issues
cd /Users/jguida941/new_github_projects/voxterm/rust_tui
cargo run -- \
  --seconds 3 \
  --ffmpeg-device ":0" \
  --whisper-cmd ../stubs/fake_whisper \
  --whisper-model base \
  --codex-cmd codex

echo ""
echo "=========================================="
echo "Test Complete - Analyzing Results"
echo "=========================================="
echo ""

echo "=== SESSION START COUNT ==="
grep -c "Starting persistent Codex session" "$LOG_FILE" || echo "0"
echo "(Should be 1 if session persisted)"

echo ""
echo "=== MESSAGES SENT ==="
grep "Sending to Codex" "$LOG_FILE" | wc -l || echo "0"

echo ""
echo "=== CHECK FOR RESTARTS ==="
grep "Restarting Codex session" "$LOG_FILE" || echo "No restarts (good!)"

echo ""
echo "=== CODEX PROCESS CHECK ==="
echo "During the test, check if only ONE codex process is running:"
echo "ps aux | grep codex | grep -v grep"
echo ""

echo "Full log saved at: $LOG_FILE"