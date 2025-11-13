# Latency Remediation & Audit Plan — 2025-11-12

Comprehensive action plan derived from the latest voice latency audit. This document captures the remediation sequence so every follow-up task can be traced in daily architecture notes and CI.

## 1. Executive Summary
- Blocking capture (`Recorder::record` sleeps the entire window) and serial Whisper transcription are the primary SLA violations.
- Remaining Python fallback paths, synchronous logging, and monolithic modules further amplify latency and maintainability risks.
- Goal: bring voice→Codex round-trip latency below 750 ms on CI hardware and "a few hundred milliseconds" in production by overlapping capture/STT, adding silence-aware capture, and instrumenting every stage.

## 2. Current Implementation Status *(updated 2025-11-12)*
- [ ] Phase 2A — Incremental capture + silence stop
- [ ] Phase 2B — Chunked capture + overlapped STT (bounded SPSC queue, drop-oldest backpressure, STT worker lifespan)
- [ ] Logging/tracing upgrade (nonblocking tracing + structured metrics)
- [ ] CI perf smoke (latency gate + metrics parsing)
- [ ] Python fallback moved behind dev-only feature flag (release/CI builds error instead of falling back)
- [ ] Backpressure policy implemented + tested (drop-oldest, metrics + abort on sustained >90% utilization)


## 3. Critical Issues
1. **Blocking capture** — Replace fixed-duration recording with silence-aware, real-time capture (VAD + trailing silence cap).
2. **Serial STT** — Start Whisper once enough buffered audio exists; overlap continuous capture with transcription.
3. **Python fallback** — Remove runtime Python dependencies; keep only as documentation/reference under a feature flag disabled by default.
4. **Synchronous logging** — Switch to nonblocking `tracing` with a single open writer and structured spans for capture/STT/Codex.
5. **Monolithic modules** — Split `app.rs` and `pty_session.rs` into <300 LOC components (UI, session, voice, diagnostics, commands).
6. **Traceability gaps** — Enforce daily architecture/CHANGELOG updates via CI.

## 4. High-Impact Quick Wins (1–2 days)
- Implement silence-aware capture + chunked buffering so STT begins while capture is still running.
- Convert logging to async `tracing` with structured spans and JSON output for CI artifacts.
- Add CI gates for daily docs + CHANGELOG updates plus latency smoke tests.
- Introduce feature flags (`diagnostics`, `bench`, `dev`) to control verbose metrics without touching hot paths.

## 5. Code-Level Recommendations
- **Audio pipeline**: use cpal input callbacks feeding a lock-free ring buffer; integrate VAD (webrtc-vad/earshot/etc.) on 100–200 ms frames; keep final mono PCM interface for existing Whisper binding; retain last 500 ms context before stop.
- **Chunked STT**: background worker consumes frames via bounded channel; STT starts after ≥1 s buffered; deliver transcript when STT completes; target total latency ≈ max(capture, STT).
- **Logging**: adopt `tracing` + nonblocking file appender; emit spans (`capture_start`, `stt_start`, `codex_round`) with ms duration; export JSON logs for CI review.
- **Python fallback**: guard behind `cfg(feature = "python_fallback")` set to off by default; document differences where behavior diverges.
- **Module boundaries**: proposed layout – `ui/`, `session/`, `voice/`, `stt/`, `diag/`, `cmd/`, `integration/` directories, each ≤300 LOC target.

### 4.1 Approach Ladder & Phase Naming
- **Phase 2A — Incremental capture + silence stop**: replace `sleep(seconds)` with VAD-driven early stop. Quick win but still serial (record + STT).
- **Phase 2B — Chunked capture with overlapped STT (chosen path)**: bounded channel between recorder and STT worker; STT begins once ≥1 s of buffered audio exists and keeps consuming frames until VAD stop. This is the immediate production target.
- **Phase 3 — Full async + streaming backend**: future work once Whisper supports streaming or we adopt cloud STT; requires broader async refactor.

