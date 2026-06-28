#!/usr/bin/env bash
# Build Rust core for all Apple targets and copy artifacts to archival-app/Libs/
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
CORE_DIR="$(cd "$SCRIPT_DIR/../../archival-core" && pwd)"
LIBS_DIR="$(cd "$SCRIPT_DIR/../Libs" && pwd)"

# Ensure cargo is on PATH
source "$HOME/.cargo/env" 2>/dev/null || true

echo "Building archival-core…"
cd "$CORE_DIR"

cargo build --release \
    --target aarch64-apple-darwin

cargo build --release \
    --target aarch64-apple-ios

cargo build --release \
    --target aarch64-apple-ios-sim

echo "Copying artifacts…"
cp "target/aarch64-apple-darwin/release/libarchival_core.dylib" "$LIBS_DIR/macos/"
cp "target/aarch64-apple-darwin/release/libarchival_core.a"     "$LIBS_DIR/macos/"
cp "target/aarch64-apple-ios/release/libarchival_core.a"         "$LIBS_DIR/ios/"

echo "Done. Artifacts at:"
ls -lh "$LIBS_DIR/macos/" "$LIBS_DIR/ios/"

# Future: bundle as XCFramework
# xcodebuild -create-xcframework \
#     -library "$LIBS_DIR/macos/libarchival_core.dylib" -headers "$CORE_DIR/include/" \
#     -library "$LIBS_DIR/ios/libarchival_core.a"       -headers "$CORE_DIR/include/" \
#     -output "$LIBS_DIR/ArchivalCore.xcframework"
