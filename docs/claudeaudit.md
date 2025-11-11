# CODEX VOICE MODE – AUDIT RESPONSE (Nov 2025)
**Date**: November 10, 2025
**Prepared by**: ChatGPT (Codex) cross-checking Claude Code’s report

## Executive Summary
- ✅ Persistent Codex PTY streaming is active again; `call_codex_via_session` waits for a 350 ms quiet period before returning and keeps sanitized stream chunks visible in the TUI (`rust_tui/src/app.rs:570-626`).
- ✅ Every Codex fallback path writes prompts with a newline via `write_prompt_with_newline`, so `codex exec -` no longer hangs waiting for input (`rust_tui/src/app.rs:528-557,668-674`).
- ✅ The native audio recorder detects device format, downmixes multi-channel input, recenters unsigned samples, and resamples to 16 kHz while logging backend errors to a file instead of corrupting the UI (`rust_tui/src/audio.rs:43-146`).
- ✅ Voice capture runs off the UI thread; `start_voice_capture` spawns a worker thread and the UI simply polls for completion, preventing the freezes described in the earlier audit (`rust_tui/src/app.rs:1174-1234`).
- ⚠️ Remaining opportunities involve adding a waitpid-based shutdown to the PTY drop, surfacing the selected device name in `Recorder::record`, and exploring higher-fidelity resampling plus extra tests. These are follow-ups rather than blockers.

## Updated Code Review Assessment (Nov 10, 2025)
ChatGPT’s re-audit and Claude’s follow-up agree on the current state:

### Issues Verified as Resolved
- **PTY streaming & quiet-period waits** – `call_codex_via_session` aggregates chunks until silence or timeout (`rust_tui/src/app.rs:570-626`).
- **Uniform newline handling** – `write_prompt_with_newline` is used for interactive, PTY helper, and `codex exec -` fallbacks (`rust_tui/src/app.rs:528-557,668-674`).
- **Background voice capture** – Recording and STT run on a worker thread, with the UI polling results via `VoiceJob` (`rust_tui/src/app.rs:1174-1234`).
- **Audio downmixing & normalization** – `append_downmixed_samples` handles multi-channel input and recenters U16 data before resampling (`rust_tui/src/audio.rs:43-167`).
- **Logging & FD hygiene** – `log_debug` writes to a rotating temp file, and Whisper stderr redirection restores descriptors (`rust_tui/src/app.rs:45-68`, `rust_tui/src/stt.rs:10-37`).

### Enhancements Still Worth Doing
1. **PTY drop wait loop** – Add a bounded `waitpid` poll before escalating to SIGTERM/SIGKILL for extra robustness (`rust_tui/src/pty_session.rs:113-136`).
2. **Explicit CPAL pause + richer errors** – Call `stream.pause()` before dropping the input stream and include the device name when no samples are captured (`rust_tui/src/audio.rs:99-117`).
3. **Higher-quality resampling (low priority)** – Consider a library such as `rubato` once stability tasks are done.
4. **Integration tests & config validation** – Add tests for PTY cleanup / audio downmixing, plus early validation of Whisper model paths and audio devices.
5. **Optional FD newtype** – Wrapping raw PTY FDs in a `PtyFd` drop guard would further reduce the risk of leaks (`rust_tui/src/pty_session.rs`).

These remaining items are characterized as polish rather than blockers; the codebase is now considered production ready once the high-priority PTY-drop tweak lands.

## November 2025 Update (Latest Work)
- **Anti-alias filtering fixed** – Downsampling now uses an order-scaling Hamming-windowed FIR before linear resampling, providing >40 dB attenuation for 48 kHz→16 kHz instead of the former 3-tap moving average (`rust_tui/src/audio.rs:151-220`).
- **DSP regression tests** – Unit tests synthesize tones above and below target Nyquist to assert alias suppression and passband preservation, preventing future regressions (`rust_tui/src/audio.rs:289-331`).
- **CLI hardening** – `AppConfig` enforces canonical paths inside the repo, executable-bit checks for binary overrides, ISO-639-1 language prefixes, arg count/length caps, and shell-metacharacter rejection for `--ffmpeg-device` (`rust_tui/src/config.rs:94-287`).
- **Remaining gaps** – Path checks still have a small canonicalize/metadata TOCTOU race, Windows builds do not yet enforce executability, and PTY teardown still needs a blocking `waitpid` loop.

