#!/bin/bash
#
# VoxTerm - Quick Start
# Double-click this file or run: ./start.sh
#

# Save the user's current directory so voxterm works on their project
export VOXTERM_CWD="$(pwd)"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors - Vibrant red theme
CORAL='\033[38;2;255;90;90m'
CORAL_BRIGHT='\033[38;2;255;110;110m'
GREEN='\033[0;32m'
GOLD='\033[38;5;214m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
DIM='\033[2m'
BOLD='\033[1m'
NC='\033[0m'

if [ -n "${NO_COLOR:-}" ]; then
    CORAL=''
    CORAL_BRIGHT=''
    GREEN=''
    GOLD=''
    YELLOW=''
    RED=''
    DIM=''
    BOLD=''
    NC=''
fi

TERM_COLS="${VOXTERM_FORCE_COLUMNS:-${COLUMNS:-$(tput cols 2>/dev/null || true)}}"
if ! [ "$TERM_COLS" -gt 0 ] 2>/dev/null; then
    TERM_COLS=80
fi
TERM_LINES="${VOXTERM_FORCE_LINES:-${LINES:-$(tput lines 2>/dev/null || true)}}"
if ! [ "$TERM_LINES" -gt 0 ] 2>/dev/null; then
    TERM_LINES=24
fi

# Get version from Cargo.toml
VERSION="1.0.30"
if [ -f "$SCRIPT_DIR/rust_tui/Cargo.toml" ]; then
    VERSION=$(grep '^version' "$SCRIPT_DIR/rust_tui/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
fi

BACKEND_LABEL="codex"
THEME_LABEL="coral"
AUTO_LABEL="off"
if [ -n "${NO_COLOR:-}" ]; then
    THEME_LABEL="none"
fi

ARGS=("$@")
i=0
while [ "$i" -lt "${#ARGS[@]}" ]; do
    arg="${ARGS[$i]}"
    case "$arg" in
        --backend)
            i=$((i + 1))
            BACKEND_LABEL="${ARGS[$i]:-codex}"
            ;;
        --backend=*)
            BACKEND_LABEL="${arg#*=}"
            ;;
        --theme)
            i=$((i + 1))
            THEME_LABEL="${ARGS[$i]:-coral}"
            ;;
        --theme=*)
            THEME_LABEL="${arg#*=}"
            ;;
        --no-color|--no-colour)
            THEME_LABEL="none"
            ;;
        --auto-voice)
            AUTO_LABEL="on"
            ;;
    esac
    i=$((i + 1))
done

if [[ "$BACKEND_LABEL" == *" "* ]]; then
    BACKEND_LABEL="custom"
fi
BACKEND_LABEL="$(basename "$BACKEND_LABEL")"

truncate() {
    local value="$1"
    local width="$2"

    if [ "$width" -le 0 ]; then
        printf ""
        return
    fi
    if [ ${#value} -le "$width" ]; then
        printf "%s" "$value"
        return
    fi
    if [ "$width" -le 3 ]; then
        printf "%.*s" "$width" "$value"
        return
    fi
    printf "%s" "${value:0:$((width - 3))}..."
}

pad_line() {
    local value="$1"
    local width="$2"
    local clipped
    clipped="$(truncate "$value" "$width")"
    printf "%-*s" "$width" "$clipped"
}

print_header() {
    local max_inner=72
    local min_inner=44
    local inner=$((TERM_COLS - 6))
    if [ "$inner" -gt "$max_inner" ]; then
        inner="$max_inner"
    fi
    if [ "$inner" -lt "$min_inner" ]; then
        echo ""
        echo -e "  ${BOLD}VoxTerm${NC} ${DIM}v${VERSION}${NC}"
        echo -e "  ${DIM}Voice HUD for AI CLIs${NC}"
        echo -e "  ${DIM}Keys:${NC} ${BOLD}Ctrl+R${NC} record  ${DIM}|${NC} ${BOLD}Ctrl+V${NC} auto  ${DIM}|${NC} ${BOLD}?${NC} help  ${DIM}|${NC} ${BOLD}Ctrl+Q${NC} quit"
        echo ""
        return
    fi

    local border
    border=$(printf '%*s' "$inner" '' | tr ' ' '─')
    local line1 line2 line3 line4
    line1=$(pad_line "VoxTerm v${VERSION}" "$inner")
    line2=$(pad_line "Voice HUD for AI CLIs" "$inner")
    line3=$(pad_line "Backend: ${BACKEND_LABEL}  ·  Theme: ${THEME_LABEL}  ·  Auto: ${AUTO_LABEL}" "$inner")
    line4=$(pad_line "Keys: Ctrl+R record · Ctrl+V auto · ? help · Ctrl+Q quit" "$inner")

    echo ""
    echo -e "  ${CORAL}┌${border}┐${NC}"
    echo -e "  ${CORAL}│${NC} ${BOLD}${line1}${NC} ${CORAL}│${NC}"
    echo -e "  ${CORAL}│${NC} ${DIM}${line2}${NC} ${CORAL}│${NC}"
    echo -e "  ${CORAL}│${NC} ${DIM}${line3}${NC} ${CORAL}│${NC}"
    echo -e "  ${CORAL}│${NC} ${DIM}${line4}${NC} ${CORAL}│${NC}"
    echo -e "  ${CORAL}└${border}┘${NC}"
    echo ""
}

print_header

# Startup output-only mode for tests
if [ "${VOXTERM_STARTUP_ONLY:-0}" = "1" ]; then
    exit 0
fi

# Resolve binary (prefer local build; avoid wrapper recursion)
OVERLAY_BIN=""
if [ -x "$SCRIPT_DIR/rust_tui/target/release/voxterm" ]; then
    OVERLAY_BIN="$SCRIPT_DIR/rust_tui/target/release/voxterm"
fi

# Check if Rust overlay exists
if [ -z "$OVERLAY_BIN" ]; then
    echo -e "${YELLOW}Building VoxTerm (first time setup)...${NC}"
    cd rust_tui && cargo build --release --bin voxterm
    if [ $? -ne 0 ]; then
        echo -e "${RED}Build failed. Please check the error above.${NC}"
        exit 1
    fi
    cd ..
    OVERLAY_BIN="$SCRIPT_DIR/rust_tui/target/release/voxterm"
fi

# Check if whisper model exists
MODEL_PATH=""
DEFAULT_MODELS_DIR="$SCRIPT_DIR/models"
FALLBACK_MODELS_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/voxterm/models"
MODEL_DIR=""

IS_HOMEBREW=0
case "$SCRIPT_DIR" in
    /opt/homebrew/Cellar/*|/usr/local/Cellar/*) IS_HOMEBREW=1 ;;
esac

if [ -n "${VOXTERM_MODEL_DIR:-}" ]; then
    MODEL_DIR="$VOXTERM_MODEL_DIR"
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
    VOXTERM_MODEL_DIR="$MODEL_DIR" ./scripts/setup.sh models --base
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

echo -e "  ${DIM}Initializing...${NC}"
echo ""
if [ -z "$OVERLAY_BIN" ]; then
    echo -e "${RED}Overlay binary not found. Please run ./install.sh or build rust_tui.${NC}"
    exit 1
fi
EXTRA_ARGS=()
if [ $HAS_WHISPER_ARG -eq 0 ]; then
    EXTRA_ARGS+=(--whisper-model-path "$MODEL_PATH_ABS")
fi
"$OVERLAY_BIN" "${EXTRA_ARGS[@]}" "$@"
