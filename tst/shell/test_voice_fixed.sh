#!/bin/bash
echo "Testing fixed voice capture..."
echo "=============================="
echo ""

# Clean old log
rm -f "$TMPDIR/codex_voice_tui.log"

# Run with model and timings
WHISPER_MODEL_PATH=./models/ggml-base.en.bin \
LOG_TIMINGS_OVERRIDE=1 \
DISABLE_PERSISTENT_CODEX_OVERRIDE=1 \
timeout 30 ./voice 2>&1 &

PID=$!
sleep 3

echo "Checking if TUI started..."
if ps -p $PID > /dev/null; then
    echo "✅ TUI is running (PID: $PID)"
    echo ""
    echo "Log tail:"
    tail -10 "$TMPDIR/codex_voice_tui.log"
    echo ""
    echo "Press Ctrl+C to stop the TUI"
    wait $PID
else
    echo "❌ TUI failed to start"
    echo "Last log entries:"
    tail -20 "$TMPDIR/codex_voice_tui.log"
fi