### Verification Snapshot (Nov 10, 2025)
- **PTY read loop & newline handling** – `call_codex_via_session` waits for quiet periods before bailing (`rust_tui/src/app.rs:570-626`), and `write_prompt_with_newline` guarantees Codex receives a trailing LF in every fallback path (`rust_tui/src/app.rs:528-557,668-674`).
- **Audio capture fidelity/logging** – `append_downmixed_samples` averages all channels, U16 samples are re-centered, and downsampling now routes through an odd-length Hamming-windowed FIR before linear resampling to keep aliases >40 dB down when converting to 16 kHz (`rust_tui/src/audio.rs:151-220`).
- **Voice capture responsiveness** – Voice capture runs inside a worker thread (`rust_tui/src/app.rs:1174-1208`), and the UI loop polls non-blockingly so the TUI stays interactive (`rust_tui/src/app.rs:120-150`).
- **PTY streaming** – `app_loop` drains sanitized PTY output every frame, so streaming answers appear incrementally (`rust_tui/src/app.rs:115-134`).
- **CLI/build blockers** – The crate targets Rust 2021 and uses `shell_words` for safe `--codex-args` parsing (`rust_tui/Cargo.toml:1-20`, `rust_tui/src/app.rs:928-973`).
- **Unsafe syscall checks** – The Whisper stderr redirection path now verifies every `libc::dup`/`dup2` call, eliminating the previously identified critical bug (`rust_tui/src/stt.rs:10-37`).
- **Debug log rotation** – `init_debug_log_file` trims the temp log when it exceeds 5 MB before each run (`rust_tui/src/app.rs:45-68`), which is sufficient for single-instance use.

### Independent Verification Notes
- Follow-up testing confirmed all six “fixed” items remain resolved; the lone caveat (unchecked `dup`/`dup2`) has been addressed.
- Command injection concerns were downgraded: `shell_words` parsing plus `Command::arg` mean inputs never hit a shell, so remaining work is policy validation (e.g., disallowing dangerous Codex flags), not escaping.
- Audio fidelity improvements center on the built-in FIR + linear resampler: an order-scaling Hamming window replaces the former 3-tap average, so high-rate devices see proper anti-alias filtering without pulling in heavy external crates (`rust_tui/src/audio.rs:151-220`).
- `rust_tui/src/app.rs` continues to mix UI, config, Codex orchestration, and voice capture (≈1.2 k LOC). Splitting it into `ui`, `voice`, `codex`, and `config` modules remains high-value refactor work.

### New Test Coverage
- Audio resampling now ships with unit tests that synthesize above-/below-Nyquist tones to verify the FIR filter hits the >40 dB attenuation target while preserving passband energy (`rust_tui/src/audio.rs:289-331`).
- The `config` module now exercises boundary/validation cases plus shell-metacharacter and executable-bit enforcement (`rust_tui/src/config.rs:323-367`).
- Core `App` helpers (scrollback trimming, scroll offsets) are covered by lightweight unit tests to ensure UI interactions remain predictable (`rust_tui/src/app.rs:1293-1314`).

## Current State Snapshot

| Area | Status | Evidence / Notes |
| --- | --- | --- |
| Persistent Codex PTY session | ✅ | Streaming remains enabled, waits for silence, and falls back only after 5 s if nothing printable arrives (`rust_tui/src/app.rs:570-626`). |
| Codex fallback newline handling | ✅ | Fallback `codex exec -` path and helper always append `\n` when missing (`rust_tui/src/app.rs:528-557,668-674`). |
| Audio capture path | ✅ | Recorder inspects device config, downmixes channels, recenters unsigned samples, and now applies a configurable Hamming-windowed FIR before linear resampling so 48 kHz→16 kHz meets the alias budget (`rust_tui/src/audio.rs:43-220`). |
| Voice capture responsiveness | ✅ | Voice capture/job polling happens on background threads/mpsc, so the TUI stays responsive (`rust_tui/src/app.rs:1174-1234`). |
| Logging & log rotation | ✅ | `log_debug` writes to a temp file and truncates the log at 5 MB to avoid runaway stderr spam (`rust_tui/src/app.rs:45-68`). |
| Whisper stderr suppression / FD cleanup | ✅ | `Transcriber::new` dup/restore stderr around `WhisperContext::new` and closes the duped fd on every path (`rust_tui/src/stt.rs:10-37`). |
| Terminal query handling | ✅ | CSI queries are stripped and answered using bounded scans; no in-place iterator invalidation occurs (`rust_tui/src/pty_session.rs:292-325`). |
| Graceful PTY teardown | ⚠️ | `Drop` already sends `exit`, waits 100 ms, and escalates to SIGTERM/SIGKILL, but adding a waitpid loop would make the shutdown even more explicit (`rust_tui/src/pty_session.rs:113-136`). |

