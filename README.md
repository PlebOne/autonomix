# Autonomix 🐧

**A Linux package manager for GitHub releases** - Similar to [Obtainium](https://github.com/ImranR98/Obtainium) for Android, but designed for Linux.

![Autonomix](https://img.shields.io/badge/Platform-Linux-blue?style=flat-square)
![Python](https://img.shields.io/badge/Python-3.10+-green?style=flat-square)
![License](https://img.shields.io/badge/License-MIT-yellow?style=flat-square)

## Features

- 📦 **Multiple Package Formats**: Install `.deb`, `.rpm`, AppImage, or build from source
- 🔄 **Automatic Update Detection**: Check for updates when you open the app
- 🎯 **Smart Package Selection**: Automatically selects the best package for your system
- 🖥️ **Modern UI**: Clean, dark-themed PySide6 interface
- 📋 **Track Multiple Apps**: Monitor all your GitHub-sourced applications in one place

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/autonomix.git
cd autonomix

# Install dependencies
pip install -e .

# Run the application
autonomix
```

### Requirements

- Python 3.10+
- PySide6
- One of: dpkg (Debian/Ubuntu), rpm (Fedora/RHEL), or just run AppImages

## Usage

1. **Add an App**: Click "➕ Add App" and enter a GitHub repository URL
2. **Fetch Release**: The app will fetch the latest release and detect available packages
3. **Install**: Click "Install" to download and install the package
4. **Check Updates**: Click "🔄 Check Updates" or updates are checked automatically on startup

## Supported Package Types

| Type | Description | Distribution |
|------|-------------|--------------|
| `.deb` | Debian package | Debian, Ubuntu, Mint, Pop!_OS |
| `.rpm` | RPM package | Fedora, RHEL, CentOS, openSUSE |
| `.AppImage` | Portable app | Any Linux distro |
| Source | Build from source | Any Linux (requires build tools) |

## Screenshots

*Coming soon*

## Roadmap

- [ ] System tray integration for background update checks
- [ ] Scheduled automatic updates
- [ ] GitHub token support for private repos and higher rate limits
- [ ] Export/import app list
- [ ] Flatpak support
- [ ] Snap support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- Inspired by [Obtainium](https://github.com/ImranR98/Obtainium) for Android
- Built with [PySide6](https://doc.qt.io/qtforpython-6/)
