# Backlog

- Check if supported on native windows (have parrells, will it work native on same system.

- Remove the Python fallback by making the native Whisper path required and packaging models cleanly.
- Add a clear warning when Python fallback is active (status line + logs already indicate the pipeline).
- Decide on Windows support for the Rust overlay (PTY abstraction or WSL-only guidance).
- Remove unused legacy CLI code paths once the Rust overlay covers all workflows.
