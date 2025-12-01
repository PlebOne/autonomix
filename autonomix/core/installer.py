"""Package installer for various Linux package formats."""

import os
import shutil
import subprocess
import tempfile
from abc import ABC, abstractmethod
from pathlib import Path
from typing import Optional, Callable

from .github_api import ReleaseAsset


class InstallerError(Exception):
    """Raised when installation fails."""
    pass


class BaseInstaller(ABC):
    """Base class for package installers."""
    
    @abstractmethod
    def install(self, package_path: str, progress_callback: Optional[Callable] = None) -> str:
        """Install a package and return the install path."""
        pass
    
    @abstractmethod
    def uninstall(self, app_name: str, install_path: Optional[str] = None) -> bool:
        """Uninstall a package."""
        pass
    
    @abstractmethod
    def is_available(self) -> bool:
        """Check if this installer is available on the system."""
        pass
    
    @abstractmethod
    def is_installed(self, app_name: str) -> bool:
        """Check if a package is installed."""
        pass
    
    @abstractmethod
    def get_installed_version(self, app_name: str) -> Optional[str]:
        """Get the installed version of a package, or None if not installed."""
        pass


class DebInstaller(BaseInstaller):
    """Installer for .deb packages (Debian/Ubuntu)."""
    
    def is_available(self) -> bool:
        return shutil.which('dpkg') is not None
    
    def install(self, package_path: str, progress_callback: Optional[Callable] = None) -> str:
        if not os.path.exists(package_path):
            raise InstallerError(f"Package not found: {package_path}")
        
        try:
            # Install using dpkg, then fix dependencies with apt
            result = subprocess.run(
                ['pkexec', 'dpkg', '-i', package_path],
                capture_output=True,
                text=True
            )
            
            if result.returncode != 0:
                # Try to fix dependencies
                subprocess.run(
                    ['pkexec', 'apt-get', 'install', '-f', '-y'],
                    capture_output=True,
                    text=True
                )
            
            # Get installed path from dpkg
            pkg_name = self._get_package_name(package_path)
            return f"/usr (managed by dpkg: {pkg_name})"
            
        except subprocess.CalledProcessError as e:
            raise InstallerError(f"Failed to install .deb package: {e.stderr}")
    
    def uninstall(self, app_name: str, install_path: Optional[str] = None) -> bool:
        try:
            result = subprocess.run(
                ['pkexec', 'dpkg', '-r', app_name],
                capture_output=True,
                text=True
            )
            return result.returncode == 0
        except subprocess.CalledProcessError:
            return False
    
    def _get_package_name(self, package_path: str) -> str:
        """Extract package name from .deb file."""
        result = subprocess.run(
            ['dpkg-deb', '-f', package_path, 'Package'],
            capture_output=True,
            text=True
        )
        return result.stdout.strip()
    
    def is_installed(self, app_name: str) -> bool:
        """Check if a deb package is installed."""
        # Try multiple name variations
        names_to_try = self._get_package_name_variations(app_name)
        for name in names_to_try:
            try:
                result = subprocess.run(
                    ['dpkg', '-s', name],
                    capture_output=True,
                    text=True
                )
                if result.returncode == 0 and 'Status: install ok installed' in result.stdout:
                    return True
            except Exception:
                pass
        return False
    
    def get_installed_version(self, app_name: str) -> Optional[str]:
        """Get the installed version of a deb package."""
        names_to_try = self._get_package_name_variations(app_name)
        for name in names_to_try:
            try:
                result = subprocess.run(
                    ['dpkg-query', '-W', '-f=${Version}', name],
                    capture_output=True,
                    text=True
                )
                if result.returncode == 0 and result.stdout.strip():
                    return result.stdout.strip()
            except Exception:
                pass
        return None
    
    def _get_package_name_variations(self, app_name: str) -> list[str]:
        """Get possible package name variations to try."""
        name = app_name.lower()
        variations = [
            name,
            name.replace('-', ''),
            name.replace('_', '-'),
            name.replace('-', '_'),
        ]
        # Also try without common suffixes/prefixes
        if name.endswith('-bin'):
            variations.append(name[:-4])
        if name.endswith('-git'):
            variations.append(name[:-4])
        if name.startswith('lib'):
            variations.append(name[3:])
        
        # Search dpkg for any package containing this name
        try:
            result = subprocess.run(
                ['dpkg-query', '-W', '-f=${Package}\n'],
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                for pkg in result.stdout.strip().split('\n'):
                    pkg = pkg.strip()
                    if pkg and name in pkg.lower():
                        variations.append(pkg)
        except Exception:
            pass
        
        return list(dict.fromkeys(variations))  # Remove duplicates, preserve order


class RpmInstaller(BaseInstaller):
    """Installer for .rpm packages (Fedora/RHEL/openSUSE)."""
    
    def is_available(self) -> bool:
        return shutil.which('rpm') is not None
    
    def install(self, package_path: str, progress_callback: Optional[Callable] = None) -> str:
        if not os.path.exists(package_path):
            raise InstallerError(f"Package not found: {package_path}")
        
        try:
            # Try dnf first (Fedora), then zypper (openSUSE), then yum
            if shutil.which('dnf'):
                cmd = ['pkexec', 'dnf', 'install', '-y', package_path]
            elif shutil.which('zypper'):
                cmd = ['pkexec', 'zypper', 'install', '-y', package_path]
            else:
                cmd = ['pkexec', 'rpm', '-i', package_path]
            
            result = subprocess.run(cmd, capture_output=True, text=True)
            
            if result.returncode != 0:
                raise InstallerError(f"RPM installation failed: {result.stderr}")
            
            pkg_name = self._get_package_name(package_path)
            return f"/usr (managed by rpm: {pkg_name})"
            
        except subprocess.CalledProcessError as e:
            raise InstallerError(f"Failed to install .rpm package: {e.stderr}")
    
    def uninstall(self, app_name: str, install_path: Optional[str] = None) -> bool:
        try:
            if shutil.which('dnf'):
                cmd = ['pkexec', 'dnf', 'remove', '-y', app_name]
            elif shutil.which('zypper'):
                cmd = ['pkexec', 'zypper', 'remove', '-y', app_name]
            else:
                cmd = ['pkexec', 'rpm', '-e', app_name]
            
            result = subprocess.run(cmd, capture_output=True, text=True)
            return result.returncode == 0
        except subprocess.CalledProcessError:
            return False
    
    def _get_package_name(self, package_path: str) -> str:
        """Extract package name from .rpm file."""
        result = subprocess.run(
            ['rpm', '-qp', '--queryformat', '%{NAME}', package_path],
            capture_output=True,
            text=True
        )
        return result.stdout.strip()
    
    def is_installed(self, app_name: str) -> bool:
        """Check if an rpm package is installed."""
        names_to_try = self._get_package_name_variations(app_name)
        for name in names_to_try:
            try:
                result = subprocess.run(
                    ['rpm', '-q', name],
                    capture_output=True,
                    text=True
                )
                if result.returncode == 0:
                    return True
            except Exception:
                pass
        return False
    
    def get_installed_version(self, app_name: str) -> Optional[str]:
        """Get the installed version of an rpm package."""
        names_to_try = self._get_package_name_variations(app_name)
        for name in names_to_try:
            try:
                result = subprocess.run(
                    ['rpm', '-q', '--queryformat', '%{VERSION}', name],
                    capture_output=True,
                    text=True
                )
                if result.returncode == 0 and result.stdout.strip():
                    return result.stdout.strip()
            except Exception:
                pass
        return None
    
    def _get_package_name_variations(self, app_name: str) -> list[str]:
        """Get possible package name variations to try."""
        name = app_name.lower()
        variations = [
            name,
            name.replace('-', ''),
            name.replace('_', '-'),
            name.replace('-', '_'),
        ]
        # Search rpm for any package containing this name
        try:
            result = subprocess.run(
                ['rpm', '-qa', '--queryformat', '%{NAME}\n'],
                capture_output=True,
                text=True
            )
            if result.returncode == 0:
                for pkg in result.stdout.strip().split('\n'):
                    pkg = pkg.strip()
                    if pkg and name in pkg.lower():
                        variations.append(pkg)
        except Exception:
            pass
        
        return list(dict.fromkeys(variations))


class AppImageInstaller(BaseInstaller):
    """Installer for AppImage files."""
    
    INSTALL_DIR = Path.home() / '.local' / 'bin'
    DESKTOP_DIR = Path.home() / '.local' / 'share' / 'applications'
    
    def __init__(self):
        self.INSTALL_DIR.mkdir(parents=True, exist_ok=True)
        self.DESKTOP_DIR.mkdir(parents=True, exist_ok=True)
    
    def is_available(self) -> bool:
        return True  # AppImages are always available on Linux
    
    def install(self, package_path: str, progress_callback: Optional[Callable] = None) -> str:
        if not os.path.exists(package_path):
            raise InstallerError(f"Package not found: {package_path}")
        
        # Copy to install directory
        filename = os.path.basename(package_path)
        install_path = self.INSTALL_DIR / filename
        
        shutil.copy2(package_path, install_path)
        os.chmod(install_path, 0o755)  # Make executable
        
        # Try to extract and create desktop entry
        self._create_desktop_entry(install_path, filename)
        
        return str(install_path)
    
    def uninstall(self, app_name: str, install_path: Optional[str] = None) -> bool:
        try:
            if install_path and os.path.exists(install_path):
                os.remove(install_path)
            
            # Remove desktop entry
            desktop_file = self.DESKTOP_DIR / f"{app_name}.desktop"
            if desktop_file.exists():
                desktop_file.unlink()
            
            return True
        except Exception:
            return False
    
    def _create_desktop_entry(self, appimage_path: Path, filename: str):
        """Create a .desktop file for the AppImage."""
        app_name = filename.replace('.AppImage', '').replace('.appimage', '')
        app_name = app_name.split('-')[0]  # Get base name
        
        desktop_content = f"""[Desktop Entry]
Type=Application
Name={app_name}
Exec={appimage_path}
Icon=application-x-executable
Terminal=false
Categories=Utility;
Comment=Installed via Autonomix
"""
        desktop_file = self.DESKTOP_DIR / f"{app_name.lower()}.desktop"
        desktop_file.write_text(desktop_content)
    
    def is_installed(self, app_name: str) -> bool:
        """Check if an AppImage is installed."""
        # Check for any AppImage matching the app name
        for f in self.INSTALL_DIR.iterdir():
            if f.is_file() and app_name.lower() in f.name.lower() and 'appimage' in f.name.lower():
                return True
        return False
    
    def get_installed_version(self, app_name: str) -> Optional[str]:
        """Get the installed version of an AppImage (extracted from filename)."""
        import re
        for f in self.INSTALL_DIR.iterdir():
            if f.is_file() and app_name.lower() in f.name.lower() and 'appimage' in f.name.lower():
                # Try to extract version from filename like "App-1.2.3.AppImage"
                match = re.search(r'[-_]v?(\d+\.\d+(?:\.\d+)?)', f.name)
                if match:
                    return match.group(1)
                return "installed"  # Version unknown but installed
        return None
    
    def get_install_path(self, app_name: str) -> Optional[str]:
        """Get the install path of an AppImage."""
        for f in self.INSTALL_DIR.iterdir():
            if f.is_file() and app_name.lower() in f.name.lower() and 'appimage' in f.name.lower():
                return str(f)
        return None


class SourceInstaller(BaseInstaller):
    """Installer that builds from source."""
    
    INSTALL_PREFIX = Path.home() / '.local'
    
    def is_available(self) -> bool:
        # Check for common build tools
        return (shutil.which('make') is not None or 
                shutil.which('cmake') is not None or
                shutil.which('cargo') is not None or
                shutil.which('go') is not None)
    
    def install(self, package_path: str, progress_callback: Optional[Callable] = None) -> str:
        """Install from source tarball/directory."""
        if not os.path.exists(package_path):
            raise InstallerError(f"Source not found: {package_path}")
        
        # Create temp directory for extraction
        with tempfile.TemporaryDirectory() as tmpdir:
            src_dir = Path(tmpdir) / 'source'
            
            # Extract if it's an archive
            if package_path.endswith('.tar.gz') or package_path.endswith('.tgz'):
                subprocess.run(['tar', 'xzf', package_path, '-C', tmpdir], check=True)
            elif package_path.endswith('.tar.xz'):
                subprocess.run(['tar', 'xJf', package_path, '-C', tmpdir], check=True)
            elif package_path.endswith('.zip'):
                subprocess.run(['unzip', package_path, '-d', tmpdir], check=True)
            else:
                src_dir = Path(package_path)
            
            # Find the actual source directory
            if not src_dir.exists():
                dirs = [d for d in Path(tmpdir).iterdir() if d.is_dir()]
                if dirs:
                    src_dir = dirs[0]
            
            # Detect build system and build
            install_path = self._build_project(src_dir)
            return install_path
    
    def _build_project(self, src_dir: Path) -> str:
        """Detect build system and build the project."""
        prefix = str(self.INSTALL_PREFIX)
        
        # Rust/Cargo
        if (src_dir / 'Cargo.toml').exists():
            subprocess.run(['cargo', 'install', '--path', str(src_dir), '--root', prefix], check=True)
            return f"{prefix}/bin"
        
        # Go
        if (src_dir / 'go.mod').exists():
            env = os.environ.copy()
            env['GOBIN'] = f"{prefix}/bin"
            subprocess.run(['go', 'install', './...'], cwd=src_dir, env=env, check=True)
            return f"{prefix}/bin"
        
        # CMake
        if (src_dir / 'CMakeLists.txt').exists():
            build_dir = src_dir / 'build'
            build_dir.mkdir(exist_ok=True)
            subprocess.run(['cmake', '..', f'-DCMAKE_INSTALL_PREFIX={prefix}'], cwd=build_dir, check=True)
            subprocess.run(['make', '-j'], cwd=build_dir, check=True)
            subprocess.run(['make', 'install'], cwd=build_dir, check=True)
            return prefix
        
        # Meson
        if (src_dir / 'meson.build').exists():
            build_dir = src_dir / 'build'
            subprocess.run(['meson', 'setup', str(build_dir), f'--prefix={prefix}'], cwd=src_dir, check=True)
            subprocess.run(['ninja', '-C', str(build_dir)], check=True)
            subprocess.run(['ninja', '-C', str(build_dir), 'install'], check=True)
            return prefix
        
        # Autotools/Make
        if (src_dir / 'configure').exists():
            subprocess.run(['./configure', f'--prefix={prefix}'], cwd=src_dir, check=True)
            subprocess.run(['make', '-j'], cwd=src_dir, check=True)
            subprocess.run(['make', 'install'], cwd=src_dir, check=True)
            return prefix
        
        if (src_dir / 'Makefile').exists():
            subprocess.run(['make', '-j', f'PREFIX={prefix}'], cwd=src_dir, check=True)
            subprocess.run(['make', 'install', f'PREFIX={prefix}'], cwd=src_dir, check=True)
            return prefix
        
        raise InstallerError("Could not detect build system")
    
    def uninstall(self, app_name: str, install_path: Optional[str] = None) -> bool:
        # Source installations are hard to track, return False
        return False
    
    def is_installed(self, app_name: str) -> bool:
        """Check if source-built app is installed (check for binary in ~/.local/bin)."""
        bin_path = self.INSTALL_PREFIX / 'bin' / app_name
        return bin_path.exists()
    
    def get_installed_version(self, app_name: str) -> Optional[str]:
        """Try to get version of source-built app."""
        bin_path = self.INSTALL_PREFIX / 'bin' / app_name
        if bin_path.exists():
            # Try running with --version
            try:
                result = subprocess.run(
                    [str(bin_path), '--version'],
                    capture_output=True,
                    text=True,
                    timeout=5
                )
                if result.returncode == 0:
                    # Extract first version-like string
                    import re
                    match = re.search(r'(\d+\.\d+(?:\.\d+)?)', result.stdout)
                    if match:
                        return match.group(1)
            except Exception:
                pass
            return "installed"
        return None


class PackageInstaller:
    """Main package installer that selects the appropriate installer."""
    
    def __init__(self):
        self.installers = {
            'deb': DebInstaller(),
            'rpm': RpmInstaller(),
            'appimage': AppImageInstaller(),
            'tarball': SourceInstaller(),
            'source': SourceInstaller(),
        }
    
    def get_available_types(self) -> list[str]:
        """Get list of available package types on this system."""
        available = []
        for pkg_type, installer in self.installers.items():
            if installer.is_available():
                available.append(pkg_type)
        return available
    
    def get_system_package_type(self) -> str:
        """Detect the preferred package type for this system."""
        if shutil.which('dpkg'):
            return 'deb'
        elif shutil.which('rpm'):
            return 'rpm'
        return 'appimage'
    
    def install(self, package_path: str, package_type: str, 
                progress_callback: Optional[Callable] = None) -> str:
        """Install a package using the appropriate installer."""
        installer = self.installers.get(package_type)
        if not installer:
            raise InstallerError(f"Unknown package type: {package_type}")
        
        if not installer.is_available():
            raise InstallerError(f"Installer for {package_type} is not available")
        
        return installer.install(package_path, progress_callback)
    
    def uninstall(self, package_type: str, app_name: str, 
                  install_path: Optional[str] = None) -> bool:
        """Uninstall a package."""
        installer = self.installers.get(package_type)
        if not installer:
            return False
        
        return installer.uninstall(app_name, install_path)
    
    def is_installed(self, app_name: str, package_type: Optional[str] = None) -> bool:
        """Check if an app is installed."""
        if package_type:
            installer = self.installers.get(package_type)
            if installer and installer.is_available():
                return installer.is_installed(app_name)
            return False
        
        # Check all installers
        for installer in self.installers.values():
            if installer.is_available() and installer.is_installed(app_name):
                return True
        return False
    
    def get_installed_version(self, app_name: str, package_type: Optional[str] = None) -> Optional[str]:
        """Get the installed version of an app."""
        if package_type:
            installer = self.installers.get(package_type)
            if installer and installer.is_available():
                return installer.get_installed_version(app_name)
            return None
        
        # Check all installers
        for installer in self.installers.values():
            if installer.is_available():
                version = installer.get_installed_version(app_name)
                if version:
                    return version
        return None
    
    def check_app_status(self, app_name: str, package_type: str) -> tuple[bool, Optional[str]]:
        """Check if an app is installed and get its version.
        
        Returns:
            tuple: (is_installed, version_or_none)
        """
        installer = self.installers.get(package_type)
        if not installer or not installer.is_available():
            return False, None
        
        is_installed = installer.is_installed(app_name)
        version = installer.get_installed_version(app_name) if is_installed else None
        return is_installed, version
    
    def select_best_asset(self, assets: list[ReleaseAsset], 
                          preferred_type: Optional[str] = None) -> Optional[ReleaseAsset]:
        """Select the best asset for this system from a list of assets."""
        import platform
        
        system_arch = platform.machine()
        arch_map = {
            'x86_64': 'x86_64',
            'amd64': 'x86_64',
            'aarch64': 'arm64',
            'armv7l': 'armhf',
        }
        system_arch = arch_map.get(system_arch, system_arch)
        
        if preferred_type is None:
            preferred_type = self.get_system_package_type()
        
        # Filter assets by package type and architecture
        candidates = []
        for asset in assets:
            if asset.package_type == preferred_type:
                asset_arch = asset.architecture
                if asset_arch is None or asset_arch == system_arch:
                    candidates.append(asset)
        
        # Prefer exact architecture match
        for asset in candidates:
            if asset.architecture == system_arch:
                return asset
        
        # Return any matching type
        if candidates:
            return candidates[0]
        
        # Fallback: try AppImage
        if preferred_type != 'appimage':
            for asset in assets:
                if asset.package_type == 'appimage':
                    asset_arch = asset.architecture
                    if asset_arch is None or asset_arch == system_arch:
                        return asset
        
        return None
