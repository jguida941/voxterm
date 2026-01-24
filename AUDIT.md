# Codex Voice Install Audit

Date: Sat Jan 24 02:14:41 EST 2026
Scope: README.md, QUICK_START.md, /tmp/homebrew-codex-voice/README.md

Summary:
- Homebrew install path succeeded and reinstalled v1.0.2 after tap update.
- Startup banner switches to the compact TS-style box for short terminals.
- Quick controls now render as a shorter dual-color table to avoid cutoffs.
- Model download verified using a temp model dir via `CODEX_VOICE_MODEL_DIR`.
- `codex-voice --help` used for a non-interactive run from a temp project directory.
- Commands outside the Homebrew/local install paths were not run (developer flows).

Extra commands executed (not in docs):
- `brew update` (refresh tap after formula update).
- `brew uninstall --ignore-dependencies codex-voice` (clean reinstall).
- `brew install codex-voice` (install v1.0.2 from tap).
- `mktemp -d /tmp/codex-voice-project-XXXX` (temp project dir).
- `mktemp -d /tmp/codex-voice-models-XXXX` (temp model dir).
- `CODEX_VOICE_MODEL_DIR=... codex-voice --help` (startup output + model download without full overlay).
- `CODEX_VOICE_MODEL_DIR=... /path/to/start.sh --help` (local launch output check).

## Homebrew tap README (/tmp/homebrew-codex-voice/README.md)
| Command | Status | Notes |
| --- | --- | --- |
| `brew tap jguida941/homebrew-codex-voice` | PASS | Tap already present; updated via `brew update`. |
| `brew install codex-voice` | PASS | Installed v1.0.2 after uninstall/reinstall from tap. |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-XXXX`. |
| `codex-voice` | PASS | Ran `codex-voice --help` with `CODEX_VOICE_MODEL_DIR` to show banner/tables and trigger download. |
| `$(brew --prefix)/opt/codex-voice/libexec/scripts/setup.sh models --base` | NOT RUN | Model download verified via `CODEX_VOICE_MODEL_DIR` instead. |

## README.md
### Prerequisites
| Command | Status | Notes |
| --- | --- | --- |
| `npm install -g @anthropic-ai/codex` | NOT RUN | Not needed for Homebrew/local install validation. |

### Install & Run
| Command | Status | Notes |
| --- | --- | --- |
| `git clone https://github.com/jguida941/codex-voice.git` | NOT RUN | Repo already present. |
| `cd codex-voice` | PASS | Repo already present. |
| `./install.sh` | PASS | Built overlay and installed wrapper to `~/.local/bin`. |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-local`. |
| `codex-voice` | PASS | Ran `/Users/jguida941/.local/bin/codex-voice` to avoid PATH conflicts. |

### Manual build (Rust only)
| Command | Status | Notes |
| --- | --- | --- |
| `cd rust_tui && cargo build --release --bin codex_overlay` | NOT RUN | Not part of install flows. |
| `./target/release/codex_overlay` | NOT RUN | Not part of install flows. |

### Homebrew (optional)
| Command | Status | Notes |
| --- | --- | --- |
| `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"` | NOT RUN | Homebrew already installed. |
| `brew tap jguida941/homebrew-codex-voice` | PASS | Tap already present; updated via `brew update`. |
| `brew install codex-voice` | PASS | Installed v1.0.2 after uninstall/reinstall from tap. |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-XXXX`. |
| `codex-voice` | PASS | Ran `codex-voice --help` with `CODEX_VOICE_MODEL_DIR` to show banner/tables and trigger download. |
| `$(brew --prefix)/opt/codex-voice/libexec/scripts/setup.sh models --base` | NOT RUN | Model download verified via `CODEX_VOICE_MODEL_DIR` instead. |

### Manual (no Homebrew)
| Command | Status | Notes |
| --- | --- | --- |
| `CODEX_VOICE_CWD="$(pwd)" /path/to/codex-voice/start.sh` | NOT RUN | Not part of install flows. |

### Using With Your Own Projects (macOS Option B)
| Command | Status | Notes |
| --- | --- | --- |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-local`. |
| `/path/to/codex-voice/start.sh` | NOT RUN | Not part of install flows. |

### Linux / Command Line
| Command | Status | Notes |
| --- | --- | --- |
| `cd ~/my-project` | NOT RUN | Linux-only flow. |
| `/path/to/codex-voice/start.sh` | NOT RUN | Linux-only flow. |

