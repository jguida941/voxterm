#!/usr/bin/env python3
"""Simple test to verify audio recording works."""

import subprocess
import sys
import time

print("Testing audio recording functionality...")
print("=" * 50)

# First, let's test with the Python pipeline
print("\n1. Testing Python pipeline (fallback):")
result = subprocess.run([
    sys.executable,
    "codex_voice.py",
    "--seconds", "3",
    "--no-codex",
    "--emit-json"
], capture_output=True, text=True)

if result.returncode == 0:
    print("✓ Python pipeline works!")
    # Parse JSON output
    import json
    for line in result.stdout.splitlines():
        if line.strip().startswith('{'):
            try:
                data = json.loads(line)
                print(f"  Transcript: {data.get('transcript', 'No transcript')}")
                break
            except json.JSONDecodeError:
                pass
else:
    print("✗ Python pipeline failed:")
    print(result.stderr)

print("\n2. Testing Rust TUI voice capture:")
print("Please launch the TUI manually and press Ctrl+R to test voice capture:")
print("")
print("  cd rust_tui")
print("  WHISPER_MODEL_PATH='../models/ggml-base.en.bin' cargo run")
print("")
print("Then press Ctrl+R and speak for 5 seconds.")
print("")
print("Expected behavior after fix:")
print("  - Audio should record successfully")
print("  - Sample rate will be resampled from device rate to 16kHz")
print("  - Transcription should work with whisper model")
print("")
print("If it still fails, check /tmp/codex_voice_tui.log for errors.")