## Response to Claude’s November 2025 Findings

### 1. Resource management & safety
- **CPAL stream lifecycle** – The recorder now pauses the CPAL stream before dropping it and reports which input device produced zero samples, making troubleshooting far easier (`rust_tui/src/audio.rs:43-117`).
- **PTY session teardown** – The current `Drop` path already attempts a graceful `exit\n`, waits between signals, and closes descriptors (`rust_tui/src/pty_session.rs:113-136`). Adding an explicit `waitpid` loop is tracked as a follow-up, but the existing code does not kill the child before giving it at least 100 ms to exit.

### 2. Memory-safety claims
- `strip_and_reply` does not mutate the buffer while iterating through the same indices; it keeps an `idx` cursor, calls `find_subsequence`, drains the matched slice, and continues from the prior absolute index (`rust_tui/src/pty_session.rs:313-325`). This avoids the overlapping-drain bug the review warned about.
- The PTY reader uses a fixed 4 KiB stack buffer and only clones the bytes that were actually read before sending them across the channel, so there is no unbounded growth in that loop (`rust_tui/src/pty_session.rs:224-244`).

### 3. Error handling & diagnostics
- `Recorder::record` logs the full hardware format, pauses CPAL before teardown, and now applies the FIR prefilter discussed above before linear resampling, returning actionable errors if zero samples arrive so users are no longer left guessing which device failed (`rust_tui/src/audio.rs:51-220`).
- Whisper initialization temporarily redirects stderr to `/dev/null` and then restores it even if `WhisperContext::new` fails, preventing the descriptor leak described in the review (`rust_tui/src/stt.rs:10-37`).
- PTY writes go through `write_all`, which retries on `EINTR` and reports failures with context (`rust_tui/src/pty_session.rs:270-289`).

### 4. Performance & UX
- The event loop already polls the terminal every 100 ms only when no data arrives, and voice capture happens on a worker thread, so the UI no longer stalls during recording or transcription (`rust_tui/src/app.rs:120-150,1174-1234`).
- Linear resampling remains the final interpolation stage; a future migration to `rubato`/`speexdsp` is still on the roadmap, but the new FIR front-end keeps today’s implementation within spec for 48 kHz→16 kHz downsampling (`rust_tui/src/audio.rs:150-220`).
- The PTY output thread reuses a stack buffer and only allocates when forwarding chunks through the bounded channel, so there is no “excessive allocation” hotspot in its current form (`rust_tui/src/pty_session.rs:224-244`).

### 5. Logging, configuration, and quoting
- Debug logs never touch stderr; they go to `env::temp_dir()/codex_voice_tui.log`, and the file is truncated whenever it exceeds 5 MB to avoid runaway disk usage (`rust_tui/src/app.rs:45-68`).
- CLI parsing validates required values and supports both `--codex-arg` and a whitespace-separated `--codex-args` that is parsed with `shell_words`, so quoted strings stay intact (`rust_tui/src/app.rs:892-973`).
- The launcher script expands `WHISPER_MODEL_PATH` relative to the caller and tokenizes `CODEX_ARGS_OVERRIDE` with Python’s `shlex`, so quoted Codex arguments arrive exactly as typed (`scripts/run_tui.sh:43-112`).
- `AppConfig::validate` now enforces sane bounds on recording duration, ensures helper scripts exist, and rejects empty Codex commands or malformed language codes before the UI launches (`rust_tui/src/app.rs:902-964`).
- CLI parsing now uses `clap` with a dedicated `config` module, so new flags only require declarative definitions and validation stays centralized (`rust_tui/src/config.rs`).

