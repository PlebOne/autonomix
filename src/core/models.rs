use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Represents an app tracked by Autonomix
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackedApp {
    pub id: i64,
    pub repo_owner: String,
    pub repo_name: String,
    pub display_name: String,
    pub installed_version: Option<String>,
    pub latest_version: Option<String>,
    pub install_type: Option<InstallType>,
    pub last_checked: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl TrackedApp {
    pub fn repo_url(&self) -> String {
        format!("https://github.com/{}/{}", self.repo_owner, self.repo_name)
    }

    pub fn has_update(&self) -> bool {
        match (&self.installed_version, &self.latest_version) {
            (Some(installed), Some(latest)) => {
                normalize_version(latest) != normalize_version(installed)
                    && is_newer_version(latest, installed)
            }
            _ => false,
        }
    }

    pub fn is_installed(&self) -> bool {
        self.installed_version.is_some()
    }
}

/// Normalize version string for comparison
fn normalize_version(version: &str) -> String {
    version
        .trim_start_matches('v')
        .trim_start_matches('V')
        .to_lowercase()
}

/// Check if new_version is newer than old_version
fn is_newer_version(new_version: &str, old_version: &str) -> bool {
    let new_norm = normalize_version(new_version);
    let old_norm = normalize_version(old_version);

    // Try semver comparison first
    if let (Ok(new_sem), Ok(old_sem)) = (
        semver::Version::parse(&new_norm),
        semver::Version::parse(&old_norm),
    ) {
        return new_sem > old_sem;
    }

    // Fall back to string comparison
    new_norm > old_norm
}

/// Type of package installation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InstallType {
    Deb,
    Rpm,
    AppImage,
    Flatpak,
    Snap,
    Binary,
    Source,
}

impl InstallType {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "deb" => Some(Self::Deb),
            "rpm" => Some(Self::Rpm),
            "appimage" => Some(Self::AppImage),
            "flatpak" => Some(Self::Flatpak),
            "snap" => Some(Self::Snap),
            "binary" => Some(Self::Binary),
            "source" => Some(Self::Source),
            _ => None,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Deb => "deb",
            Self::Rpm => "rpm",
            Self::AppImage => "appimage",
            Self::Flatpak => "flatpak",
            Self::Snap => "snap",
            Self::Binary => "binary",
            Self::Source => "source",
        }
    }

    pub fn display_name(&self) -> &'static str {
        match self {
            Self::Deb => "Debian Package",
            Self::Rpm => "RPM Package",
            Self::AppImage => "AppImage",
            Self::Flatpak => "Flatpak",
            Self::Snap => "Snap",
            Self::Binary => "Binary",
            Self::Source => "Source",
        }
    }
}

/// A GitHub release
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Release {
    pub tag_name: String,
    pub name: Option<String>,
    pub body: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub prerelease: bool,
    pub draft: bool,
    pub assets: Vec<ReleaseAsset>,
}

/// A release asset (downloadable file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReleaseAsset {
    pub name: String,
    pub browser_download_url: String,
    pub content_type: String,
    pub size: u64,
}

impl ReleaseAsset {
    /// Detect the install type from the asset name
    pub fn detect_install_type(&self) -> Option<InstallType> {
        let name_lower = self.name.to_lowercase();

        if name_lower.ends_with(".deb") {
            Some(InstallType::Deb)
        } else if name_lower.ends_with(".rpm") {
            Some(InstallType::Rpm)
        } else if name_lower.ends_with(".appimage") {
            Some(InstallType::AppImage)
        } else if name_lower.ends_with(".flatpak") || name_lower.ends_with(".flatpakref") {
            Some(InstallType::Flatpak)
        } else if name_lower.ends_with(".snap") {
            Some(InstallType::Snap)
        } else if name_lower.ends_with(".tar.gz")
            || name_lower.ends_with(".tar.xz")
            || name_lower.ends_with(".tar.bz2")
        {
            Some(InstallType::Source)
        } else if !name_lower.contains('.') || name_lower.ends_with(".bin") {
            Some(InstallType::Binary)
        } else {
            None
        }
    }

    /// Check if this asset matches the current system architecture
    pub fn matches_architecture(&self) -> bool {
        let name_lower = self.name.to_lowercase();
        let arch = std::env::consts::ARCH;

        // Map Rust arch to common naming conventions
        let arch_patterns: Vec<&str> = match arch {
            "x86_64" => vec!["x86_64", "amd64", "x64", "linux64"],
            "aarch64" => vec!["aarch64", "arm64"],
            "x86" => vec!["i386", "i686", "x86", "linux32"],
            _ => vec![arch],
        };

        // If no architecture in name, assume it's universal
        let has_arch_indicator = name_lower.contains("x86")
            || name_lower.contains("amd64")
            || name_lower.contains("arm")
            || name_lower.contains("aarch")
            || name_lower.contains("i386")
            || name_lower.contains("i686");

        if !has_arch_indicator {
            return true;
        }

        arch_patterns.iter().any(|p| name_lower.contains(p))
    }

    /// Check if this asset is for Linux
    pub fn is_linux(&self) -> bool {
        let name_lower = self.name.to_lowercase();

        // Exclude obvious non-Linux
        if name_lower.contains("windows")
            || name_lower.contains(".exe")
            || name_lower.contains(".msi")
            || name_lower.contains("macos")
            || name_lower.contains("darwin")
            || name_lower.contains(".dmg")
        {
            return false;
        }

        // Linux-specific formats always match
        if name_lower.ends_with(".deb")
            || name_lower.ends_with(".rpm")
            || name_lower.ends_with(".appimage")
            || name_lower.ends_with(".snap")
            || name_lower.ends_with(".flatpak")
        {
            return true;
        }

        // Explicit linux in name
        if name_lower.contains("linux") {
            return true;
        }

        // For generic archives, assume they could be Linux
        if name_lower.ends_with(".tar.gz")
            || name_lower.ends_with(".tar.xz")
            || name_lower.ends_with(".tar.bz2")
        {
            return true;
        }

        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_comparison() {
        assert!(is_newer_version("v1.2.0", "v1.1.0"));
        assert!(is_newer_version("1.2.0", "1.1.0"));
        assert!(!is_newer_version("v1.1.0", "v1.2.0"));
        assert!(!is_newer_version("v1.1.0", "v1.1.0"));
    }

    #[test]
    fn test_install_type_detection() {
        let asset = ReleaseAsset {
            name: "app_1.0.0_amd64.deb".to_string(),
            browser_download_url: String::new(),
            content_type: String::new(),
            size: 0,
        };
        assert_eq!(asset.detect_install_type(), Some(InstallType::Deb));
    }
}
