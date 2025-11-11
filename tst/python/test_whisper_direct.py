#!/usr/bin/env python3
"""Test whisper transcription directly"""

import subprocess
import tempfile
import os

print("Recording 3 seconds of audio with ffmpeg...")
print("SPEAK NOW!")

# Record audio
with tempfile.NamedTemporaryFile(suffix='.wav', delete=False) as f:
    audio_file = f.name

cmd = [
    'ffmpeg', '-y',
    '-f', 'avfoundation',
    '-i', ':0',
    '-t', '3',
    '-ar', '16000',
    '-ac', '1',
    '-sample_fmt', 's16',
    audio_file
]

result = subprocess.run(cmd, capture_output=True, text=True)
if result.returncode != 0:
    print(f"Recording failed: {result.stderr}")
    exit(1)

print(f"\nAudio saved to: {audio_file}")
print(f"File size: {os.path.getsize(audio_file)} bytes")

# Try Python whisper
print("\nTrying Python whisper...")
try:
    import whisper
    model = whisper.load_model("base")
    result = model.transcribe(audio_file)
    print(f"Python whisper result: '{result['text']}'")
except Exception as e:
    print(f"Python whisper failed: {e}")

print("\nDone!")