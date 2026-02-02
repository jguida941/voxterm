#!/bin/bash
# Test TYPING ONLY - no voice!

echo "======================================="
echo "TYPING TEST - NO VOICE"
echo "======================================="
echo ""
echo "This tests TYPING messages to Codex"
echo "(Voice is DISABLED for this test)"
echo ""
echo "Instructions:"
echo "1. Type a message with keyboard"
echo "2. Press Enter to send"
echo "3. Wait for response"
echo "4. Type another message"
echo "5. Press Ctrl+C to exit"
echo ""
echo "Starting TUI without voice..."
echo ""

cd /Users/jguida941/new_github_projects/voxterm/rust_tui

# Run with fake whisper so voice doesn't interfere
cargo run -- \
  --seconds 1 \
  --ffmpeg-device ":0" \
  --whisper-cmd ../stubs/fake_whisper \
  --whisper-model base \
  --codex-cmd codex