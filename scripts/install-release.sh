#!/bin/sh
# Installs the latest release into /Applications and clears the download's quarantine flag:
# Mellon is not notarized, so macOS would otherwise refuse to open it. Needs the gh CLI.
set -e
repo="${MELLON_REPO:-tarikalim/Mellon}"
dest="${1:-/Applications}"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

gh release download -R "$repo" -p 'Mellon-*-macos-universal.zip' -D "$tmp"
ditto -x -k "$tmp"/Mellon-*-macos-universal.zip "$tmp"
rm -rf "$dest/Mellon.app"
mv "$tmp/Mellon.app" "$dest/Mellon.app"
xattr -dr com.apple.quarantine "$dest/Mellon.app"
echo "installed: $dest/Mellon.app"