### 6. Security considerations
- Prompts flow directly to Codex by design, but every write goes through `write_prompt_with_newline`, which ensures Codex receives well-formed input and prevents accidental hangs (`rust_tui/src/app.rs:528-557,668-674`).
- `AppConfig` inherits the caller’s working directory and never rewrites it, so the TUI runs Codex in whatever directory the user launched it from (`rust_tui/src/app.rs:523-557`).
- We still plan to cap recording duration (currently user-configurable via `--seconds`) and clamp scrollback at 500 lines to guard against runaway output (`rust_tui/src/app.rs:42-57,1163-1172`).

## Remaining follow-ups
1. Break the `rust_tui/src/app.rs` monolith into focused modules (`ui`, `voice`, `codex`, `config`) and migrate CLI parsing to a declarative crate such as `clap` for long-term maintainability.
2. Improve user transparency by surfacing when the app falls back from the persistent PTY to one-shot Codex invocations, and by logging retried voice captures with clearer status messaging.
3. Add integration/property tests that exercise the PTY lifecycle and the refreshed audio resampler to guard against regressions when devices run far above/below 16 kHz.

This document now reflects the actual state of the repository as of `HEAD`; future audits should start from this baseline instead of the older snapshot Claude referenced. The original report is preserved below for context.

---

# CODEX VOICE MODE - CORRECTED AUDIT REPORT
**Date**: November 10, 2025
**Auditor**: Claude Code (reconciled with ChatGPT verification)

## EXECUTIVE SUMMARY

Voice mode still fails to deliver a usable experience because multiple subsystems regress in concert:

1. **Persistent PTY is broken by logic, not timeouts** – `call_codex_via_session` aborts after the first 500 ms poll and returns the very first bytes it sees, so Codex never has a chance to stream output even though PTY stays enabled by default (`rust_tui/src/app.rs:361-659`).
2. **`codex exec` fallback never sends a newline** – when PTY and the interactive pipe both fail, the final fallback writes the prompt without `\n`, leaving Codex hung waiting for completion (`rust_tui/src/app.rs:563-585`).
3. **Audio capture corrupts data on most devices** – channel interleaving is never downmixed and unsigned 16-bit samples keep a DC offset, feeding Whisper distorted input (`rust_tui/src/audio.rs:52-105`).
4. **Voice capture blocks the UI thread** – every voice shortcut synchronously records, transcribes, and even re-captures after each send when auto mode is on, so the TUI freezes for 5-10 s at a time (`rust_tui/src/app.rs:201-420`).
5. **Build/config obstacles remain** – the crate still targets the nonexistent Rust 2024 edition (`rust_tui/Cargo.toml:4`), audio errors still print to stderr inside the TUI (`rust_tui/src/audio.rs:52`), and quoted Codex arguments are split incorrectly in both CLI parsing paths (`rust_tui/src/app.rs:938-943`, `scripts/run_tui.sh:50-88`).
6. **Dead and misleading code paths cause churn** – `rust_tui/src/codex_session.rs` is an unused placeholder, while comments describe PTY as “disabled” even though it is attempted on every send, hiding the real bug.

Fixing the PTY read loop, audio handling, and newline bug will unlock most of the missing functionality; the remaining work (async voice capture, logging, quoting) ensures polish and stability.

## ARCHITECTURE OVERVIEW

### Dual Pipeline Design
- **Python (`codex_voice.py`)**: shells out to FFmpeg for capture, runs Whisper CLI/whisper.cpp, then calls Codex via argv/stdin/PTY fallbacks.
- **Rust TUI (`rust_tui/src/app.rs`)**: records with `cpal`, transcribes with `whisper-rs`, calls Codex directly, and manages the terminal UI via `ratatui`.

### Session Management Reality
- The `AppConfig` default keeps `persistent_codex = true` (`rust_tui/src/app.rs:1023-1034`), so every prompt still tries to use `call_codex_via_session` before falling back. Only the streaming render loop was commented out near line 97; the PTY itself is still spawned.
- `pty_session.rs` already strips/responds to cursor/device CSI sequences, so the current failure mode is the 500 ms read loop plus newline omissions, not cursor queries.

### Codex Invocation Chain (Rust)
1. Attempt persistent PTY (`call_codex_via_session`).
2. Fallback: Python PTY helper (`scripts/run_in_pty.py`).
3. Fallback: spawn Codex with stdin pipes.
4. Last resort: `codex exec - -C …` but without the missing newline it never completes.

