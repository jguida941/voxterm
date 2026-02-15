# Troubleshooting Backend Issues

Use this guide for backend-runtime issues with Codex/Claude, including prompt
detection, queueing, and process cleanup.

## Contents

- [Codex not responding](#codex-not-responding)
- [Many codex/claude processes remain after quitting](#many-codexclaude-processes-remain-after-quitting)
- [Auto-voice not triggering](#auto-voice-not-triggering)
- [Transcript queued (N)](#transcript-queued-n)
- [Transcript queue full (oldest dropped)](#transcript-queue-full-oldest-dropped)
- [See Also](#see-also)

## Codex not responding

1. Verify backend CLI exists:

   ```bash
   which codex
   which claude
   ```

2. Verify authentication:

   ```bash
   codex login
   # or
   claude login
   ```

   Or from VoiceTerm:

   ```bash
   voiceterm --login --codex
   voiceterm --login --claude
   ```

3. Restart `voiceterm` if the session is stuck.

## Many codex/claude processes remain after quitting

Recent builds terminate backend process groups and reap child processes on exit.
If you still observe leftovers:

1. Confirm version:

   ```bash
   voiceterm --version
   ```

2. Check for orphaned backend processes:

   ```bash
   ps -axo ppid,pid,command | egrep '(^ *1 .*\\b(codex|claude)\\b)'
   ```

3. If orphans remain, report with:
- `voiceterm --version`
- terminal/IDE name + version
- launch command
- relevant `${TMPDIR}/voiceterm_tui.log` lines

## Auto-voice not triggering

Auto-voice waits for prompt readiness before listening again.

1. Override prompt detection for your shell/backend prompt:

   ```bash
   voiceterm --prompt-regex '^codex> $'
   ```

2. Enable prompt logging:

   ```bash
   voiceterm --prompt-log /tmp/voiceterm_prompt.log
   ```

3. Inspect the prompt log and adjust regex.

## Transcript queued (N)

Backend output is still streaming, so transcript injection is deferred.

1. Wait for prompt return.
2. If urgent, stop current generation (`Ctrl+C`) and retry.
3. If this happens often, tune prompt detection (`--prompt-regex`) and
   transcript timeout (`--transcript-idle-ms`).

## Transcript queue full (oldest dropped)

You recorded more items than queue capacity while backend was busy.

1. Pause speaking until prompt returns.
2. Use shorter chunks.
3. Prefer `insert` mode if you need manual pacing.

## See Also

| Topic | Link |
|-------|------|
| Troubleshooting hub | [TROUBLESHOOTING.md](TROUBLESHOOTING.md) |
| CLI flags | [CLI_FLAGS.md](CLI_FLAGS.md) |
| Usage (backend support matrix) | [USAGE.md#backend-support](USAGE.md#backend-support) |
