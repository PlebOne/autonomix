Name:           autonomix
Version:        0.1.0
Release:        1%{?dist}
Summary:        Linux package manager for GitHub releases

License:        MIT
URL:            https://github.com/plebone/autonomix
Source0:        %{name}-%{version}.tar.gz

BuildArch:      noarch
BuildRequires:  python3-devel
BuildRequires:  python3-pip
BuildRequires:  python3-setuptools

Requires:       python3 >= 3.10
Requires:       python3-pyside6
Requires:       python3-requests

%description
Autonomix is a Linux application similar to Obtainium for Android.
It allows you to track GitHub repositories and install their releases
as .deb, .rpm, AppImage packages, or build from source.

Features include automatic update detection, a modern dark-themed UI,
and support for multiple package formats.

%prep
%autosetup

%build
%py3_build

%install
%py3_install

# Install desktop file
install -Dm644 data/autonomix.desktop %{buildroot}%{_datadir}/applications/autonomix.desktop

# Install icon
install -Dm644 autonomix/resources/autonomix.svg %{buildroot}%{_datadir}/icons/hicolor/scalable/apps/autonomix.svg

%files
%license LICENSE
%doc README.md
%{python3_sitelib}/autonomix/
%{python3_sitelib}/autonomix-*.egg-info/
%{_bindir}/autonomix
%{_datadir}/applications/autonomix.desktop
%{_datadir}/icons/hicolor/scalable/apps/autonomix.svg

%changelog
* Sat Nov 30 2024 Plebone <contact@plebone.org> - 0.1.0-1
- Initial release
