use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use super::models::InstallType;

/// Package installer that handles different package formats
pub struct Installer {
    downloads_dir: PathBuf,
    appimage_dir: PathBuf,
}

impl Installer {
    pub fn new() -> Result<Self> {
        let data_dir = dirs::data_dir()
            .context("Could not find data directory")?
            .join("autonomix");

        let downloads_dir = data_dir.join("downloads");
        let appimage_dir = data_dir.join("appimages");

        std::fs::create_dir_all(&downloads_dir)?;
        std::fs::create_dir_all(&appimage_dir)?;

        Ok(Self {
            downloads_dir,
            appimage_dir,
        })
    }

    /// Get the downloads directory
    pub fn downloads_dir(&self) -> &Path {
        &self.downloads_dir
    }

    /// Install a package from a downloaded file
    pub fn install(&self, path: &Path, install_type: InstallType) -> Result<()> {
        match install_type {
            InstallType::Deb => self.install_deb(path),
            InstallType::Rpm => self.install_rpm(path),
            InstallType::AppImage => self.install_appimage(path),
            InstallType::Flatpak => self.install_flatpak(path),
            InstallType::Snap => self.install_snap(path),
            InstallType::Binary => self.install_binary(path),
            InstallType::Source => Err(anyhow::anyhow!("Source installation not yet supported")),
        }
    }

    /// Install a .deb package using pkexec and dpkg
    fn install_deb(&self, path: &Path) -> Result<()> {
        log::info!("Installing deb package: {:?}", path);

        // Use pkexec with dpkg -i
        let status = Command::new("pkexec")
            .args(["dpkg", "-i"])
            .arg(path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run pkexec dpkg")?;

        if !status.success() {
            // Try to fix dependencies
            log::info!("Attempting to fix dependencies...");
            let fix_status = Command::new("pkexec")
                .args(["apt-get", "install", "-f", "-y"])
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status();

            if fix_status.map(|s| !s.success()).unwrap_or(true) {
                anyhow::bail!("Failed to install deb package");
            }
        }

        Ok(())
    }

    /// Install an .rpm package using pkexec and rpm/dnf
    fn install_rpm(&self, path: &Path) -> Result<()> {
        log::info!("Installing rpm package: {:?}", path);

        // Try dnf first (Fedora, RHEL 8+)
        let dnf_available = Command::new("which")
            .arg("dnf")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false);

        let status = if dnf_available {
            Command::new("pkexec")
                .args(["dnf", "install", "-y"])
                .arg(path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run pkexec dnf")?
        } else {
            // Fall back to rpm
            Command::new("pkexec")
                .args(["rpm", "-i", "--force"])
                .arg(path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run pkexec rpm")?
        };

        if !status.success() {
            anyhow::bail!("Failed to install rpm package");
        }

        Ok(())
    }

    /// Install an AppImage by copying to ~/.local/share/autonomix/appimages and making executable
    fn install_appimage(&self, path: &Path) -> Result<()> {
        log::info!("Installing AppImage: {:?}", path);

        let filename = path
            .file_name()
            .context("Invalid AppImage path")?
            .to_string_lossy();

        let dest = self.appimage_dir.join(filename.as_ref());

        // Copy the AppImage
        std::fs::copy(path, &dest).context("Failed to copy AppImage")?;

        // Make it executable
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }

        // Create a .desktop file for it
        self.create_appimage_desktop_entry(&dest, &filename)?;

        log::info!("AppImage installed to: {:?}", dest);
        Ok(())
    }

    /// Create a desktop entry for an AppImage
    fn create_appimage_desktop_entry(&self, appimage_path: &Path, name: &str) -> Result<()> {
        let applications_dir = dirs::data_dir()
            .context("Could not find data directory")?
            .join("applications");

        std::fs::create_dir_all(&applications_dir)?;

        // Clean up the name for the desktop file
        let clean_name = name
            .trim_end_matches(".AppImage")
            .trim_end_matches(".appimage")
            .replace('-', " ")
            .replace('_', " ");

        let desktop_entry = format!(
            r#"[Desktop Entry]
Type=Application
Name={}
Exec="{}"
Terminal=false
Categories=Utility;
"#,
            clean_name,
            appimage_path.display()
        );

        let desktop_file = applications_dir.join(format!(
            "{}.desktop",
            name.trim_end_matches(".AppImage")
                .trim_end_matches(".appimage")
        ));

        std::fs::write(&desktop_file, desktop_entry)?;
        log::info!("Created desktop entry: {:?}", desktop_file);

        Ok(())
    }

    /// Install a Flatpak package
    fn install_flatpak(&self, path: &Path) -> Result<()> {
        log::info!("Installing Flatpak: {:?}", path);

        let path_str = path.to_string_lossy();

        // Check if it's a .flatpakref (remote install) or .flatpak (bundle)
        if path_str.ends_with(".flatpakref") {
            let status = Command::new("flatpak")
                .args(["install", "--user", "-y"])
                .arg(path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run flatpak install")?;

            if !status.success() {
                anyhow::bail!("Failed to install flatpakref");
            }
        } else {
            // Bundle install
            let status = Command::new("flatpak")
                .args(["install", "--user", "-y", "--bundle"])
                .arg(path)
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit())
                .status()
                .context("Failed to run flatpak install")?;

            if !status.success() {
                anyhow::bail!("Failed to install flatpak bundle");
            }
        }

        Ok(())
    }

    /// Install a Snap package
    fn install_snap(&self, path: &Path) -> Result<()> {
        log::info!("Installing Snap: {:?}", path);

        let status = Command::new("pkexec")
            .args(["snap", "install", "--dangerous"])
            .arg(path)
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .status()
            .context("Failed to run snap install")?;

        if !status.success() {
            anyhow::bail!("Failed to install snap package");
        }

        Ok(())
    }

    /// Install a binary to ~/.local/bin
    fn install_binary(&self, path: &Path) -> Result<()> {
        log::info!("Installing binary: {:?}", path);

        let bin_dir = dirs::home_dir()
            .context("Could not find home directory")?
            .join(".local")
            .join("bin");

        std::fs::create_dir_all(&bin_dir)?;

        let filename = path
            .file_name()
            .context("Invalid binary path")?;

        let dest = bin_dir.join(filename);

        std::fs::copy(path, &dest).context("Failed to copy binary")?;

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let mut perms = std::fs::metadata(&dest)?.permissions();
            perms.set_mode(0o755);
            std::fs::set_permissions(&dest, perms)?;
        }

        log::info!("Binary installed to: {:?}", dest);
        Ok(())
    }

    /// Check which package managers are available on the system
    pub fn detect_available_installers() -> Vec<InstallType> {
        let mut available = Vec::new();

        // Check for dpkg (deb support)
        if Command::new("which")
            .arg("dpkg")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            available.push(InstallType::Deb);
        }

        // Check for rpm
        if Command::new("which")
            .arg("rpm")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            available.push(InstallType::Rpm);
        }

        // AppImage always works on Linux
        available.push(InstallType::AppImage);

        // Check for flatpak
        if Command::new("which")
            .arg("flatpak")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            available.push(InstallType::Flatpak);
        }

        // Check for snap
        if Command::new("which")
            .arg("snap")
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            available.push(InstallType::Snap);
        }

        // Binary always works
        available.push(InstallType::Binary);

        available
    }
}

impl Default for Installer {
    fn default() -> Self {
        Self::new().expect("Failed to create installer")
    }
}