## DETAILED FINDINGS

### 1. Persistent PTY session never succeeds
- `send_prompt` still enters the PTY path whenever `persistent_codex` is true (`rust_tui/src/app.rs:361-387`).
- Inside `call_codex_via_session`, the first 500 ms poll that yields zero bytes triggers an immediate `bail!` (`rust_tui/src/app.rs:619-624`). Even when bytes do arrive, the code returns as soon as any sanitized text becomes non-empty (`rust_tui/src/app.rs:644-651`), so multi-chunk answers are truncated.
- Because `persistent_codex` defaults to true, every prompt incurs this broken attempt before the expensive one-shot CLI path. The audit previously claimed “5 s timeout” and “disabled PTY”; the real issue is this read loop plus the missing newline in the exec fallback.

### 2. `codex exec` fallback omits newline
- When both the PTY helper and interactive pipe fail, the code uses `codex exec -` but writes only `prompt.as_bytes()` (`rust_tui/src/app.rs:579-584`).Unlike earlier fallbacks, it never appends `\n`, so Codex waits for either a newline or EOF. Users perceive this as another silent hang, compounding latency after the broken PTY attempt.

### 3. Audio capture corrupts samples
- The callbacks never inspect `device_config.channels`; they simply append interleaved frames into a mono buffer (`rust_tui/src/audio.rs:55-84`). Stereo hardware therefore doubles the effective playback speed and alternates left/right samples, confusing Whisper.
- Unsigned 16-bit samples are scaled into the 0..1 range (`rust_tui/src/audio.rs:75-80`), leaving a DC bias instead of shifting to −1..1.
- The shared `err_fn` logs directly to stderr (`rust_tui/src/audio.rs:52`), corrupting the TUI whenever the driver reports xruns.

### 4. Voice capture freezes the UI
- Every key binding wraps `capture_voice` in `with_normal_terminal`, but that helper only drains pending crossterm events; it does not exit raw mode or spawn a background task (`rust_tui/src/app.rs:165-185`).
- `capture_voice` records, transcribes, and possibly runs the python fallback synchronously (`rust_tui/src/app.rs:428-459`). When `voice_enabled` is true, another capture immediately starts after each Codex response (`rust_tui/src/app.rs:412-420`). Users therefore cannot scroll, type, or cancel during the 5-10 s capture/STT cycle.

### 5. Build/config inconsistencies
- `rust_tui/Cargo.toml` still sets `edition = "2024"`, which stable toolchains reject.
- CLI extras are parsed incorrectly twice: `--codex-args` uses `split_whitespace` (`rust_tui/src/app.rs:938-943`), and `scripts/run_tui.sh` performs shell word-splitting (`CODEX_ARGS_ARRAY=($CODEX_ARGS_OVERRIDE)`), so any quoted flag with spaces (e.g., `--tool "Open Browser"`) breaks.
- `codex_session.rs` remains an unused placeholder referencing an `echo` command, which no longer matches the actual PTY implementation and causes confusion during maintenance.

### 6. Misattributed debug spam
- Whisper stderr is already suppressed during both model load and transcription via `gag::Hold` (`rust_tui/src/app.rs:480-486`, `rust_tui/src/stt.rs:5-44`).
- `with_normal_terminal` never tears down the TUI, so the earlier hypothesis that “exiting the UI” exposes Whisper output is incorrect. The real terminal noise observed today comes from `cpal` error prints and any subprocess invoked outside the gag guards (e.g., FFmpeg in the python fallback).

### 7. Additional inconsistencies
- `respond_to_terminal_queries` inside `pty_session.rs` already handles cursor/device CSI sequences, undermining the narrative that cursor probes are the current blocker.
- Latency accounting in earlier drafts double-counted Whisper model load even though `App::get_transcriber` memoizes the context (`rust_tui/src/app.rs:1103-1118`). Actual cold-start latency is dominated by the 5 s capture, the broken PTY attempt, and the repeated Codex spawn.