### Install Globally (macOS/Linux)
| Command | Status | Notes |
| --- | --- | --- |
| `cd /path/to/codex-voice` | PASS | Used the repo root. |
| `./install.sh` | PASS | Built overlay and installed wrapper to `~/.local/bin`. |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-local`. |
| `codex-voice` | PASS | Ran `/Users/jguida941/.local/bin/codex-voice` to avoid PATH conflicts. |

### Common flags
| Command | Status | Notes |
| --- | --- | --- |
| `codex-voice --auto-voice` | NOT RUN | Requires interactive audio test. |
| `codex-voice --voice-send-mode insert` | NOT RUN | Requires interactive audio test. |
| `codex-voice --voice-vad-threshold-db -50` | NOT RUN | Sensitivity adjustment example. |
| `codex-voice --prompt-regex '^codex> $'` | NOT RUN | Requires interactive prompt test. |
| `codex-voice --input-device <NAME>` | NOT RUN | Requires device selection test. |
| `codex-voice --list-input-devices` | NOT RUN | Not part of install validation. |
| `codex-voice --whisper-model-path <PATH>` | NOT RUN | Requires manual model path. |
| `codex-voice --no-python-fallback` | NOT RUN | Requires interactive audio test. |

### CLI Options (Overlay)
| Command | Status | Notes |
| --- | --- | --- |
| `codex-voice --help` | PASS | Ran with `CODEX_VOICE_MODEL_DIR` to validate startup output non-interactively. |

### Building
| Command | Status | Notes |
| --- | --- | --- |
| `cd rust_tui && cargo build --release --bin codex_overlay` | NOT RUN | Dev command; not part of install. |
| `cd rust_tui && cargo build --release` | NOT RUN | Dev command; not part of install. |

### Testing
| Command | Status | Notes |
| --- | --- | --- |
| `cd rust_tui && cargo test` | NOT RUN | Dev command; not part of install. |
| `cd rust_tui && cargo test --bin codex_overlay` | NOT RUN | Dev command; not part of install. |

### Troubleshooting - Voice not working
| Command | Status | Notes |
| --- | --- | --- |
| `./scripts/setup.sh models --base` | NOT RUN | Not part of install flows. |
| `codex-voice --list-input-devices` | NOT RUN | Not part of install flows. |
| `codex-voice --no-python-fallback` | NOT RUN | Not part of install flows. |

### Troubleshooting - Codex not responding
| Command | Status | Notes |
| --- | --- | --- |
| `which codex` | NOT RUN | Not part of install flows. |
| `codex login` | NOT RUN | Requires provider credentials. |

### Troubleshooting - Homebrew link conflict
| Command | Status | Notes |
| --- | --- | --- |
| `brew link --overwrite codex-voice` | NOT RUN | Not needed; install linked successfully. |

## QUICK_START.md
### One-time setup
| Command | Status | Notes |
| --- | --- | --- |
| `npm install -g @anthropic-ai/codex` | NOT RUN | Not needed for Homebrew/local install validation. |

### Install Codex Voice (one time)
| Command | Status | Notes |
| --- | --- | --- |
| `git clone https://github.com/jguida941/codex-voice.git` | NOT RUN | Homebrew/local install used existing repo. |
| `cd codex-voice` | PASS | Repo already present. |
| `./install.sh` | PASS | Built overlay and installed wrapper to `~/.local/bin`. |

### Run from any project
| Command | Status | Notes |
| --- | --- | --- |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-local`. |
| `codex-voice` | PASS | Ran `/Users/jguida941/.local/bin/codex-voice` to avoid PATH conflicts. |

### Common flags
| Command | Status | Notes |
| --- | --- | --- |
| `codex-voice --auto-voice` | NOT RUN | Requires interactive audio test. |
| `codex-voice --voice-send-mode insert` | NOT RUN | Requires interactive audio test. |
| `codex-voice --voice-vad-threshold-db -50` | NOT RUN | Sensitivity adjustment example. |
| `codex-voice --prompt-regex '^codex> $'` | NOT RUN | Requires interactive prompt test. |

### Homebrew (optional)
| Command | Status | Notes |
| --- | --- | --- |
| `/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"` | NOT RUN | Homebrew already installed. |
| `brew tap jguida941/homebrew-codex-voice` | PASS | Tap already present; updated via `brew update`. |
| `brew install codex-voice` | PASS | Installed v1.0.2 after uninstall/reinstall from tap. |
| `cd ~/my-project` | PASS | Used `/tmp/codex-voice-project-XXXX`. |
| `codex-voice` | PASS | Ran `codex-voice --help` with `CODEX_VOICE_MODEL_DIR` to show banner/tables and trigger download. |
| `$(brew --prefix)/opt/codex-voice/libexec/scripts/setup.sh models --base` | NOT RUN | Model download verified via `CODEX_VOICE_MODEL_DIR` instead. |

### Troubleshooting
| Command | Status | Notes |
| --- | --- | --- |
| `./scripts/setup.sh models --base` | NOT RUN | Not part of install flows. |
| `./start.sh --no-python-fallback` | NOT RUN | Not part of install flows. |
| `brew link --overwrite codex-voice` | NOT RUN | Not needed; install linked successfully. |
