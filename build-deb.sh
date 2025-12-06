#!/bin/bash
set -e

# Build script for Autonomix packages
VERSION="0.3.2"
ARCH="amd64"

echo "Building Autonomix v${VERSION}..."

# Build release binary
cargo build --release

# Create package directory structure
PKG_DIR="pkg-build/autonomix_${VERSION}_${ARCH}"
rm -rf "$PKG_DIR"
mkdir -p "$PKG_DIR/DEBIAN"
mkdir -p "$PKG_DIR/usr/bin"
mkdir -p "$PKG_DIR/usr/share/applications"
mkdir -p "$PKG_DIR/usr/share/icons/hicolor/scalable/apps"

# Copy binary
cp target/release/autonomix "$PKG_DIR/usr/bin/"

# Copy desktop file and icon
cp resources/io.github.plebone.autonomix.desktop "$PKG_DIR/usr/share/applications/"
cp resources/io.github.plebone.autonomix.svg "$PKG_DIR/usr/share/icons/hicolor/scalable/apps/"

# Create control file
cat > "$PKG_DIR/DEBIAN/control" << EOF
Package: autonomix
Version: ${VERSION}
Section: utils
Priority: optional
Architecture: ${ARCH}
Depends: libqt5core5a, libqt5gui5, libqt5qml5, libqt5quick5
Maintainer: PlebOne <plebone@example.com>
Description: GitHub Release Manager for Linux
 A Linux package manager for GitHub releases, similar to Obtainium for Android.
 Features:
  - Track GitHub releases from any repository
  - Install .deb, .rpm, AppImage, Flatpak, and Snap packages
  - Check for updates with one click
  - Self-updates included
EOF

# Build the deb package
dpkg-deb --build "$PKG_DIR"

echo "Package created: ${PKG_DIR}.deb"
echo ""
echo "To install: sudo dpkg -i ${PKG_DIR}.deb"
