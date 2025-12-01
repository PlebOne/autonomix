#!/bin/bash
# Build script for Autonomix packages

set -e

VERSION="0.1.0"
PKG_NAME="autonomix"

echo "=== Autonomix Package Builder ==="
echo ""

# Create build directory
BUILD_DIR="$(pwd)/build-pkg"
rm -rf "$BUILD_DIR"
mkdir -p "$BUILD_DIR"

build_deb() {
    echo "Building Debian package..."
    
    if ! command -v dpkg-buildpackage &> /dev/null; then
        echo "Error: dpkg-buildpackage not found. Install with: sudo apt install build-essential devscripts debhelper dh-python"
        return 1
    fi
    
    # Build the package
    dpkg-buildpackage -us -uc -b
    
    # Move the .deb to build directory
    mv ../*.deb "$BUILD_DIR/" 2>/dev/null || true
    mv ../*.buildinfo "$BUILD_DIR/" 2>/dev/null || true
    mv ../*.changes "$BUILD_DIR/" 2>/dev/null || true
    
    echo "Debian package built: $BUILD_DIR/"
}

build_rpm() {
    echo "Building RPM package..."
    
    if ! command -v rpmbuild &> /dev/null; then
        echo "Error: rpmbuild not found. Install with: sudo dnf install rpm-build"
        return 1
    fi
    
    # Create tarball
    TARBALL="$PKG_NAME-$VERSION.tar.gz"
    mkdir -p "$BUILD_DIR/rpmbuild/{SOURCES,SPECS,BUILD,RPMS,SRPMS}"
    
    # Create source tarball
    git archive --format=tar.gz --prefix="$PKG_NAME-$VERSION/" HEAD > "$BUILD_DIR/rpmbuild/SOURCES/$TARBALL"
    
    # Copy spec file
    cp autonomix.spec "$BUILD_DIR/rpmbuild/SPECS/"
    
    # Build RPM
    rpmbuild --define "_topdir $BUILD_DIR/rpmbuild" -ba "$BUILD_DIR/rpmbuild/SPECS/autonomix.spec"
    
    # Copy built RPMs to build directory
    find "$BUILD_DIR/rpmbuild/RPMS" -name "*.rpm" -exec cp {} "$BUILD_DIR/" \;
    
    echo "RPM package built: $BUILD_DIR/"
}

build_wheel() {
    echo "Building Python wheel..."
    
    python3 -m pip install --upgrade build
    python3 -m build --outdir "$BUILD_DIR"
    
    echo "Python wheel built: $BUILD_DIR/"
}

# Parse arguments
case "${1:-all}" in
    deb)
        build_deb
        ;;
    rpm)
        build_rpm
        ;;
    wheel)
        build_wheel
        ;;
    all)
        build_wheel
        build_deb 2>/dev/null || echo "Skipping deb build (not on Debian-based system)"
        build_rpm 2>/dev/null || echo "Skipping rpm build (not on RPM-based system)"
        ;;
    *)
        echo "Usage: $0 [deb|rpm|wheel|all]"
        exit 1
        ;;
esac

echo ""
echo "=== Build complete ==="
ls -la "$BUILD_DIR"
