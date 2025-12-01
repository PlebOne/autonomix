Name:           autonomix
Version:        0.1.1
Release:        1%{?dist}
Summary:        Linux package manager for GitHub releases

License:        MIT
URL:            https://github.com/PlebOne/autonomix
Source0:        autonomix
Source1:        autonomix.desktop
Source2:        autonomix.svg

BuildArch:      x86_64

%description
Autonomix is a Linux application similar to Obtainium for Android.
It allows you to track GitHub repositories and install their releases
as .deb, .rpm, AppImage, Flatpak, or Snap packages, or build from source.

Features include automatic update detection, a modern dark-themed UI,
and support for multiple package formats.

%install
mkdir -p %{buildroot}/usr/bin
mkdir -p %{buildroot}/usr/share/applications
mkdir -p %{buildroot}/usr/share/icons/hicolor/scalable/apps

install -m 755 %{SOURCE0} %{buildroot}/usr/bin/autonomix
install -m 644 %{SOURCE1} %{buildroot}/usr/share/applications/autonomix.desktop
install -m 644 %{SOURCE2} %{buildroot}/usr/share/icons/hicolor/scalable/apps/autonomix.svg

%files
/usr/bin/autonomix
/usr/share/applications/autonomix.desktop
/usr/share/icons/hicolor/scalable/apps/autonomix.svg

%changelog
* Sun Dec 01 2024 PlebOne <contact@plebone.org> - 0.1.1-1
- Add Flatpak and Snap support
- Fix installation detection for self-registration

* Sat Nov 30 2024 PlebOne <contact@plebone.org> - 0.1.0-1
- Initial release
