# Troubleshooting Install/Update Issues

Use this guide for installation conflicts, stale versions, PATH shadowing, and
Homebrew update problems.

## Contents

- [Homebrew link conflict](#homebrew-link-conflict)
- [Wrong version after update](#wrong-version-after-update)
- [See Also](#see-also)

## Homebrew link conflict

If `brew install voiceterm` fails because the command already exists:

```bash
brew link --overwrite voiceterm
```

## Wrong version after update

Start with a normal update:

```bash
brew update
brew upgrade voiceterm
```

If Homebrew still shows an older version, refresh taps:

```bash
brew untap jguida941/voiceterm 2>/dev/null || true
brew untap jguida941/homebrew-voiceterm 2>/dev/null || true
brew tap jguida941/voiceterm
brew update
brew info voiceterm
```

If still stale, clear cache and reinstall:

```bash
rm -f "$(brew --cache)"/voiceterm--*
brew reinstall voiceterm
```

If `voiceterm --version` is still old, check PATH shadowing:

```bash
which -a voiceterm
```

Common shadow path from local install:

```bash
mv ~/.local/bin/voiceterm ~/.local/bin/voiceterm.bak
hash -r
```

Check for repo-local wrapper too:

```bash
ls -l ~/voiceterm/bin/voiceterm 2>/dev/null
```

Relink Homebrew and clear shell cache:

```bash
brew unlink voiceterm && brew link --overwrite voiceterm
hash -r
```

Verify Homebrew binary directly:

```bash
$(brew --prefix)/opt/voiceterm/libexec/src/target/release/voiceterm --version
```

## See Also

| Topic | Link |
|-------|------|
| Troubleshooting hub | [TROUBLESHOOTING.md](TROUBLESHOOTING.md) |
| Install guide | [INSTALL.md](INSTALL.md) |
