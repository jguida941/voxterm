# Troubleshooting Terminal/IDE Issues

Use this guide for rendering/input behavior differences across terminal emulators
(especially JetBrains/Cursor/VS Code).

## Contents

- [IDE terminal controls not working](#ide-terminal-controls-not-working-jetbrainscursor)
- [HUD duplicates in JetBrains terminals](#hud-duplicates-in-jetbrains-terminals)
- [Overlay flickers in JetBrains terminals](#overlay-flickers-in-jetbrains-terminals)
- [PTY exit write error in logs](#pty-exit-write-error-in-logs)
- [Startup banner missing](#startup-banner-missing)
- [Startup banner lingers in IDE terminal](#startup-banner-lingers-in-ide-terminal)
- [Theme colors look muted in IDE terminal](#theme-colors-look-muted-in-ide-terminal)
- [See Also](#see-also)

## IDE terminal controls not working (JetBrains/Cursor)

If HUD button clicks or arrow navigation fail in one terminal app but not
another:

1. Verify core shortcuts still work (`Ctrl+U`, `Ctrl+O`).
2. Capture input diagnostics:

   ```bash
   voiceterm --logs
   VOICETERM_DEBUG_INPUT=1 voiceterm --logs
   ```

3. Reproduce once and inspect `${TMPDIR}/voiceterm_tui.log` for `input bytes`
   and `input events` lines.

## HUD duplicates in JetBrains terminals

If Full HUD appears stacked/repeated:

1. Verify version:

   ```bash
   voiceterm --version
   ```

2. Re-run once with logs:

   ```bash
   voiceterm --logs
   ```

3. Share `${TMPDIR}/voiceterm_tui.log` if still reproducible.

## Overlay flickers in JetBrains terminals

If HUD rapidly flashes in JetBrains but not Cursor/VS Code:

1. Verify version.
2. Reproduce with `voiceterm --logs`.
3. Share logs + terminal app/version if it persists.

Implementation details on redraw/resize behavior are in
`dev/ARCHITECTURE.md`.

## PTY exit write error in logs

If you see:

```text
failed to send PTY exit command: PTY write failed: Input/output error (os error 5)
```

This is usually a benign shutdown race where the PTY was already closing.

## Startup banner missing

Splash is shown by default in non-JetBrains terminals. JetBrains terminals may
skip splash intentionally.

Check if banner is explicitly disabled:

```bash
env | rg VOICETERM_NO_STARTUP_BANNER
```

Disable explicitly (all terminals):

```bash
VOICETERM_NO_STARTUP_BANNER=1 voiceterm
```

## Startup banner lingers in IDE terminal

1. Check version:

   ```bash
   voiceterm --version
   ```

2. Test immediate splash clear:

   ```bash
   VOICETERM_STARTUP_SPLASH_MS=0 voiceterm
   ```

3. Disable splash globally if preferred:

   ```bash
   VOICETERM_NO_STARTUP_BANNER=1 voiceterm
   ```

## Theme colors look muted in IDE terminal

Some IDE profiles do not expose truecolor env vars.

1. Inspect env:

   ```bash
   env | rg 'COLORTERM|TERM|TERM_PROGRAM|TERMINAL_EMULATOR|NO_COLOR'
   ```

2. Ensure `NO_COLOR` is not set.
3. A/B test truecolor:

   ```bash
   COLORTERM=truecolor voiceterm --theme catppuccin
   ```

## See Also

| Topic | Link |
|-------|------|
| Troubleshooting hub | [TROUBLESHOOTING.md](TROUBLESHOOTING.md) |
| Install/update issues | [TROUBLESHOOTING_INSTALL.md](TROUBLESHOOTING_INSTALL.md) |
| Backend issues | [TROUBLESHOOTING_BACKEND.md](TROUBLESHOOTING_BACKEND.md) |
