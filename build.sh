#!/usr/bin/env bash
set -euo pipefail

TARGET="${1:-}"

RED='\033[0;31m'
GRN='\033[0;32m'
YLW='\033[1;33m'
BLU='\033[1;34m'
NC='\033[0m'

info()    { echo -e "${BLU}[build]${NC} $*"; }
success() { echo -e "${GRN}[build]${NC} $*"; }
die()     { echo -e "${RED}[build] ERROR:${NC} $*" >&2; exit 1; }

# Preflight checks
command -v node >/dev/null 2>&1 || die "Node.js is not installed."
command -v npm  >/dev/null 2>&1 || die "npm is not installed."

NODE_MAJOR=$(node -e "process.stdout.write(String(process.versions.node.split('.')[0]))")
if [[ "$NODE_MAJOR" -lt 18 ]]; then
  die "Node.js 18 or higher is required."
fi

# Install dependencies if needed
if [[ ! -d node_modules ]]; then
  info "Installing dependencies..."
  npm install
fi

# ELECTRON-BUILDER
case "$TARGET" in
  windows)
    info "Building Windows x64 installer..."
    npx electron-builder --win --x64 --publish never
    ;;
  linux)
    info "Building Linux x64 AppImage..."
    npx electron-builder --linux --x64 --publish never
    ;;
  linux-arm)
    info "Building Linux ARM64 AppImage..."
    npx electron-builder --linux --arm64 --publish never
    ;;
  mac)
    info "Building macOS x64 DMG..."
    npx electron-builder --mac --x64 --publish never
    ;;
  mac-arm)
    info "Building macOS ARM64 DMG..."
    npx electron-builder --mac --arm64 --publish never
    ;;
  all)
    info "Building all platforms..."
    npx electron-builder -mwl --publish never
    ;;
  "")
    success "Source files verified and ready"
    info "To create distributable, run: $0 <target>"
    info "Available targets: windows, linux, linux-arm, mac, mac-arm, all"
    exit 0
    ;;
  *)
    die "Unknown target: '${TARGET}'. Valid: windows, linux, linux-arm, mac, mac-arm, all"
    ;;
esac

echo ""
success "Build completed successfully!"