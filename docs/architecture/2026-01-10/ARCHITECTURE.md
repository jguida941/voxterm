# Architecture Decision: Provider-Agnostic Backend with Claude Support

**Date**: 2026-01-10
**Status**: PROPOSED (awaiting approval)
**Author**: Claude (AI assistant)

---

## Decision

Implement a provider-agnostic backend that supports both Codex and Claude CLIs with full slash-command parity for Codex and honest capability reporting for Claude.

## Context

The current implementation has critical issues:
1. IPC mode hangs when sending prompts (Codex output computed but never delivered to TS)
2. Ctrl+R voice hotkey not wired in TypeScript
3. TS rejects unknown `/` commands instead of forwarding to provider
4. No Claude support despite user need for both providers

## Alternatives Considered

### Alternative 1: Minimal Stabilization Only
- Fix IPC bugs, wire Ctrl+R, forward slash commands
- **Pros**: Fastest to implement
- **Cons**: No Claude support, not future-proof, will need rework later

### Alternative 2: Provider-Agnostic Backend + Thin TS (RECOMMENDED)
- Add capability handshake, provider routing, Claude backend via PTY
- **Pros**: Supports both providers, clean architecture, slash command parity
- **Cons**: More work than minimal fix

### Alternative 3: Provider-Agnostic + Streaming + Phase 2B Voice
- Full implementation with streaming tokens and parallel STT
- **Pros**: Best UX, lowest latency
- **Cons**: Largest change, longer timeline

## Recommendation

**Go with Alternative 2 now**, then schedule Alternative 3 for streaming/latency compliance later.

---

## Design

### Provider Configuration

```
Priority order (highest to lowest):
1. CLI args: --provider codex|claude, --claude-cmd, --codex-cmd
2. Environment: CODEX_VOICE_PROVIDER, CLAUDE_CMD, CODEX_CMD
3. Config file: ~/.config/codex-voice/config.toml
4. Project file: ./codex_voice.toml
5. Defaults: provider=codex, claude_cmd="claude", codex_cmd="codex"
```

### Capability Handshake

On startup, Rust backend emits a `capabilities` event:

```json
{
  "event": "capabilities",
  "session_id": "abc123",
  "log_path": "/tmp/codex_voice_abc123.log",
  "mic_available": true,
  "input_device": "MacBook Pro Microphone",
  "whisper_model_loaded": true,
  "whisper_model_path": "/path/to/ggml-base.en.bin",
  "python_fallback_allowed": false,
  "providers_available": ["codex", "claude"],
  "default_provider": "codex",
  "active_provider": "codex",
  "working_dir": "/Users/user/project",
  "codex_cmd_resolved": "/opt/homebrew/bin/codex",
  "claude_cmd_resolved": "/opt/homebrew/bin/claude"
}
```

TS displays warnings for missing capabilities (no mic, no whisper model, etc.)

### Provider Backend Trait

```rust
pub trait ProviderBackend: Send + Sync {
    fn name(&self) -> &'static str;
    fn start(&self, request: ProviderRequest) -> Result<BackendJob, BackendError>;
    fn cancel(&self, job_id: JobId);
    fn supports_slash_commands(&self) -> bool;
}
```

Implementations:
- `CodexBackend`: Existing PTY session, supports all `/` commands
- `ClaudeBackend`: Spawns `claude` CLI via PTY, streams stdout as tokens, plain prompts only

### Slash Command Routing

**Rust parses all `/` commands**, not TypeScript.

| Command | Handler | Notes |
|---------|---------|-------|
| `/provider codex\|claude` | Wrapper | Switches active provider for session |
| `/codex <prompt>` | Wrapper | One-off prompt to Codex |
| `/claude <prompt>` | Wrapper | One-off prompt to Claude |
| `/voice` | Wrapper | Start voice capture |
| `/status` | Wrapper | Show status + capabilities |
| `/capabilities` | Wrapper | Emit capabilities event |
| `/help` | Wrapper | Show wrapper help |
| `/exit` | Wrapper | Exit |
| All other `/xxx` | Forward to Codex | Codex parity (only when Codex active) |

When Claude is active and user enters a Codex-specific command (e.g., `/edit`):
- Return error: "Command /edit is Codex-specific. Switch with /provider codex or use /codex /edit ..."

### TypeScript Layer (Thin)

TS responsibilities:
1. Render ANSI banner and output
2. Handle keyboard input (Ctrl+R → send `start_voice` command)
3. Display capabilities warnings
4. Forward all text input to Rust (no local `/` parsing except `/help`, `/exit`)
5. Render streamed tokens from IPC

### IPC Protocol Updates

New events:
```json
{"event": "capabilities", ...}
{"event": "provider_changed", "provider": "claude"}
{"event": "provider_error", "message": "/edit is Codex-specific..."}
```

New commands:
```json
{"cmd": "set_provider", "provider": "claude"}
{"cmd": "send_prompt", "prompt": "...", "provider": "codex"}  // optional one-off override
```

---

## Identified Bugs to Fix

| Location | Issue | Fix |
|----------|-------|-----|
| `ts_cli/src/index.ts:98` | Ctrl+R not wired | Wire keypress to `start_voice` command |
| `rust_tui/src/ipc.rs:258` | Codex output not delivered | Fix event emission in `process_codex_job` |
| `ts_cli/src/index.ts:77` | TS rejects unknown `/` commands | Forward to Rust instead |
| `rust_tui/src/ipc.rs:133` | IPC blocks during jobs | Make non-blocking or use select |
| `rust_tui/src/config.rs:324` | Silent fallback to Python STT | Report in capabilities, warn user |

---

## Risks

1. **Claude CLI behavior unknown**: Need to verify `claude` CLI accepts piped/PTY input
2. **PTY complexity**: Current PTY code has issues; may need debugging
3. **Scope creep**: Keep to Alternative 2, defer streaming to Phase 3

## Mitigations

1. Test `claude` CLI before implementation
2. Add integration tests for PTY paths
3. Explicit scope boundaries in implementation tasks

---

## Implementation Order

1. **Capability handshake** (Rust: emit event, TS: display warnings)
2. **Provider config layer** (CLI → env → config file)
3. **ProviderBackend trait + CodexBackend refactor**
4. **ClaudeBackend implementation**
5. **Slash command routing in Rust**
6. **TS thinning** (remove local `/` parsing, wire Ctrl+R)
7. **Bug fixes** (IPC blocking, output delivery)
8. **Tests + CI gates**

---

## Approval Required

This document requires explicit approval before implementation begins.

**To approve**: Reply "approved" or "proceed"
**To modify**: Specify changes needed
