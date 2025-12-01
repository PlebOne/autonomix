"""GitHub API client for fetching release information."""

import re
from dataclasses import dataclass
from typing import Optional
import requests


@dataclass
class ReleaseAsset:
    """Represents a downloadable asset from a GitHub release."""
    name: str
    download_url: str
    size: int
    content_type: str
    
    @property
    def package_type(self) -> Optional[str]:
        """Detect the package type from the filename."""
        name_lower = self.name.lower()
        if name_lower.endswith('.deb'):
            return 'deb'
        elif name_lower.endswith('.rpm'):
            return 'rpm'
        elif name_lower.endswith('.appimage'):
            return 'appimage'
        elif name_lower.endswith('.tar.gz') or name_lower.endswith('.tar.xz'):
            return 'tarball'
        elif name_lower.endswith('.zip'):
            return 'zip'
        return None
    
    @property
    def architecture(self) -> Optional[str]:
        """Detect architecture from filename."""
        name_lower = self.name.lower()
        if any(x in name_lower for x in ['x86_64', 'amd64', 'x64']):
            return 'x86_64'
        elif any(x in name_lower for x in ['aarch64', 'arm64']):
            return 'arm64'
        elif any(x in name_lower for x in ['i386', 'i686', 'x86']):
            return 'x86'
        elif 'armhf' in name_lower or 'armv7' in name_lower:
            return 'armhf'
        return None


@dataclass
class Release:
    """Represents a GitHub release."""
    tag_name: str
    name: str
    published_at: str
    body: str
    assets: list[ReleaseAsset]
    prerelease: bool
    html_url: str
    
    @property
    def version(self) -> str:
        """Extract version number from tag name."""
        # Remove common prefixes like 'v', 'V', 'release-', etc.
        version = re.sub(r'^[vV]?(?:release[_-]?)?', '', self.tag_name)
        return version


class GitHubAPI:
    """Client for interacting with the GitHub API."""
    
    BASE_URL = "https://api.github.com"
    
    def __init__(self, token: Optional[str] = None):
        self.session = requests.Session()
        self.session.headers.update({
            'Accept': 'application/vnd.github+json',
            'X-GitHub-Api-Version': '2022-11-28',
        })
        if token:
            self.session.headers['Authorization'] = f'Bearer {token}'
    
    def parse_repo_url(self, url: str) -> tuple[str, str]:
        """Parse a GitHub URL to extract owner and repo name."""
        # Handle various GitHub URL formats
        patterns = [
            r'github\.com/([^/]+)/([^/]+?)(?:\.git)?(?:/.*)?$',
            r'^([^/]+)/([^/]+)$',  # owner/repo format
        ]
        for pattern in patterns:
            match = re.search(pattern, url)
            if match:
                return match.group(1), match.group(2)
        raise ValueError(f"Could not parse GitHub repository from: {url}")
    
    def get_latest_release(self, owner: str, repo: str) -> Optional[Release]:
        """Fetch the latest release for a repository."""
        url = f"{self.BASE_URL}/repos/{owner}/{repo}/releases/latest"
        try:
            response = self.session.get(url)
            response.raise_for_status()
            return self._parse_release(response.json())
        except requests.exceptions.HTTPError as e:
            if e.response.status_code == 404:
                return None
            raise
    
    def get_releases(self, owner: str, repo: str, include_prerelease: bool = False) -> list[Release]:
        """Fetch all releases for a repository."""
        url = f"{self.BASE_URL}/repos/{owner}/{repo}/releases"
        response = self.session.get(url)
        response.raise_for_status()
        
        releases = [self._parse_release(r) for r in response.json()]
        if not include_prerelease:
            releases = [r for r in releases if not r.prerelease]
        return releases
    
    def get_repo_info(self, owner: str, repo: str) -> dict:
        """Fetch repository information."""
        url = f"{self.BASE_URL}/repos/{owner}/{repo}"
        response = self.session.get(url)
        response.raise_for_status()
        return response.json()
    
    def _parse_release(self, data: dict) -> Release:
        """Parse a release from API response."""
        assets = [
            ReleaseAsset(
                name=a['name'],
                download_url=a['browser_download_url'],
                size=a['size'],
                content_type=a['content_type'],
            )
            for a in data.get('assets', [])
        ]
        return Release(
            tag_name=data['tag_name'],
            name=data.get('name', data['tag_name']),
            published_at=data['published_at'],
            body=data.get('body', ''),
            assets=assets,
            prerelease=data.get('prerelease', False),
            html_url=data['html_url'],
        )
    
    def download_asset(self, asset: ReleaseAsset, destination: str, progress_callback=None) -> str:
        """Download a release asset to a destination path."""
        import os
        
        response = self.session.get(asset.download_url, stream=True)
        response.raise_for_status()
        
        total_size = int(response.headers.get('content-length', 0))
        downloaded = 0
        
        filepath = os.path.join(destination, asset.name)
        with open(filepath, 'wb') as f:
            for chunk in response.iter_content(chunk_size=8192):
                f.write(chunk)
                downloaded += len(chunk)
                if progress_callback and total_size:
                    progress_callback(downloaded, total_size)
        
        return filepath
