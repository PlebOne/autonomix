enum InstallType {
  deb,
  rpm,
  appImage,
  flatpak,
  snap,
  binary,
  source;

  String get name {
    switch (this) {
      case InstallType.deb:
        return 'deb';
      case InstallType.rpm:
        return 'rpm';
      case InstallType.appImage:
        return 'appimage';
      case InstallType.flatpak:
        return 'flatpak';
      case InstallType.snap:
        return 'snap';
      case InstallType.binary:
        return 'binary';
      case InstallType.source:
        return 'source';
    }
  }

  static InstallType? fromString(String? value) {
    if (value == null) return null;
    switch (value.toLowerCase()) {
      case 'deb':
        return InstallType.deb;
      case 'rpm':
        return InstallType.rpm;
      case 'appimage':
        return InstallType.appImage;
      case 'flatpak':
        return InstallType.flatpak;
      case 'snap':
        return InstallType.snap;
      case 'binary':
        return InstallType.binary;
      case 'source':
        return InstallType.source;
      default:
        return null;
    }
  }
  
  String get displayName {
    switch (this) {
      case InstallType.deb:
        return 'DEB';
      case InstallType.rpm:
        return 'RPM';
      case InstallType.appImage:
        return 'AppImage';
      case InstallType.flatpak:
        return 'Flatpak';
      case InstallType.snap:
        return 'Snap';
      case InstallType.binary:
        return 'Binary';
      case InstallType.source:
        return 'Source';
    }
  }
}
