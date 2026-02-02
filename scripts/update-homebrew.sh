#!/usr/bin/env bash
#
# Update Homebrew tap for VoxTerm
# Usage: ./scripts/update-homebrew.sh <version>
# Example: ./scripts/update-homebrew.sh 1.0.33
#
set -euo pipefail

VERSION="${1:-}"
if [[ -z "$VERSION" ]]; then
    echo "Usage: $0 <version>"
    echo "Example: $0 1.0.33"
    exit 1
fi

TAG="v$VERSION"
HOMEBREW_REPO="${HOMEBREW_VOXTERM_PATH:-$HOME/testing_upgrade/homebrew-voxterm}"
FORMULA="$HOMEBREW_REPO/Formula/voxterm.rb"

echo "=== Updating Homebrew tap for $TAG ==="

# Check Homebrew repo exists
if [[ ! -d "$HOMEBREW_REPO" ]]; then
    echo "Error: Homebrew repo not found at $HOMEBREW_REPO"
    echo "Set HOMEBREW_VOXTERM_PATH or clone the repo first."
    exit 1
fi

# Get SHA256 of release tarball
TARBALL_URL="https://github.com/jguida941/voxterm/archive/refs/tags/$TAG.tar.gz"
echo "Fetching SHA256 for $TARBALL_URL..."
SHA256=$(curl -sL "$TARBALL_URL" | shasum -a 256 | cut -d' ' -f1)

if [[ -z "$SHA256" || "$SHA256" == "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855" ]]; then
    echo "Error: Failed to get SHA256 (empty tarball or tag doesn't exist)"
    echo "Make sure tag $TAG exists on GitHub."
    exit 1
fi

echo "SHA256: $SHA256"

# Update formula
echo "Updating $FORMULA..."
cd "$HOMEBREW_REPO"

# Update version
sed -i '' "s|url \"https://github.com/jguida941/voxterm/archive/refs/tags/v[0-9.]*\.tar\.gz\"|url \"$TARBALL_URL\"|" "$FORMULA"
sed -i '' "s|version \"[0-9.]*\"|version \"$VERSION\"|" "$FORMULA"
sed -i '' "s|sha256 \"[a-f0-9]*\"|sha256 \"$SHA256\"|" "$FORMULA"

# Show diff
echo ""
echo "Changes:"
git diff "$FORMULA"
echo ""

# Commit and push
read -p "Commit and push these changes? (y/n) " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]; then
    git add "$FORMULA"
    git commit -m "Update to v$VERSION"
    git push origin main
    echo ""
    echo "=== Homebrew tap updated ==="
    echo "Users can now run: brew update && brew upgrade voxterm"
else
    echo "Changes not committed. Run manually:"
    echo "  cd $HOMEBREW_REPO"
    echo "  git add Formula/voxterm.rb"
    echo "  git commit -m 'Update to v$VERSION'"
    echo "  git push origin main"
fi
