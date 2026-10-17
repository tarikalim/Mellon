#!/bin/sh
# Build release and install "/Applications/Mellon.app".
# scripts/bundle.sh <binary> <out dir> packages a prebuilt binary without installing.
# Icon: assets/icon.svg is the source; assets/AppIcon.icns and assets/mellon.png are rendered from it.
# The app's binary is the only one: Claude Code's MCP entry runs it too, so one bundle updates both.
set -e
cd "$(dirname "$0")/.."
bin="${1:-target/release/mellon}"
app="${2:-/Applications}/Mellon.app"
lsregister=/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister
[ -n "$1" ] || cargo build --release -j 4
# Drop the pre-rename bundles so only one app remains.
if [ -z "$1" ]; then
  for old in /Applications/envhub.app "$HOME/Applications/envhub.app"; do
    [ -d "$old" ] && { "$lsregister" -u "$old" || true; rm -rf "$old"; }
  done
fi
# Updated in place, not deleted: deleting it dropped it from Spotlight until macOS indexed it again.
# The binary is moved over, not copied over, so a running `mellon mcp` keeps its old file.
mkdir -p "$app/Contents/MacOS" "$app/Contents/Resources"
cp "$bin" "$app/Contents/MacOS/mellon.new"
mv -f "$app/Contents/MacOS/mellon.new" "$app/Contents/MacOS/mellon"
cp assets/AppIcon.icns "$app/Contents/Resources/"
version=$(grep -m1 '^version' Cargo.toml | cut -d'"' -f2)
cat > "$app/Contents/Info.plist" <<PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0"><dict>
  <key>CFBundleName</key><string>Mellon</string>
  <key>CFBundleDisplayName</key><string>Mellon</string>
  <key>CFBundleIdentifier</key><string>com.tarikalim.mellon</string>
  <key>CFBundleExecutable</key><string>mellon</string>
  <key>CFBundleIconFile</key><string>AppIcon</string>
  <key>CFBundlePackageType</key><string>APPL</string>
  <key>CFBundleShortVersionString</key><string>$version</string>
  <key>CFBundleVersion</key><string>$version</string>
  <key>LSMinimumSystemVersion</key><string>12.0</string>
  <key>LSApplicationCategoryType</key><string>public.app-category.developer-tools</string>
  <key>NSHighResolutionCapable</key><true/>
  <key>NSSupportsAutomaticGraphicsSwitching</key><true/>
</dict></plist>
PLIST
codesign --force --deep -s - "$app"
# Updated in place, the bundle keeps its date and macOS keeps the cached icon; a new date refreshes it.
touch "$app"
[ -n "$1" ] || { "$lsregister" -f "$app"; mdimport "$app"; }
echo "installed: $app"
