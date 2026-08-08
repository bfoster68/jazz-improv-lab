#!/usr/bin/env bash
# Build the Bevy app for the browser and zip it for itch.io.   macOS / Linux
#
#   ./build-wasm.sh            build + package
#   ./build-wasm.sh --serve    build, package, then serve locally to test
#
# One-time setup:
#   rustup target add wasm32-unknown-unknown
#   cargo install wasm-bindgen-cli --version <must match the wasm-bindgen crate version>
#   cargo install wasm-opt      # optional, roughly halves the download
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
root="$(dirname "$here")"
dist="$here/dist"

echo "==> building wasm (release)"
(cd "$root/rust" && cargo build --release --target wasm32-unknown-unknown -p jazz-app)

echo "==> running wasm-bindgen"
rm -rf "$dist"; mkdir -p "$dist"
wasm-bindgen --no-typescript --target web \
  --out-dir "$dist" --out-name jazz-app \
  "$root/rust/target/wasm32-unknown-unknown/release/jazz_app.wasm"

if command -v wasm-opt >/dev/null 2>&1; then
  echo "==> optimising with wasm-opt"
  wasm-opt -Oz --output "$dist/jazz-app_bg.wasm" "$dist/jazz-app_bg.wasm"
else
  echo "   (wasm-opt not found - skipping; the upload will be larger)"
fi

cp "$here/bevy-shell/index.html" "$dist/"

zip="$here/jazz-lab-bevy-itch.zip"
rm -f "$zip"
(cd "$dist" && zip -qr "$zip" .)
echo "==> wrote $zip ($(du -h "$zip" | cut -f1))"
echo "    Upload to itch.io, tick 'This file will be played in the browser'."

if [[ "${1:-}" == "--serve" ]]; then
  echo "==> serving http://localhost:8080  (Ctrl+C to stop)"
  (cd "$dist" && python3 -m http.server 8080)
fi
