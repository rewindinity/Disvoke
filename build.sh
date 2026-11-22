#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-native}"

case "$TARGET" in
  native)
    cargo build --release
    ;;
  linux)
    cargo build --release --target x86_64-unknown-linux-gnu
    ;;
  linux-arm)
    cargo build --release --target aarch64-unknown-linux-gnu
    ;;
  windows)
    cargo build --release --target x86_64-pc-windows-gnu
    ;;
  windows-arm)
    cargo build --release --target aarch64-pc-windows-msvc
    ;;
  mac)
    cargo build --release --target x86_64-apple-darwin
    ;;
  mac-arm)
    cargo build --release --target aarch64-apple-darwin
    ;;
  *)
    echo "Unknown target: '$TARGET'. Valid: native, linux, linux-arm, windows, windows-arm, mac, mac-arm" >&2
    exit 1
    ;;
esac