### 4.2 Backpressure, Frames, and Shutdown Contracts (Phase 2B)
- **Bounded queues**: recorder blocks when the channel is full; log structured warnings if backpressure lasts >N ms. Hard cap utterances at ~10–15 s on both capture and STT sides.
- **Frame invariants**: frames are fixed-duration (e.g., 160 ms) mono f32 @ 16 kHz. Document this in the module docstring and encode it in a trait so tests can feed synthetic frames.
- **Lifecycle**: recorder owns the sender; close it when VAD triggers stop or errors. STT worker reads until channel close, finalizes transcript, emits timing span, and exits cleanly (no zombie threads).
- **Fallback policy**: Python fallback stays behind `python_fallback` feature (off by default). Even when enabled, log structured events (`fallback="python"`, `reason=<error>`) and surface the path in UI/metrics so it is treated as a defect, not silent behavior.
- **Structured latency logs**: emit one JSON record per utterance with capture_ms, stt_ms, total_ms, frames, bytes, and pipeline path. CI perf gate reads this artifact (fail if total_ms > 750 on baseline input).
- **Backpressure policy**: the capture callback must never block. When the queue is full, drop the oldest frames, increment `frames_dropped_total`, and continue. If utilization stays >90% for >1 s, abort the interaction with a clear error and log `backpressure_abort=true`.

### 4.3 Test Coverage Expectations
- **VAD behavior**: unit tests for thresholds, trailing silence, and silent inputs (ensuring no ghost transcripts).
- **Backpressure/race**: stress test where frames arrive faster than STT consumes; assert no deadlock and policy matches spec (e.g., capture blocks vs dropping frames).
- **Short utterances**: verify pipeline finalizes correctly when only a few frames exist.
- **Fallback visibility**: integration test ensuring fallback logs structured event and UI reports Python path when feature enabled; test that disabling fallback yields clear error.
- **Latency SLA**: benchmark golden clips and assert P50/P95/P99 within budget (<5 s P95, <7 s P99, <10 s absolute).
- **Memory budget**: simulate slow STT and confirm RSS stays <32 MB while backpressure sheds frames according to policy.

### 4.4 Failure Modes & Recovery Hierarchy
- Introduce `TranscriptionStrategy` enum (StreamingWhisper → BatchWhisper → PythonFallback → ManualEntry) and always log which tier produced the final transcript.
- **Timeout enforcement**: cancel STT work when inference exceeds 10 s (configurable) and cascade to the next strategy.
- **Retry policy**: transient audio device failures retry up to three times with jitter before degrading.
- **Poison handling**: treat mutex poison as recoverable—recreate the component and log `recoverable_poison=true`.
- **User feedback**: UI surfaces “transcription slow” state when STT exceeds 3 s and explicitly indicates when a fallback path is active.

### 4.5 VAD Configuration & Safety Rails
- `VadConfig` defaults: `silence_threshold_db=-40`, `silence_duration_ms=500`, `max_recording_duration_ms=30_000`, `min_recording_duration_ms=200`, `adaptive_threshold=true`.
- **Adaptive baseline**: continuously sample ambient noise to adjust the threshold and log adjustments for later tuning.
- **Manual override**: allow Ctrl+R (or ESC) to force-stop capture regardless of VAD state.
- **Instrumentation**: record VAD start/stop events, reasons (silence, timeout, manual), and false-positive/-negative counters for later analysis.

### 4.6 Resource Budgets & Backpressure Policy
- **Channel size**: bounded queue of 100 frames (~10 s at 100 ms frames).
- **Memory**: pre-allocate ≤16 MB per utterance; cut off at 32 MB total to avoid OOM.
- **Duration**: enforce 30 s absolute maximum per utterance even if VAD never triggers.
- **Backpressure action**: drop oldest frames (ring-buffer semantics) once utilization >80%; emit structured warnings and abort capture if utilization stays >90% for >1 s.
- **Metrics**: expose counters for `channel_full_events`, `frames_dropped`, `utterance_too_long`, and publish them in tracing spans + CI artifacts.

