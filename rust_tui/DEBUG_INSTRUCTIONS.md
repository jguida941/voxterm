# Debugging Voice Crash Issue

## Current Status
We've implemented multiple layers of filtering and safety checks to prevent the byte index underflow crash in ratatui:

### Fixes Applied:

1. **Pre-filtering** (`pre_filter_terminal_sequences` in `app.rs`):
   - Removes complete ANSI escape sequences from PTY chunks before concatenation
   - Prevents sequences from being split across chunk boundaries

2. **Sanitization** (`sanitize_pty_output` in `app.rs`):
   - Explicitly removes `"0;0;0u"` pattern
   - Strips orphaned terminal sequences
   - Multiple passes to catch all variations

3. **Validation** (`validate_for_display` in `app.rs`):
   - Aggressively removes terminal sequences
   - Strips control characters and zero-width spaces
   - Ensures valid UTF-8 boundaries
   - Returns safe default (" ") for empty/invalid strings

4. **UI Protection** (`ui.rs`):
   - Final filtering before passing to ratatui
   - Replaces zero-width characters with spaces
   - Detects problematic text and disables wrapping when needed
   - Never passes empty strings to `Line::from()`

5. **PTY Session** (`pty_session.rs`):
   - Strips terminal queries without sending replies
   - Prevents feedback loops

## Debug Logging
The code now includes extensive debug logging. To enable it:

```bash
export CODEX_DEBUG=1
export RUST_BACKTRACE=1
cargo run -- --seconds 3 --lang en --codex-cmd codex
```

Check `debug.log` for:
- Hex dumps of bytes being processed
- Detection of control characters
- Removal of terminal sequences
- Final output being sent to ratatui

## If Crash Still Occurs

1. **Check the debug log** for the last few entries before the crash:
   ```bash
   tail -n 50 debug.log
   ```

2. **Look for patterns** in the logged bytes:
   - Sequences starting with `1B` (ESC character)
   - Patterns like `30 3B 30 3B 30 75` ("0;0;0u")
   - Any bytes < 0x20 (control characters)

3. **Note the exact error**:
   - The byte index number (e.g., 18446638520581340673)
   - The string content shown in the error
   - The location (should be tui/src/wrapping.rs:21)

## Theory
The huge byte index (≈ usize::MAX) indicates an unsigned integer underflow. This happens when:
- ratatui tries to calculate `position - offset` where offset > position
- The text width calculation doesn't match the actual byte length
- There are invisible/zero-width characters confusing the wrapper

## Testing
Run the minimal test to verify ratatui handles the strings correctly:
```bash
cargo run --bin test_crash
```

If this doesn't crash but the main app does, the issue is likely related to:
- Terminal dimensions
- Scroll offset
- Multiple lines interaction
- The specific rendering context

## Next Steps If Still Crashing

1. Add more logging around the exact strings being passed to ratatui
2. Try disabling wrapping entirely: Remove `.wrap(Wrap { trim: false })`
3. Check if it's related to terminal size (try different terminal window sizes)
4. Consider updating ratatui to latest version
5. File a bug report with ratatui if we can create a minimal reproduction