#!/bin/bash
set -e

# Build script for Autonomix Flatpak package
VERSION="0.3.0"
APP_ID="io.github.plebone.autonomix"

echo "Building Autonomix Flatpak v${VERSION}..."

# Build release binary first
cargo build --release

# Create flatpak build directory
FLATPAK_DIR="flatpak-build"
rm -rf "$FLATPAK_DIR"
mkdir -p "$FLATPAK_DIR"

# Create flatpak manifest
cat > "$FLATPAK_DIR/${APP_ID}.yml" << EOF
app-id: ${APP_ID}
runtime: org.kde.Platform
runtime-version: '6.7'
sdk: org.kde.Sdk
command: autonomix

finish-args:
  - --share=ipc
  - --share=network
  - --socket=fallback-x11
  - --socket=wayland
  - --filesystem=home
  - --talk-name=org.freedesktop.Flatpak

modules:
  - name: autonomix
    buildsystem: simple
    build-commands:
      - install -Dm755 autonomix /app/bin/autonomix
      - install -Dm644 ${APP_ID}.desktop /app/share/applications/${APP_ID}.desktop
      - install -Dm644 ${APP_ID}.svg /app/share/icons/hicolor/scalable/apps/${APP_ID}.svg || true
    sources:
      - type: file
        path: ../target/release/autonomix
      - type: file
        path: ../resources/${APP_ID}.desktop
      - type: file
        path: ../resources/${APP_ID}.svg
EOF

# Check if flatpak-builder is available
if ! command -v flatpak-builder &> /dev/null; then
    echo "flatpak-builder not found. Install with: sudo dnf install flatpak-builder"
    echo "Manifest created at: $FLATPAK_DIR/${APP_ID}.yml"
    exit 1
fi

# Install required runtime if not present
flatpak install -y flathub org.kde.Platform//6.7 org.kde.Sdk//6.7 2>/dev/null || true

# Build the flatpak
cd "$FLATPAK_DIR"
flatpak-builder --force-clean --repo=repo build-dir "${APP_ID}.yml"

# Create the bundle
mkdir -p ../pkg-build
flatpak build-bundle repo "../pkg-build/${APP_ID}-${VERSION}.flatpak" "${APP_ID}"

echo ""
echo "Flatpak bundle created: pkg-build/${APP_ID}-${VERSION}.flatpak"
echo ""
echo "To install: flatpak install pkg-build/${APP_ID}-${VERSION}.flatpak"