### 4.7 Observability & Metrics
- Assign a UUID per utterance; include it in all tracing spans, metrics, and UI events.
- Emit `VoiceLatencyMetrics` JSON per utterance (capture_ms, stt_ms, overlap_ms, total_ms, frames, bytes, fallback_strategy, channel_pressure).
- Track rolling P50/P95/P99 latencies and include them in perf-smoke summaries; auto-fail when P95 ≥5 s on golden samples.
- Capture CPU flamegraphs automatically (debug builds) when total latency exceeds 5 s to aid regression analysis.

### 4.8 Testing & CI Strategy
- **Latency benchmark**: deterministic clips checked into repo; CI asserts SLA and uploads metrics artifact.
- **Stress test**: run ≥100 sequential utterances to ensure no leaks or deadlocks under sustained load.
- **Noise/fuzz tests**: feed silence/random/noisy audio to verify VAD stability and absence of panics.
- **Cross-platform matrix**: run capture + latency smoke on macOS and Ubuntu (Windows optional follow-up).
- **Backpressure regression**: deliberately throttle STT worker to trigger frame dropping and ensure telemetry + UX match spec.

### 4.9 CI Enforcement Hooks
- `perf_smoke` workflow parses latency metrics JSON and fails when SLAs break (P95 ≥5 s, total ≥10 s, fallback triggered unexpectedly).
- `memory_guard` workflow measures peak RSS (<32 MB) during stress runs via `valgrind/heaptrack` (Linux) or `leaks` (macOS).
- CI summary comment posts latest percentiles, frames dropped, and fallback counts for reviewer visibility.
- CI job `perf_smoke` must parse per-utterance metrics from logs and fail when P95 total_latency_ms for the golden sample set exceeds 750 ms on CI hardware.

### 4.10 State Machine & Lifecycle
- Interaction state machine: `Idle → Capturing → Transcribing → Finalizing → Done`, with explicit transitions for success, user cancel, error, and timeout paths.
- Recorder owns the transition into `Capturing` and signals `Transcribing` once enough buffered audio exists; STT worker transitions to `Finalizing` when the sender closes.
- User cancel (Ctrl+C / ESC) sends a shutdown signal that drains the channel, logs `cancelled=true`, and returns a friendly message without leaving orphaned threads.
- Failures emit the final state + reason so CI/telemetry can spot stuck transitions.

### Per-Interaction State Machine Diagram

```
Idle ──Ctrl+R────▶ Capturing ──VAD silence / timeout────▶ Transcribing
 │                       │                                  │
 │ (errors)              │ (user cancel)                    │ (user cancel / timeout)
 ▼                       ▼                                  ▼
Done ◀─────── Finalizing ◀───────┴───────────◀──────────────┘
```

- **Idle**: no active capture.
- **Capturing**: audio callback enqueues frames; VAD monitors speech/silence.
- **Transcribing**: STT worker processes frames; may still receive more until sender closes.
- **Finalizing**: capture stopped, STT flushing tail audio.
- **Done**: transcript or error returned; resources released.

### 4.11 Config Surface & Deployment Profiles
- Config keys (with defaults) exposed in `AppConfig`: `voice.max_capture_ms`, `voice.silence_tail_ms`, `voice.ring_buffer_ms`, `voice.min_speech_ms_before_stt`, `voice.allow_python_fallback`, `voice.max_concurrent_sessions`.
- Dev profile: may enable Python fallback + looser latency assertions, but logs conspicuous warnings.
- Release/CI profile: Python fallback feature disabled, strict latency/memory gates enforced, and perf metrics uploaded.

Example TOML:

```toml
[voice]
sample_rate = 16000
max_capture_ms = 10000
silence_tail_ms = 500
min_speech_ms_before_stt_start = 300
buffer_ms = 10000
channel_capacity = 100
stt_timeout_ms = 10000
allow_python_fallback = false
```

