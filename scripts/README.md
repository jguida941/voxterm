# User Scripts

Scripts for installing and running VoxTerm.

| Script | Purpose | Usage |
|--------|---------|-------|
| `install.sh` | One-time installer | `./scripts/install.sh` |
| `start.sh` | Launch VoxTerm | `./scripts/start.sh` |
| `setup.sh` | Download Whisper models | `./scripts/setup.sh models --base` |
| `python_fallback.py` | Fallback STT pipeline | Used automatically |

## install.sh

Builds VoxTerm and installs the `voxterm` command.

```bash
./scripts/install.sh
```

## start.sh

Launches VoxTerm directly (downloads model if needed).

```bash
./scripts/start.sh
```

## setup.sh

Downloads Whisper models and performs initial setup.

```bash
# Download base English model (recommended)
./scripts/setup.sh models --base

# Download small model (larger, more accurate)
./scripts/setup.sh models --small

# Show help
./scripts/setup.sh --help
```

## python_fallback.py

Python fallback pipeline for speech-to-text. Used automatically when:
- Native Whisper model is not available
- Audio device issues occur

Requires: `python3`, `ffmpeg`, `whisper` CLI on PATH.

---

For developer scripts, see [dev/scripts/](../dev/scripts/).
