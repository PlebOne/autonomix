use anyhow::{Context, Result};
use reqwest::Client;
use serde::Deserialize;

use super::models::{InstallType, Release, ReleaseAsset};

/// GitHub API client
pub struct GitHubApi {
    client: Client,
}

#[derive(Debug, Deserialize)]
struct GitHubRelease {
    tag_name: String,
    name: Option<String>,
    body: Option<String>,
    published_at: Option<String>,
    prerelease: bool,
    draft: bool,
    assets: Vec<GitHubAsset>,
}

#[derive(Debug, Deserialize)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    content_type: String,
    size: u64,
}

impl GitHubApi {
    /// Create a new GitHub API client
    pub fn new() -> Result<Self> {
        let client = Client::builder()
            .user_agent("Autonomix/0.2.0")
            .timeout(std::time::Duration::from_secs(30))
            .build()?;

        Ok(Self { client })
    }

    /// Fetch the latest release for a repository
    pub async fn get_latest_release(&self, owner: &str, repo: &str) -> Result<Release> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases/latest",
            owner, repo
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch release")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "GitHub API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let gh_release: GitHubRelease = response.json().await?;
        Ok(self.convert_release(gh_release))
    }

    /// Fetch all releases for a repository
    pub async fn get_releases(&self, owner: &str, repo: &str) -> Result<Vec<Release>> {
        let url = format!(
            "https://api.github.com/repos/{}/{}/releases?per_page=10",
            owner, repo
        );

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch releases")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "GitHub API error: {} - {}",
                response.status(),
                response.text().await.unwrap_or_default()
            );
        }

        let gh_releases: Vec<GitHubRelease> = response.json().await?;
        Ok(gh_releases
            .into_iter()
            .map(|r| self.convert_release(r))
            .collect())
    }

    /// Convert GitHub release to our Release model
    fn convert_release(&self, gh: GitHubRelease) -> Release {
        Release {
            tag_name: gh.tag_name,
            name: gh.name,
            body: gh.body,
            published_at: gh
                .published_at
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc)),
            prerelease: gh.prerelease,
            draft: gh.draft,
            assets: gh
                .assets
                .into_iter()
                .map(|a| ReleaseAsset {
                    name: a.name,
                    browser_download_url: a.browser_download_url,
                    content_type: a.content_type,
                    size: a.size,
                })
                .collect(),
        }
    }

    /// Get repository info (to validate it exists and get display name)
    pub async fn get_repo_info(&self, owner: &str, repo: &str) -> Result<RepoInfo> {
        let url = format!("https://api.github.com/repos/{}/{}", owner, repo);

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .context("Failed to fetch repository info")?;

        if !response.status().is_success() {
            anyhow::bail!(
                "Repository not found: {}/{}",
                owner, repo
            );
        }

        let info: RepoInfo = response.json().await?;
        Ok(info)
    }

    /// Download a release asset to a file
    pub async fn download_asset(&self, url: &str, dest: &std::path::Path) -> Result<()> {
        let response = self
            .client
            .get(url)
            .send()
            .await
            .context("Failed to download asset")?;

        if !response.status().is_success() {
            anyhow::bail!("Download failed: {}", response.status());
        }

        let bytes = response.bytes().await?;
        std::fs::write(dest, &bytes)?;
        Ok(())
    }

    /// Find the best asset for the current system
    pub fn find_best_asset<'a>(
        &self,
        assets: &'a [ReleaseAsset],
        preferred_type: Option<InstallType>,
    ) -> Option<&'a ReleaseAsset> {
        // Filter to Linux-compatible, architecture-matching assets
        let compatible: Vec<_> = assets
            .iter()
            .filter(|a| a.is_linux() && a.matches_architecture())
            .collect();

        if compatible.is_empty() {
            return None;
        }

        // If user has a preferred type, try to find that first
        if let Some(pref) = preferred_type {
            if let Some(asset) = compatible.iter().find(|a| a.detect_install_type() == Some(pref)) {
                return Some(asset);
            }
        }

        // Priority order: deb > rpm > appimage > flatpak > snap > binary > source
        let priority = [
            InstallType::Deb,
            InstallType::Rpm,
            InstallType::AppImage,
            InstallType::Flatpak,
            InstallType::Snap,
            InstallType::Binary,
            InstallType::Source,
        ];

        for install_type in priority {
            if let Some(asset) = compatible
                .iter()
                .find(|a| a.detect_install_type() == Some(install_type))
            {
                return Some(asset);
            }
        }

        // Return first compatible if nothing matches
        compatible.first().copied()
    }
}

impl Default for GitHubApi {
    fn default() -> Self {
        Self::new().expect("Failed to create GitHub API client")
    }
}

#[derive(Debug, Deserialize)]
pub struct RepoInfo {
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub html_url: String,
}

/// Parse a GitHub URL into owner and repo
pub fn parse_github_url(url: &str) -> Option<(String, String)> {
    let url = url.trim().trim_end_matches('/');

    // Handle full URLs
    if url.starts_with("https://github.com/") || url.starts_with("http://github.com/") {
        let path = url
            .trim_start_matches("https://github.com/")
            .trim_start_matches("http://github.com/");
        return parse_repo_path(path);
    }

    // Handle github.com/owner/repo format
    if url.starts_with("github.com/") {
        let path = url.trim_start_matches("github.com/");
        return parse_repo_path(path);
    }

    // Handle owner/repo format
    if url.contains('/') && !url.contains(':') && !url.contains("://") {
        return parse_repo_path(url);
    }

    None
}

fn parse_repo_path(path: &str) -> Option<(String, String)> {
    let parts: Vec<_> = path.split('/').collect();
    if parts.len() >= 2 && !parts[0].is_empty() && !parts[1].is_empty() {
        let owner = parts[0].to_string();
        let repo = parts[1].trim_end_matches(".git").to_string();
        Some((owner, repo))
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_github_url() {
        assert_eq!(
            parse_github_url("https://github.com/octocat/hello-world"),
            Some(("octocat".to_string(), "hello-world".to_string()))
        );
        assert_eq!(
            parse_github_url("github.com/octocat/hello-world"),
            Some(("octocat".to_string(), "hello-world".to_string()))
        );
        assert_eq!(
            parse_github_url("octocat/hello-world"),
            Some(("octocat".to_string(), "hello-world".to_string()))
        );
        assert_eq!(
            parse_github_url("https://github.com/octocat/hello-world.git"),
            Some(("octocat".to_string(), "hello-world".to_string()))
        );
    }
}
