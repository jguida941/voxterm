# Backlog

## UX
- [ ] Auto-voice status: while a capture is active in auto mode, the status line should keep showing "Listening..." even after toggling send mode or other settings.
- [ ] Add a mic-meter hotkey so users can calibrate VAD without restarting the app.
- [ ] Optional HUD input preview while Codex is thinking (Phase 3).
- [ ] Processing delay/freeze after sending input while Codex is thinking (audit after the queue fix).
- [ ] Status spam: repeated "Transcript ready (Rust pipeline)" lines appear many times in a row.

## Bugs / Reliability
- [ ] CSI-u garbage text appears at end of prompt (e.g., "48;0;0u"). Proposed fix: filter CSI-u sequences in overlay input (pending verification).
- [ ] Transcripts dropped while Codex is thinking. Proposed fix: queue transcripts and send on next prompt (pending verification).
- [ ] Unexpected command hint appears in output: "Use /skills to list available sk ..." shows up in the UI.
