#!/bin/bash
#
# Codex Voice - Quick Start
# Double-click this file or run: ./start.sh
#

# Save the user's current directory so codex-voice works on their project
export CODEX_VOICE_CWD="$(pwd)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo ""
echo -e "${GREEN}"
cat <<'BANNER'
  ____          _           __     __      _
 / ___|___   __| | ___ _ __ \ \   / /__ _ _| |_ ___
| |   / _ \ / _` |/ _ \ '__| \ \ / / _ \ '__| __/ __|
| |__| (_) | (_| |  __/ |     \ V /  __/ |  | |_\__ \
 \____\___/ \__,_|\___|_|      \_/ \___|_|   \__|___/
BANNER
echo -e "${NC}"
echo -e "${GREEN}Starting Codex Voice...${NC}"
echo ""
EXAMPLE_CMD="codex-voice"
if ! command -v codex-voice &> /dev/null; then
    EXAMPLE_CMD="./start.sh"
fi
echo "Quick controls: Ctrl+R record, Ctrl+V auto-voice, Ctrl+T send mode, Ctrl++/Ctrl+- sensitivity (5 dB), Ctrl+Q exit"
echo "Note: Ctrl++ is often Ctrl+=, Ctrl+- may require Ctrl+Shift+-"
echo "Start in auto-voice: $EXAMPLE_CMD --auto-voice"
echo "Start in insert mode: $EXAMPLE_CMD --voice-send-mode insert"
echo "Adjust sensitivity: $EXAMPLE_CMD --voice-vad-threshold-db -50"
echo "Auto-voice idle default: 1200ms (adjust with --auto-voice-idle-ms 700)"
echo ""

# Resolve overlay binary (prefer local build, fall back to PATH)
OVERLAY_BIN=""
if [ -x "$SCRIPT_DIR/rust_tui/target/release/codex_overlay" ]; then
    OVERLAY_BIN="$SCRIPT_DIR/rust_tui/target/release/codex_overlay"
elif command -v codex-overlay &> /dev/null; then
    OVERLAY_BIN="$(command -v codex-overlay)"
fi

# Check if Rust overlay exists
if [ -z "$OVERLAY_BIN" ]; then
    echo -e "${YELLOW}Building Rust overlay (first time setup)...${NC}"
    cd rust_tui && cargo build --release --bin codex_overlay
    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed. Please check the error above.${NC}"
        exit 1
    fi
    cd ..
    OVERLAY_BIN="$SCRIPT_DIR/rust_tui/target/release/codex_overlay"
fi

# Check if whisper model exists
MODEL_PATH=""
DEFAULT_MODELS_DIR="$SCRIPT_DIR/models"
FALLBACK_MODELS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/codex-voice/models"
MODEL_DIR=""

IS_HOMEBREW=0
case "$SCRIPT_DIR" in
    /opt/homebrew/Cellar/*|/usr/local/Cellar/*) IS_HOMEBREW=1 ;;
esac

if [ -n "${CODEX_VOICE_MODEL_DIR:-}" ]; then
    MODEL_DIR="$CODEX_VOICE_MODEL_DIR"
elif [ "$IS_HOMEBREW" -eq 1 ]; then
    MODEL_DIR="$FALLBACK_MODELS_DIR"
else
    if mkdir -p "$DEFAULT_MODELS_DIR" 2>/dev/null && [ -w "$DEFAULT_MODELS_DIR" ]; then
        MODEL_DIR="$DEFAULT_MODELS_DIR"
    else
        MODEL_DIR="$FALLBACK_MODELS_DIR"
    fi
fi

HAS_WHISPER_ARG=0
for arg in "$@"; do
    case "$arg" in
        --whisper-model-path|--whisper-model-path=*)
            HAS_WHISPER_ARG=1
            ;;
    esac
done

find_model() {
    local search_dir="$1"
    for candidate in \
        "ggml-small.en.bin" \
        "ggml-small.bin" \
        "ggml-base.en.bin" \
        "ggml-base.bin" \
        "ggml-tiny.en.bin" \
        "ggml-tiny.bin"; do
        if [ -f "$search_dir/$candidate" ]; then
            echo "$search_dir/$candidate"
            return 0
        fi
    done
    return 1
}

MODEL_PATH="$(find_model "$DEFAULT_MODELS_DIR" || true)"
if [ -z "$MODEL_PATH" ] && [ "$MODEL_DIR" != "$DEFAULT_MODELS_DIR" ]; then
    MODEL_PATH="$(find_model "$MODEL_DIR" || true)"
fi
if [ -z "$MODEL_PATH" ]; then
    echo -e "${YELLOW}Downloading Whisper model (first time setup)...${NC}"
    CODEX_VOICE_MODEL_DIR="$MODEL_DIR" ./scripts/setup.sh models --base
    if [ $? -ne 0 ]; then
        echo -e "${RED}Model download failed. Please check the error above.${NC}"
        exit 1
    fi
    MODEL_PATH="$(find_model "$MODEL_DIR" || true)"
fi

if [ -z "$MODEL_PATH" ]; then
    echo -e "${RED}Whisper model not found. Run: ./scripts/setup.sh models --base${NC}"
    exit 1
fi
MODEL_PATH_ABS="$MODEL_PATH"

echo -e "${GREEN}Launching overlay mode...${NC}"
if [ -z "$OVERLAY_BIN" ]; then
    echo -e "${RED}Overlay binary not found. Please run ./install.sh or build rust_tui.${NC}"
    exit 1
fi
EXTRA_ARGS=()
if [ $HAS_WHISPER_ARG -eq 0 ]; then
    EXTRA_ARGS+=(--whisper-model-path "$MODEL_PATH_ABS")
fi
"$OVERLAY_BIN" "${EXTRA_ARGS[@]}" "$@"