### 8. Housekeeping / low-severity gaps
- `env_logger` is pulled into `rust_tui/Cargo.toml` but never initialized; either wire `env_logger::init()` before logging or drop the dependency to reduce compile time.
- `run_python_transcription` scans stdout in reverse and deserializes the first `{...}` block it sees (`rust_tui/src/app.rs:765-799`). If the Python script prints braces for other reasons, the parser may grab the wrong payload; a structured channel (e.g., dedicated JSON line) would be safer.
- `scripts/run_tui.sh` now canonicalizes `WHISPER_MODEL_PATH` before changing directories (`scripts/run_tui.sh:41-46`). Relative paths like `../models/…` that relied on resolving after `cd rust_tui` (e.g., when launching from the repo root) now point outside the repo and cause the launcher to fail. Normalize the path after entering `$PROJECT_DIR` or derive it from the script directory so existing workflows keep working.
- `rust_tui/tst/test_audio.rs` prints emoji glyphs when reporting success/failure. Minimal terminals (and Codex PTYs) can’t render them reliably, so consider gating them behind a flag.
- `rust_tui/docs/PTY_FIX_PLAN.md` still shows every step as “Not started”. Keep it in sync (or fold it into the main README) so contributors know the status of the migration to `portable-pty`/async.

## CORRECTIONS TO EARLIER AUDIT TEXT
- **Whisper warm-up** – happens once per process; remove claims that every capture pays 2-3 s.
- **with_normal_terminal** – does not disable raw mode or leave the alternate screen; spam cannot be attributed to “returning to normal terminal mode.”
- **Persistent PTY** – is not “disabled”; it is attempted by default and fails because of the 500 ms bail-out and newline omissions.
- **Latency table** – should focus on capture, PTY retry cost, and Codex spawn time, not repeated model loads.

## RECOMMENDED FIXES

### Phase 0 – unblock core functionality
1. **Fix PTY read loop** *(✅ `rust_tui/src/app.rs:561-618`)*: keep reading until either `max_wait` elapses or sanitized output stops changing; only declare failure after the full timeout.
2. **Write newline in exec fallback** *(✅ `rust_tui/src/app.rs:497-549`)*: mirror the earlier fallbacks by appending `\n` (or closing stdin) so Codex actually runs.
3. **Handle multi-channel audio properly** *(✅ `rust_tui/src/audio.rs:43-145`)*: detect `device_config.channels`, downmix to mono, and normalize U16 samples into the −1..1 range.
4. **Silence driver errors inside TUI** *(✅ `rust_tui/src/audio.rs:56-96`)*: redirect `err_fn` to the existing file logger instead of stderr.
5. **Set `edition = "2021"` and align CLI parsing** *(✅ `rust_tui/Cargo.toml`, `rust_tui/src/app.rs:892-901`)*: unblock cargo builds for contributors and make `--codex-args` parsing shell-safe.

### Phase 1 – UX and reliability
1. **Move voice capture/STT off the event loop** (spawn a worker thread/future, show progress in status bar, allow cancellation).
2. **Gate persistent PTY behind a healthy default**: either default `persistent_codex` to false until the loop is fixed, or ensure the session is torn down cleanly after repeated failures.
3. **Repair argument parsing**: use `shlex`/`shellwords` semantics in both the Rust CLI and `run_tui.sh` so complex Codex flags survive transit.
4. **Remove dead `codex_session.rs`** or wire it up to the actual PTY implementation to avoid duplicate abstractions.

### Phase 2 – polish
- Improve status/state machine handling in `handle_key_event`, add visual feedback for long-running captures, and validate configuration (e.g., require `--whisper-model-path` up front when running the native pipeline).

## TESTING SUGGESTIONS
1. **PTY regression test** – Send a known prompt through `call_codex_via_session` and ensure full output arrives without falling back (unit test around a mock PTY or integration test hitting a dummy CLI).
2. **Audio fidelity test** – Feed known stereo and U16 signals through `Recorder::record` (via dependency injection/mocks) and compare the captured buffer against expected mono data.
3. **CLI quoting test** – Run `scripts/run_tui.sh` with `CODEX_ARGS_OVERRIDE='--tool "Foo Bar"'` and assert the Rust binary receives the quoted argument intact.
4. **UI responsiveness test** – Simulate rapid keypresses during voice capture to confirm the event loop remains responsive once the async refactor lands.

## SUCCESS METRICS
- **Functional PTY**: persistent sessions answer in <1 s without truncation.
- **Audio fidelity**: stereo/U16 devices produce clean mono waveforms (verified with automated tests).
- **Latency**: voice capture (5 s) + STT (<1 s) + Codex (<1 s persistent) ⇒ total <7 s by the end of Phase 1.
- **Stability**: no raw-mode corruption on panic, no stderr spam inside the TUI, and accurate CLI flag propagation.

