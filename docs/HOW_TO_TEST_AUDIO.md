# How to Test Audio Recording Fix

## What Was Fixed

The audio recording wasn't working because of a sample rate mismatch. The recorder was capturing at 48kHz but whisper expected 16kHz. I've added resampling to fix this.

## Test Instructions

### Method 1: Direct Audio Test (Recommended)
```bash
cd rust_tui
cargo run --bin test_audio
```
This will record 3 seconds of audio and tell you if it detected sound.

### Method 2: Run the TUI with Debug Output
```bash
cd rust_tui
WHISPER_MODEL_PATH='../models/ggml-base.en.bin' cargo run 2>&1 | tee /tmp/audio_test.log
```

Then:
1. Press **Ctrl+R** (or **Alt+R** or **F2**) to start recording
2. Speak for 5 seconds
3. Check if transcript appears

### Method 3: Check Debug Logs
After running the TUI and pressing Ctrl+R, check the debug log:
```bash
tail -100 /tmp/codex_voice_tui.log
```

Look for these debug messages:
- `capture_voice_native: Starting`
- `DEBUG: Starting recording for 5 seconds`
- `DEBUG: Device config - Sample rate: 48000Hz`
- `DEBUG: Captured XXXXX total samples`
- `DEBUG: Resampling from 48000Hz to 16000Hz`
- `capture_voice_native: Transcription complete`

## What to Expect

### If Working:
- Audio records successfully
- You see debug output showing resampling
- Transcript appears in the input field

### If Not Working:
- Check that macOS has granted microphone permission
- Look for error messages in `/tmp/codex_voice_tui.log`
- Try the Python fallback by NOT setting WHISPER_MODEL_PATH

## Troubleshooting

1. **No microphone access**: macOS may need to grant permission to Terminal/iTerm
2. **Model not found**: Make sure `models/ggml-base.en.bin` exists
3. **Still silent**: The test_audio binary will tell you if audio is being captured at all

## Key Changes Made

1. Added sample rate detection in `audio.rs`
2. Implemented `resample_linear()` function to convert audio rates
3. Added extensive debug logging throughout the recording pipeline
4. Fixed the mismatch between device rate (48kHz) and whisper rate (16kHz)