### 4.12 Concurrency Scope
- Primary mode: single interactive session (one recorder + one STT worker). Architecture supports extension to N sessions by instantiating separate ring buffers/workers per session and capping `max_concurrent_sessions`.
- Document and enforce the limit so Whisper threads cannot starve the system on multi-session builds.

### 4.13 Implementation Timeline (≈15 days)
1. Design (3 d): finalize module boundaries, resource limits, recovery matrix.
2. Implementation (7 d): capture worker, STT worker, VAD, metrics, fallback scaffolding.
3. Testing (3 d): latency benchmarks, stress/fuzz, cross-platform validation.
4. CI wiring (2 d): perf + memory gates, artifact uploads, documentation.

## 6. Documentation & Repo Workflow
- **Root**: retain indexes only (`README.md`, `PROJECT_OVERVIEW.md`, `master_index.md`, `agents.md`, `CHANGELOG.md`).
- **Daily folders**: `docs/architecture/YYYY-MM-DD/ARCHITECTURE.md` and `CHANGELOG.md` remain the single source of truth; CI blocks merges without updates.
- **Templates**: adopt ADR-style sections (Goal, Decision, Alternatives, Tradeoffs, Benchmarks, Risks, Approvals) for each daily architecture entry.

## 7. CI/CD Enhancements
- Lint: `cargo fmt -- --check`, `cargo clippy -D warnings`, link checker (lychee) optional.
- Build/Test: `cargo nextest` for unit/integration; release build check.
- Coverage: `cargo llvm-cov --fail-under-lines 75`.
- Security: `cargo audit`, `cargo deny`.
- Docs gate: script ensures latest `docs/architecture/YYYY-MM-DD` + root `CHANGELOG.md` mutated.
- Perf smoke: run capture→STT once (fixed sample) and fail PR if ≥750 ms.
- Concurrency: cancel previous workflow runs per branch push.

## 8. Space Usage & Hygiene
- Identify large artifacts via `du -hx -d 3 | sort -h | tail -n 30` and `find … -printf '%s\t%p' | sort -n | tail -n 50 | numfmt`.
- Ignore heavy dirs: `/target`, `/models`, `/logs`, `/data/*.wav`, `.venv`, etc.; use Git LFS for unavoidable binaries (wav/mp3/bin/onnx/ggml/model).

## 9. Verification Harness (Local)
1. Confirm today’s architecture folder exists and `master_index.md` references it.
2. Run `lychee` link check on docs.
3. Execute all fenced bash blocks via `scripts/run-doc-commands.py` (SKIP_CI when needed).
4. Voice smoke test: `RUST_LOG=info cargo run --features diagnostics -- --seconds 1 --sample tests/samples/short.wav`; parse JSON logs with `scripts/check-latency.py --budget-ms 750`.
5. Security scans: `cargo audit`, `cargo deny check`.

## 10. Automation Instructions for Codex
- **Artifact gathering**: export governance docs, workflows, `docs/architecture/*` (last 14 days), `rust_tui` sources, scripts, tests; run size report, link check, nextest + llvm-cov, voice smoke; package results.
- **Logging refactor**: create `diag/` module for tracing init, apply spans, add integration test, update docs/architecture + changelog.
- **Silence-aware capture**: refactor Recorder to cpal callback + VAD; add tests for VAD thresholds.
- **Chunked STT**: implement bounded channel, overlapping STT, benchmarks vs Phase 1.

## 11. Acceptance Criteria
- Voice→Codex latency ≤750 ms on CI hardware for golden sample; aspirational “few hundred ms.”
- No Python runtime fallback unless feature enabled.
- No module >300 LOC in `rust_tui/src`.
- CI green on lint/build/test/coverage/audit/docs/perf.
- Every PR touching code updates daily architecture folder + CHANGELOG.

*Owner:* Platform & Voice Infrastructure  \
*Approved by:* Project Owner (2025-11-12)  \
*Linked Notes:* [`docs/architecture/2025-11-12/`](../architecture/2025-11-12/)  \
*Next Review:* Start of 2025-11-13 session