## REMEDIATION PLAN & STATUS (NOV 2025)

| Area | Status | Notes |
| --- | --- | --- |
| PTY read loop & exec newline | ✅ Done | `rust_tui/src/app.rs:497-618` waits through quiet periods, and `write_prompt_with_newline` centralizes newline handling for interactive + exec modes. |
| Audio capture fidelity/logging | ✅ Done | `rust_tui/src/audio.rs:43-145` downmixes multi-channel inputs, normalizes U16, and logs driver errors via `log_debug`. |
| Voice capture responsiveness | ✅ Done | Background worker + polling (`rust_tui/src/app.rs:171-458`, `rust_tui/src/app.rs:1032-1255`) keep the UI responsive. |
| PTY streaming enabled | ✅ Done | `rust_tui/src/app.rs:82-111` once again drains the PTY session each frame, so streaming output appears without waiting for Enter. |
| CLI/build blockers | ✅ Done | Rust edition bumped to 2021, `shell-words` parsing added, launcher path handling fixed (`rust_tui/Cargo.toml`, `scripts/run_tui.sh:1-114`). |
| PTY documentation accuracy | ✅ Done | `rust_tui/docs/PTY_FIX_PLAN.md` now reflects the current Unix PTY implementation and remaining gaps. |
| PTY unsafe/error handling | ✅ Done | `rust_tui/src/pty_session.rs` now checks every `libc` call, closes FDs on failure, and propagates I/O errors. |
| Streaming PTY updates | ✅ Done | `app_loop` consumes session output on every tick so streaming answers show up immediately. |
| Automated tests & CI | ⚠️ In progress | Initial Rust unit tests cover audio downmix/resample paths, but broader coverage + CI workflows are still missing. |
| Architecture modularization | ⚠️ Pending | `rust_tui/src/app.rs` remains a 1.2 k-line monolith; split into modules (`ui`, `voice`, `codex`, `config`) per audit plan. |
| Resource lifecycle | ⚠️ Pending | Cached Whisper context (`rust_tui/src/app.rs:1063-1091`) never unloads; temp artifacts rely on Python cleanup—add timeouts/teardown. |
| Plugin/extensibility hooks | ⚠️ Pending | Still tightly coupled; consider strategy objects or plugin interfaces after modularization. |
| Error messaging & docs | ⚠️ Pending | Expand user-facing status/help text, add Rustdoc/README updates, and document troubleshooting steps. |
| CI/CD & linting | ⚠️ Pending | Add fmt/clippy/test workflows and pre-commit checks to prevent regressions. |
| Debug log lifecycle | ✅ Done | `rust_tui/src/app.rs:26-57` trims the temp log if it grows beyond 5 MB before each run to avoid unbounded disk usage. |

## CLAIM VS REALITY SNAPSHOT

| Area | Previously Claimed | Verified Status (Nov 2025) | Evidence |
| --- | --- | --- | --- |
| PTY read loop | “Fixed” | ✅ waits for quiet period but still tracked for future tuning | `rust_tui/src/app.rs:562-618` |
| PTY streaming | “Disabled” | ✅ re-enabled in `app_loop`, sanitized before display | `rust_tui/src/app.rs:82-111` |
| Newline handling | “Uniform” | ✅ centralized helper prevents double/ missing LF | `rust_tui/src/app.rs:640-648` |
| WHISPER_MODEL_PATH | “Resolved before cd” | ✅ unchanged | `scripts/run_tui.sh:1-114` |
| Unsafe syscalls | “Unchecked” | ✅ guarded with error handling | `rust_tui/src/pty_session.rs:1-230` |
| Infinite debug log | “Leaking” | ✅ log truncated whenever it exceeds 5 MB | `rust_tui/src/app.rs:26-57` |
| Automated tests | “None” | ⚠️ partial – audio unit tests exist, more needed | `rust_tui/src/audio.rs:120-171` |
| Monolithic `app.rs` | “Yes” | ⚠️ still pending modularisation | `rust_tui/src/app.rs` |

Use this table to track ongoing work; update statuses as fixes land so future audits start from accurate context.

---
*End of audit report*
