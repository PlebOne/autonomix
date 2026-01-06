# Autonomix

A Linux package manager for GitHub releases, built with Flutter.

Autonomix helps you track, install, update, and manage applications distributed via GitHub releases. It provides a clean, modern GUI for managing your GitHub-sourced applications with support for multiple package formats.

![License](https://img.shields.io/github/license/plebone/autonomix)
![Version](https://img.shields.io/github/v/release/plebone/autonomix)

## Features

### 🚀 Core Functionality
- **Track Applications** - Add any GitHub repository to track its releases
- **Automatic Update Checking** - Check for new versions across all tracked apps
- **Multi-Format Support** - Install `.deb`, `.rpm`, AppImage, Flatpak, and Snap packages
- **Version Management** - View installed versions and available updates at a glance
- **Launch Applications** - Start installed apps directly from Autonomix
- **Self-Management** - Autonomix tracks itself and can update to new versions

### 📦 Package Format Support
- **DEB** - Debian/Ubuntu packages (via `dpkg`)
- **RPM** - Fedora/RHEL/CentOS packages (via `rpm`)
- **AppImage** - Portable Linux applications
- **Flatpak** - Universal Linux packages
- **Snap** - Canonical's universal packages

### 🎨 Modern UI
- Material Design 3 interface
- Clean, intuitive navigation
- Real-time status updates
- Visual indicators for available updates
- Responsive design

### 💾 Data Management
- Lightweight JSON-based storage
- Tracks installation metadata
- Stores package information and launch commands
- Preserves update history

## Installation

### Debian/Ubuntu
```bash
wget https://github.com/plebone/autonomix/releases/latest/download/autonomix_0.3.5-1_amd64.deb
sudo dpkg -i autonomix_0.3.5-1_amd64.deb
```

### Fedora/RHEL/CentOS
```bash
wget https://github.com/plebone/autonomix/releases/latest/download/autonomix-0.3.5-1.x86_64.rpm
sudo rpm -i autonomix-0.3.5-1.x86_64.rpm
```

### From Source
```bash
git clone https://github.com/plebone/autonomix.git
cd autonomix
flutter pub get
flutter build linux --release
```

## Usage

### Adding an Application
1. Click the **+** button
2. Enter the GitHub repository details (owner/repo)
3. Provide a display name for the app
4. Click **Add**

### Checking for Updates
- Click the **Refresh** icon in the app bar to check all tracked apps for updates
- Apps with available updates will show an "Update Available" badge

### Installing/Updating
1. Tap on an app in the list
2. Choose the package format if multiple are available
3. Click **Install** or **Update**
4. Authenticate when prompted (packages requiring root access use `pkexec`)

### Launching Applications
1. Tap on an installed app
2. Click **Launch**

### Uninstalling
1. Tap on an installed app
2. Click **Uninstall**
3. Confirm the action

## Architecture

### Technology Stack
- **Framework**: Flutter 3.0+
- **Language**: Dart
- **State Management**: Provider
- **Storage**: JSON file-based database
- **HTTP Client**: http package for GitHub API
- **System Integration**: process_run for package management

### Project Structure
```
lib/
├── main.dart                 # Application entry point
├── models/                   # Data models
│   ├── tracked_app.dart     # App tracking model
│   ├── release.dart         # GitHub release model
│   └── install_type.dart    # Package format enum
├── services/                 # Business logic
│   ├── database_service.dart    # JSON storage
│   ├── github_service.dart      # GitHub API client
│   └── installer_service.dart   # Package installation
└── ui/                       # User interface
    ├── home_screen.dart     # Main screen
    └── widgets/             # Reusable UI components
        ├── app_list_item.dart
        └── add_app_dialog.dart
```

## Development

### Prerequisites
- Flutter SDK (>=3.0.0)
- Linux development environment
- Basic Linux package management tools (dpkg, rpm, etc.)

### Building
```bash
# Get dependencies
flutter pub get

# Run in debug mode
flutter run -d linux

# Build release
flutter build linux --release

# Run tests
flutter test
```

### Building Packages

#### DEB Package
```bash
flutter build linux --release
# Package structure is created in packaging/deb/
dpkg-deb --build packaging/deb/autonomix_0.3.5-1_amd64
```

#### RPM Package
```bash
flutter build linux --release
# Package spec is in packaging/rpm/SPECS/autonomix.spec
rpmbuild --define "_topdir $PWD/packaging/rpm" -bb packaging/rpm/SPECS/autonomix.spec
```

## Configuration

Configuration and data are stored in:
- **Linux**: `~/.local/share/autonomix/`
  - `apps.json` - Tracked applications database
  - `downloads/` - Temporary download storage
  - `appimages/` - Installed AppImage files

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add some amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## Roadmap

- [ ] Flatpak and Snap installation support
- [ ] Automatic update scheduling
- [ ] Application categories and tags
- [ ] Search and filter functionality
- [ ] Import/export tracked apps list
- [ ] Custom installation directories
- [ ] Release notes display
- [ ] Multiple GitHub accounts support

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Credits

Built with [Flutter](https://flutter.dev/) - Google's UI toolkit for beautiful, natively compiled applications.

## Support

If you encounter any issues or have questions:
- Open an issue on [GitHub Issues](https://github.com/plebone/autonomix/issues)
- Check existing issues for solutions
