#!/bin/bash
# Autonomix installation script

set -e

PREFIX="${PREFIX:-/usr/local}"
INSTALL_USER="${SUDO_USER:-$USER}"

echo "Installing Autonomix..."

# Install Python package
pip3 install --prefix="$PREFIX" .

# Install desktop file
install -Dm644 data/autonomix.desktop "$PREFIX/share/applications/autonomix.desktop"

# Install icon (multiple sizes)
install -Dm644 autonomix/resources/autonomix.svg "$PREFIX/share/icons/hicolor/scalable/apps/autonomix.svg"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
fi

echo "Autonomix installed successfully!"
echo "You can run it with: autonomix"
