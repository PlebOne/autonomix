#!/bin/bash
# Autonomix uninstallation script

set -e

PREFIX="${PREFIX:-/usr/local}"

echo "Uninstalling Autonomix..."

# Remove Python package
pip3 uninstall -y autonomix 2>/dev/null || true

# Remove desktop file
rm -f "$PREFIX/share/applications/autonomix.desktop"

# Remove icon
rm -f "$PREFIX/share/icons/hicolor/scalable/apps/autonomix.svg"

# Update desktop database
if command -v update-desktop-database &> /dev/null; then
    update-desktop-database "$PREFIX/share/applications" 2>/dev/null || true
fi

# Update icon cache
if command -v gtk-update-icon-cache &> /dev/null; then
    gtk-update-icon-cache -f -t "$PREFIX/share/icons/hicolor" 2>/dev/null || true
fi

echo "Autonomix uninstalled successfully!"
