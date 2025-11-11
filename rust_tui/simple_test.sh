#!/bin/bash
clear
echo "=========================================="
echo "     CODEX VOICE TUI - QUICK TEST"
echo "=========================================="
echo
echo "✓ Models available:"
ls -lh ../models/*.bin | awk '{print "    •", $NF, "("$5")"}'
echo
echo "✓ Audio devices:"
./target/release/rust_tui --list-input-devices 2>&1 | sed 's/^/    /'
echo
echo "=========================================="
echo "  TO RUN THE TUI:"
echo "=========================================="
echo
echo "  cd $(pwd)"
echo "  ./target/release/rust_tui"
echo
echo "  Then:"
echo "    • Press 'v' to start voice capture"
echo "    • Speak for 3 seconds"
echo "    • Check status line:"
echo "        - 'Rust pipeline' = FAST (native)"
echo "        - 'Python fallback' = SLOW"
echo "    • Press 'q' to quit"
echo
echo "=========================================="
echo
echo "The TUI should use the RUST path since you have:"
echo "  • ggml-tiny.en.bin model ✓"
echo "  • MacBook Pro Microphone ✓"
echo "  • Release build compiled ✓"
echo
echo "If it's slow, check /tmp/codex_voice_tui.log"
