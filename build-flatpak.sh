#!/bin/bash
set -e

# Build script for Autonomix Flatpak package
VERSION="0.3.1"
APP_ID="io.github.plebone.autonomix"

echo "Building Autonomix Flatpak v${VERSION}..."

# Flatpak build directory
FLATPAK_DIR="flatpak-build"

# Check if flatpak-builder is available
if ! command -v flatpak-builder &> /dev/null; then
    echo "flatpak-builder not found. Install with: sudo dnf install flatpak-builder"
    exit 1
fi

# Install required runtime and SDK if not present
echo "Ensuring runtimes are installed..."
flatpak install -y flathub org.kde.Platform//6.8 org.kde.Sdk//6.8 2>/dev/null || true
flatpak install -y flathub org.freedesktop.Sdk.Extension.rust-stable//24.08 2>/dev/null || true

# Build the flatpak (compiles Rust inside the sandbox)
echo "Building Flatpak (this compiles Rust inside the sandbox)..."
cd "$FLATPAK_DIR"
flatpak-builder --force-clean --repo=repo --install-deps-from=flathub build-dir "${APP_ID}.yml"

# Create the bundle
mkdir -p ../pkg-build
flatpak build-bundle repo "../pkg-build/${APP_ID}-${VERSION}.flatpak" "${APP_ID}"

echo ""
echo "Flatpak bundle created: pkg-build/${APP_ID}-${VERSION}.flatpak"
echo ""
echo "To install: flatpak install pkg-build/${APP_ID}-${VERSION}.flatpak"
