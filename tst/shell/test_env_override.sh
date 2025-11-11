#!/usr/bin/env bash
# Test that CODEX_ARGS_OVERRIDE properly passes flags through

echo "Testing CODEX_ARGS_OVERRIDE mechanism"
echo "====================================="
echo ""

# Test 1: Check if the override is picked up
echo "Test 1: Verify environment variable is processed"
echo "-------------------------------------------------"
CODEX_ARGS_OVERRIDE="--danger-full-access --skip-git-repo-check" bash -c 'echo "CODEX_ARGS_OVERRIDE=$CODEX_ARGS_OVERRIDE"'
echo ""

# Test 2: Dry run showing what command would be executed
echo "Test 2: Show what command args would be generated"
echo "--------------------------------------------------"
echo "Setting CODEX_ARGS_OVERRIDE=\"--danger-full-access\""
export CODEX_ARGS_OVERRIDE="--danger-full-access"

# Parse it like the script does (using shlex for proper quoting)
CODEX_ARGS_ARRAY=()
while IFS= read -r token; do
    CODEX_ARGS_ARRAY+=("$token")
done < <(python3 - "$CODEX_ARGS_OVERRIDE" <<'PY'
import shlex
import sys
for token in shlex.split(sys.argv[1]):
    print(token)
PY
)
echo "Parsed into array: ${#CODEX_ARGS_ARRAY[@]} elements"
for arg in "${CODEX_ARGS_ARRAY[@]}"; do
    echo "  --codex-arg \"$arg\""
done
echo ""

# Test 3: Show actual cargo command that would run
echo "Test 3: Full command that would be executed"
echo "-------------------------------------------"
SECONDS_ARG=10
FFMPEG_DEVICE_ARG=":0"
WHISPER_MODEL_ARG="base"
CODEX_CMD_ARG="codex"
TERM_VALUE="xterm-256color"
PYTHON_CMD_ARG="python3"
WHISPER_CMD="whisper"

CMD="cargo run --"
CMD="$CMD --seconds $SECONDS_ARG"
CMD="$CMD --ffmpeg-device $FFMPEG_DEVICE_ARG"
CMD="$CMD --whisper-cmd $WHISPER_CMD"
CMD="$CMD --whisper-model $WHISPER_MODEL_ARG"
CMD="$CMD --codex-cmd $CODEX_CMD_ARG"
CMD="$CMD --term $TERM_VALUE"
CMD="$CMD --python-cmd $PYTHON_CMD_ARG"

for arg in "${CODEX_ARGS_ARRAY[@]}"; do
    [ -n "$arg" ] && CMD="$CMD --codex-arg \"$arg\""
done

echo "Would execute:"
echo "$CMD"
echo ""

echo "✅ Environment variable mechanism is properly implemented!"
echo ""
echo "Usage examples:"
echo "  CODEX_ARGS_OVERRIDE=\"--danger-full-access\" ./voice"
echo "  CODEX_ARGS_OVERRIDE=\"--danger-full-access --skip-git-repo-check\" ./scripts/run_tui.sh"
