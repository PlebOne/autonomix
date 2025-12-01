"""SQLite database for storing app information."""

import sqlite3
from dataclasses import dataclass
from datetime import datetime
from typing import Optional
from pathlib import Path

# Autonomix's own repository for self-updates
AUTONOMIX_REPO_URL = "https://github.com/PlebOne/autonomix"
AUTONOMIX_OWNER = "PlebOne"
AUTONOMIX_REPO = "autonomix"


@dataclass
class App:
    """Represents an installed/tracked application."""
    id: Optional[int]
    name: str
    repo_url: str
    owner: str
    repo: str
    installed_version: Optional[str]
    latest_version: Optional[str]
    package_type: str  # deb, rpm, appimage, source
    install_path: Optional[str]
    added_at: str
    updated_at: str
    auto_update: bool = False
    include_prerelease: bool = False
    
    @property
    def has_update(self) -> bool:
        """Check if an update is available."""
        if not self.installed_version or not self.latest_version:
            return False
        return self.installed_version != self.latest_version


class Database:
    """SQLite database manager for Autonomix."""
    
    def __init__(self, db_path: Optional[str] = None):
        if db_path is None:
            config_dir = Path.home() / '.config' / 'autonomix'
            config_dir.mkdir(parents=True, exist_ok=True)
            db_path = str(config_dir / 'autonomix.db')
        
        self.db_path = db_path
        self._init_db()
    
    def _init_db(self):
        """Initialize the database schema."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute('''
                CREATE TABLE IF NOT EXISTS apps (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    name TEXT NOT NULL,
                    repo_url TEXT NOT NULL UNIQUE,
                    owner TEXT NOT NULL,
                    repo TEXT NOT NULL,
                    installed_version TEXT,
                    latest_version TEXT,
                    package_type TEXT NOT NULL,
                    install_path TEXT,
                    added_at TEXT NOT NULL,
                    updated_at TEXT NOT NULL,
                    auto_update INTEGER DEFAULT 0,
                    include_prerelease INTEGER DEFAULT 0
                )
            ''')
            conn.execute('''
                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT
                )
            ''')
            conn.commit()
        
        # Register Autonomix itself for self-updates on first run
        self._register_self()
    
    def _register_self(self):
        """Register Autonomix itself for self-updates if not already registered."""
        existing = self.get_app_by_repo(AUTONOMIX_OWNER, AUTONOMIX_REPO)
        if existing:
            return  # Already registered
        
        # Get current installed version
        try:
            from importlib.metadata import version
            installed_version = version("autonomix")
        except Exception:
            # Running from source - try to get version from package
            try:
                from autonomix import __version__
                installed_version = __version__
            except Exception:
                installed_version = "dev"
        
        now = datetime.now().isoformat()
        autonomix_app = App(
            id=None,
            name="Autonomix",
            repo_url=AUTONOMIX_REPO_URL,
            owner=AUTONOMIX_OWNER,
            repo=AUTONOMIX_REPO,
            installed_version=installed_version,
            latest_version=None,
            package_type="pip",  # Use pip for self-updates
            install_path=None,
            added_at=now,
            updated_at=now,
            auto_update=False,
            include_prerelease=False,
        )
        self.add_app(autonomix_app)
    
    def add_app(self, app: App) -> int:
        """Add a new app to the database."""
        now = datetime.now().isoformat()
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute('''
                INSERT INTO apps (name, repo_url, owner, repo, installed_version,
                                  latest_version, package_type, install_path,
                                  added_at, updated_at, auto_update, include_prerelease)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            ''', (app.name, app.repo_url, app.owner, app.repo,
                  app.installed_version, app.latest_version, app.package_type,
                  app.install_path, now, now, int(app.auto_update),
                  int(app.include_prerelease)))
            conn.commit()
            return cursor.lastrowid
    
    def get_all_apps(self) -> list[App]:
        """Get all tracked apps."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute('SELECT * FROM apps ORDER BY name')
            return [self._row_to_app(row) for row in cursor.fetchall()]
    
    def get_app(self, app_id: int) -> Optional[App]:
        """Get an app by ID."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute('SELECT * FROM apps WHERE id = ?', (app_id,))
            row = cursor.fetchone()
            return self._row_to_app(row) if row else None
    
    def get_app_by_repo(self, owner: str, repo: str) -> Optional[App]:
        """Get an app by repository."""
        with sqlite3.connect(self.db_path) as conn:
            conn.row_factory = sqlite3.Row
            cursor = conn.execute(
                'SELECT * FROM apps WHERE owner = ? AND repo = ?',
                (owner, repo)
            )
            row = cursor.fetchone()
            return self._row_to_app(row) if row else None
    
    def update_app(self, app: App):
        """Update an existing app."""
        now = datetime.now().isoformat()
        with sqlite3.connect(self.db_path) as conn:
            conn.execute('''
                UPDATE apps SET
                    name = ?, installed_version = ?, latest_version = ?,
                    package_type = ?, install_path = ?, updated_at = ?,
                    auto_update = ?, include_prerelease = ?
                WHERE id = ?
            ''', (app.name, app.installed_version, app.latest_version,
                  app.package_type, app.install_path, now,
                  int(app.auto_update), int(app.include_prerelease), app.id))
            conn.commit()
    
    def delete_app(self, app_id: int):
        """Delete an app from the database."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute('DELETE FROM apps WHERE id = ?', (app_id,))
            conn.commit()
    
    def get_setting(self, key: str, default: str = None) -> Optional[str]:
        """Get a setting value."""
        with sqlite3.connect(self.db_path) as conn:
            cursor = conn.execute(
                'SELECT value FROM settings WHERE key = ?', (key,)
            )
            row = cursor.fetchone()
            return row[0] if row else default
    
    def set_setting(self, key: str, value: str):
        """Set a setting value."""
        with sqlite3.connect(self.db_path) as conn:
            conn.execute('''
                INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)
            ''', (key, value))
            conn.commit()
    
    def _row_to_app(self, row: sqlite3.Row) -> App:
        """Convert a database row to an App object."""
        return App(
            id=row['id'],
            name=row['name'],
            repo_url=row['repo_url'],
            owner=row['owner'],
            repo=row['repo'],
            installed_version=row['installed_version'],
            latest_version=row['latest_version'],
            package_type=row['package_type'],
            install_path=row['install_path'],
            added_at=row['added_at'],
            updated_at=row['updated_at'],
            auto_update=bool(row['auto_update']),
            include_prerelease=bool(row['include_prerelease']),
        )
