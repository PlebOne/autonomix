#!/bin/bash
set -e

# Build script for Autonomix RPM package
VERSION="0.3.4"
RELEASE="1"
ARCH="x86_64"

echo "Building Autonomix RPM v${VERSION}..."

# Build release binary
cargo build --release

# Create RPM build directory structure
RPM_BUILD_DIR="$HOME/rpmbuild"
mkdir -p "$RPM_BUILD_DIR"/{BUILD,RPMS,SOURCES,SPECS,SRPMS}

# Create tarball for sources
TARBALL_DIR="autonomix-${VERSION}"
mkdir -p "$TARBALL_DIR"
cp target/release/autonomix "$TARBALL_DIR/"
cp resources/io.github.plebone.autonomix.desktop "$TARBALL_DIR/"
cp resources/io.github.plebone.autonomix.svg "$TARBALL_DIR/" 2>/dev/null || echo "No icon file found"
cp resources/io.github.plebone.autonomix.metainfo.xml "$TARBALL_DIR/" 2>/dev/null || echo "No metainfo file found"
tar czf "$RPM_BUILD_DIR/SOURCES/autonomix-${VERSION}.tar.gz" "$TARBALL_DIR"
rm -rf "$TARBALL_DIR"

# Create spec file
cat > "$RPM_BUILD_DIR/SPECS/autonomix.spec" << EOF
Name:           autonomix
Version:        ${VERSION}
Release:        ${RELEASE}%{?dist}
Summary:        GitHub Release Manager for Linux

License:        MIT
URL:            https://github.com/PlebOne/autonomix
Source0:        autonomix-%{version}.tar.gz

# Disable debug packages since we're using pre-built binary
%global debug_package %{nil}

Requires:       qt6-qtbase qt6-qtdeclarative

%description
A Linux package manager for GitHub releases, similar to Obtainium for Android.
Features:
 - Track GitHub releases from any repository
 - Install .deb, .rpm, AppImage, Flatpak, and Snap packages
 - Check for updates with one click
 - Self-updates included

%prep
%setup -q

%install
mkdir -p %{buildroot}%{_bindir}
mkdir -p %{buildroot}%{_datadir}/applications
mkdir -p %{buildroot}%{_datadir}/icons/hicolor/scalable/apps
mkdir -p %{buildroot}%{_datadir}/metainfo

install -m 755 autonomix %{buildroot}%{_bindir}/autonomix
install -m 644 io.github.plebone.autonomix.desktop %{buildroot}%{_datadir}/applications/
install -m 644 io.github.plebone.autonomix.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/ 2>/dev/null || :
install -m 644 io.github.plebone.autonomix.metainfo.xml %{buildroot}%{_datadir}/metainfo/ 2>/dev/null || :

%post
/usr/bin/gtk-update-icon-cache -f -t %{_datadir}/icons/hicolor &>/dev/null || :
/usr/bin/update-desktop-database %{_datadir}/applications &>/dev/null || :

%postun
/usr/bin/gtk-update-icon-cache -f -t %{_datadir}/icons/hicolor &>/dev/null || :
/usr/bin/update-desktop-database %{_datadir}/applications &>/dev/null || :

%files
%{_bindir}/autonomix
%{_datadir}/applications/io.github.plebone.autonomix.desktop
%{_datadir}/icons/hicolor/scalable/apps/io.github.plebone.autonomix.svg
%{_datadir}/metainfo/io.github.plebone.autonomix.metainfo.xml

%changelog
* $(date "+%a %b %d %Y") PlebOne <plebone@example.com> - ${VERSION}-${RELEASE}
- v0.3.0: Switch to Qt/QML UI
- Added self-install detection for proper update method
- Added uninstall functionality
EOF

# Build RPM
rpmbuild -bb "$RPM_BUILD_DIR/SPECS/autonomix.spec"

# Copy to pkg-build directory
mkdir -p pkg-build
cp "$RPM_BUILD_DIR/RPMS/$ARCH/autonomix-${VERSION}-${RELEASE}"*.rpm pkg-build/

echo ""
echo "RPM package created in pkg-build/"
ls -la pkg-build/*.rpm
