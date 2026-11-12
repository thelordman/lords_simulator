#!/usr/bin/env bash
# Wraps a built binary in "Lord's Simulator.app" and zips it. Runs on macOS only.
#
# Usage: bundle.sh <binary> <version> <output-dir>
set -euo pipefail

binary=$1
version=$2
out_dir=$3

here=$(cd "$(dirname "$0")" && pwd)
repo=$(cd "$here/../.." && pwd)

app="$out_dir/Lord's Simulator.app"
contents="$app/Contents"
rm -rf "$app"
mkdir -p "$contents/MacOS" "$contents/Resources"

cp "$binary" "$contents/MacOS/lords_simulator"
sed "s/@VERSION@/$version/g" "$here/Info.plist" > "$contents/Info.plist"
plutil -lint "$contents/Info.plist"

cp "$repo/LICENSE-MIT" "$repo/LICENSE-APACHE" "$contents/Resources/"
cp "$repo/assets/fonts/Phosphor-LICENSE.txt" "$contents/Resources/LICENSE-Phosphor.txt"

work=$(mktemp -d)
iconset="$work/AppIcon.iconset"
mkdir "$iconset"
for size in 16 32 128 256 512; do
    double=$((size * 2))
    sips -z "$size" "$size" "$repo/assets/icon-1024.png" \
        --out "$iconset/icon_${size}x${size}.png" > /dev/null
    sips -z "$double" "$double" "$repo/assets/icon-1024.png" \
        --out "$iconset/icon_${size}x${size}@2x.png" > /dev/null
done
iconutil --convert icns --output "$contents/Resources/AppIcon.icns" "$iconset"
rm -rf "$work"

# Ad-hoc signature: Apple Silicon refuses to run unsigned code. Replace "-" with a
# Developer ID identity here to sign properly.
codesign --force --sign - "$app"
codesign --verify --strict "$app"

ditto -c -k --keepParent "$app" "$out_dir/Lords-Simulator-$version-macos-universal.zip"
rm -rf "$app"
