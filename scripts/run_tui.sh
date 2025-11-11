#!/usr/bin/env bash
# Wrapper to launch the Rust TUI with sane defaults so Codex runs inside IDEs.

set -euo pipefail

CALLING_DIR="$(pwd)"

ROOT_DIR="$(cd -- "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PROJECT_DIR="${ROOT_DIR}/rust_tui"

# Activate venv if it exists
if [ -f "${ROOT_DIR}/.venv/bin/activate" ]; then
    echo "Activating Python virtual environment..."
    source "${ROOT_DIR}/.venv/bin/activate"
fi

# Determine which whisper command to use
if command -v whisper &> /dev/null; then
    # Use real whisper if available
    WHISPER_CMD="${ROOT_DIR}/.venv/bin/whisper"
    if [ ! -f "$WHISPER_CMD" ]; then
        WHISPER_CMD="whisper"
    fi
    echo "Using whisper: $WHISPER_CMD"
else
    # Fall back to fake whisper stub for testing
    WHISPER_CMD="${ROOT_DIR}/stubs/fake_whisper"
    echo "WARNING: Using fake_whisper stub (real whisper not found)"
fi

SECONDS_ARG="${SECONDS_OVERRIDE:-5}"
FFMPEG_DEVICE_ARG="${FFMPEG_DEVICE_OVERRIDE:-:0}"
WHISPER_MODEL_ARG="${WHISPER_MODEL_OVERRIDE:-base}"
CODEX_CMD_ARG="${CODEX_CMD_OVERRIDE:-codex}"
TERM_VALUE="${TERM_OVERRIDE:-xterm-256color}"
PYTHON_CMD_ARG="${PYTHON_CMD_OVERRIDE:-python3}"
PIPELINE_SCRIPT_ARG="${PIPELINE_SCRIPT_OVERRIDE:-}"
LOG_TIMINGS_FLAG="${LOG_TIMINGS_OVERRIDE:-}"
DISABLE_PERSISTENT_CODEX="${DISABLE_PERSISTENT_CODEX_OVERRIDE:-}"
WHISPER_MODEL_PATH_ARG="${WHISPER_MODEL_PATH:-}"
INPUT_DEVICE_ARG="${INPUT_DEVICE_OVERRIDE:-}"

# Optional extra Codex CLI flags (space-separated), e.g. "--danger-full-access"
CODEX_ARGS_OVERRIDE="${CODEX_ARGS_OVERRIDE:-}"
CODEX_ARGS_ARRAY=()

if [ -n "$WHISPER_MODEL_PATH_ARG" ]; then
  WHISPER_MODEL_PATH_ARG="$("$PYTHON_CMD_ARG" - "$WHISPER_MODEL_PATH_ARG" "$CALLING_DIR" <<'PY'
import os
import sys
path = os.path.expanduser(sys.argv[1])
base = sys.argv[2]
if not os.path.isabs(path):
    path = os.path.join(base, path)
print(os.path.abspath(path))
PY
  )"
fi

cd "$PROJECT_DIR"

if [ -n "$CODEX_ARGS_OVERRIDE" ]; then
  while IFS= read -r token; do
    CODEX_ARGS_ARRAY+=("$token")
  done < <("$PYTHON_CMD_ARG" - "$CODEX_ARGS_OVERRIDE" <<'PY'
import shlex
import sys
for token in shlex.split(sys.argv[1]):
    print(token)
PY
  )
fi

CMD=(cargo run --bin rust_tui -- \
  --seconds "$SECONDS_ARG" \
  --ffmpeg-device "$FFMPEG_DEVICE_ARG" \
  --whisper-cmd "$WHISPER_CMD" \
  --whisper-model "$WHISPER_MODEL_ARG" \
  --codex-cmd "$CODEX_CMD_ARG" \
  --term "$TERM_VALUE" \
  --python-cmd "$PYTHON_CMD_ARG")

if [ -n "$PIPELINE_SCRIPT_ARG" ]; then
  CMD+=(--pipeline-script "$PIPELINE_SCRIPT_ARG")
fi

if [ -n "$INPUT_DEVICE_ARG" ]; then
  CMD+=(--input-device "$INPUT_DEVICE_ARG")
fi

if [ -n "$LOG_TIMINGS_FLAG" ]; then
  CMD+=(--log-timings)
fi

if [ -n "$DISABLE_PERSISTENT_CODEX" ]; then
  CMD+=(--no-persistent-codex)
fi

if [ -n "$WHISPER_MODEL_PATH_ARG" ]; then
  CMD+=(--whisper-model-path "$WHISPER_MODEL_PATH_ARG")
fi

if [ ${#CODEX_ARGS_ARRAY[@]} -gt 0 ]; then
  for arg in "${CODEX_ARGS_ARRAY[@]}"; do
    [ -n "$arg" ] && CMD+=(--codex-arg "$arg")
  done
fi

# Forward any extra CLI args (e.g., --list-input-devices) to the Rust binary
if [ "$#" -gt 0 ]; then
  CMD+=("$@")
fi

exec "${CMD[@]}"
