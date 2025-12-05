use anyhow::{Context, Result};
use chrono::Utc;
use rusqlite::{params, Connection};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use super::installer::detect_self_install_type;
use super::models::{InstallType, TrackedApp};

/// Database manager for Autonomix
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    /// Get the default database path
    pub fn default_path() -> Result<PathBuf> {
        let config_dir = dirs::config_dir()
            .context("Could not find config directory")?
            .join("autonomix");

        std::fs::create_dir_all(&config_dir)?;
        Ok(config_dir.join("autonomix.db"))
    }

    /// Open or create the database
    pub fn open(path: Option<PathBuf>) -> Result<Self> {
        let db_path = match path {
            Some(p) => p,
            None => Self::default_path()?,
        };

        let conn = Connection::open(&db_path)?;
        let db = Self {
            conn: Arc::new(Mutex::new(conn)),
        };
        db.init()?;
        Ok(db)
    }

    /// Initialize database schema
    fn init(&self) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS apps (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                repo_owner TEXT NOT NULL,
                repo_name TEXT NOT NULL,
                display_name TEXT NOT NULL,
                installed_version TEXT,
                latest_version TEXT,
                install_type TEXT,
                last_checked TEXT,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                UNIQUE(repo_owner, repo_name)
            );

            CREATE INDEX IF NOT EXISTS idx_apps_repo ON apps(repo_owner, repo_name);
            "#,
        )?;
        Ok(())
    }

    /// Add a new app to track
    pub fn add_app(&self, repo_owner: &str, repo_name: &str, display_name: &str) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO apps (repo_owner, repo_name, display_name, created_at)
             VALUES (?1, ?2, ?3, ?4)",
            params![repo_owner, repo_name, display_name, Utc::now().to_rfc3339()],
        )?;
        Ok(conn.last_insert_rowid())
    }

    /// Get all tracked apps
    pub fn get_all_apps(&self) -> Result<Vec<TrackedApp>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repo_owner, repo_name, display_name, installed_version,
                    latest_version, install_type, last_checked, created_at
             FROM apps ORDER BY display_name",
        )?;

        let apps = stmt
            .query_map([], |row| {
                let install_type_str: Option<String> = row.get(6)?;
                let last_checked_str: Option<String> = row.get(7)?;
                let created_at_str: String = row.get(8)?;

                Ok(TrackedApp {
                    id: row.get(0)?,
                    repo_owner: row.get(1)?,
                    repo_name: row.get(2)?,
                    display_name: row.get(3)?,
                    installed_version: row.get(4)?,
                    latest_version: row.get(5)?,
                    install_type: install_type_str.and_then(|s| InstallType::from_str(&s)),
                    last_checked: last_checked_str
                        .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                        .map(|dt| dt.with_timezone(&Utc)),
                    created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now()),
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(apps)
    }

    /// Get a single app by ID
    pub fn get_app(&self, id: i64) -> Result<Option<TrackedApp>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repo_owner, repo_name, display_name, installed_version,
                    latest_version, install_type, last_checked, created_at
             FROM apps WHERE id = ?1",
        )?;

        let mut rows = stmt.query(params![id])?;
        if let Some(row) = rows.next()? {
            let install_type_str: Option<String> = row.get(6)?;
            let last_checked_str: Option<String> = row.get(7)?;
            let created_at_str: String = row.get(8)?;

            Ok(Some(TrackedApp {
                id: row.get(0)?,
                repo_owner: row.get(1)?,
                repo_name: row.get(2)?,
                display_name: row.get(3)?,
                installed_version: row.get(4)?,
                latest_version: row.get(5)?,
                install_type: install_type_str.and_then(|s| InstallType::from_str(&s)),
                last_checked: last_checked_str
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update latest version for an app
    pub fn update_latest_version(&self, id: i64, version: &str) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE apps SET latest_version = ?1, last_checked = ?2 WHERE id = ?3",
            params![version, Utc::now().to_rfc3339(), id],
        )?;
        Ok(())
    }

    /// Update installed version and install type
    pub fn update_installed(
        &self,
        id: i64,
        version: &str,
        install_type: InstallType,
    ) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE apps SET installed_version = ?1, install_type = ?2 WHERE id = ?3",
            params![version, install_type.as_str(), id],
        )?;
        Ok(())
    }

    /// Clear installed status (for uninstall)
    pub fn clear_installed(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE apps SET installed_version = NULL, install_type = NULL WHERE id = ?1",
            params![id],
        )?;
        Ok(())
    }

    /// Delete an app from tracking
    pub fn delete_app(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM apps WHERE id = ?1", params![id])?;
        Ok(())
    }

    /// Check if an app is already being tracked
    pub fn is_tracked(&self, repo_owner: &str, repo_name: &str) -> Result<bool> {
        let conn = self.conn.lock().unwrap();
        let count: i64 = conn.query_row(
            "SELECT COUNT(*) FROM apps WHERE repo_owner = ?1 AND repo_name = ?2",
            params![repo_owner, repo_name],
            |row| row.get(0),
        )?;
        Ok(count > 0)
    }

    /// Register Autonomix itself for self-updates
    pub fn register_self(&self) -> Result<()> {
        let repo_owner = "PlebOne";
        let repo_name = "autonomix";
        let current_version = env!("CARGO_PKG_VERSION");

        // Detect how Autonomix was installed
        let install_type = detect_self_install_type();

        if !self.is_tracked(repo_owner, repo_name)? {
            // Add new entry for Autonomix
            let id = self.add_app(repo_owner, repo_name, "Autonomix")?;
            
            // Set the installed version and install type if detected
            if let Some(itype) = install_type {
                self.update_installed(id, current_version, itype)?;
                log::info!("Registered Autonomix for self-updates with install type: {:?}", itype);
            } else {
                // Just set the version without install type
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    "UPDATE apps SET installed_version = ?1 WHERE id = ?2",
                    params![current_version, id],
                )?;
                log::info!("Registered Autonomix for self-updates (install type unknown)");
            }
        } else {
            // Autonomix already tracked - update install type if we can detect it and it's not set
            if let Some(app) = self.get_app_by_repo(repo_owner, repo_name)? {
                // Always update the current version
                let conn = self.conn.lock().unwrap();
                conn.execute(
                    "UPDATE apps SET installed_version = ?1 WHERE id = ?2",
                    params![current_version, app.id],
                )?;
                drop(conn);

                // Update install type if not already set or if we detect a different one
                if let Some(detected_type) = install_type {
                    if app.install_type.is_none() || app.install_type != Some(detected_type) {
                        self.update_install_type(app.id, detected_type)?;
                        log::info!("Updated Autonomix install type to: {:?}", detected_type);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get an app by repository owner and name
    pub fn get_app_by_repo(&self, repo_owner: &str, repo_name: &str) -> Result<Option<TrackedApp>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, repo_owner, repo_name, display_name, installed_version,
                    latest_version, install_type, last_checked, created_at
             FROM apps WHERE repo_owner = ?1 AND repo_name = ?2",
        )?;

        let mut rows = stmt.query(params![repo_owner, repo_name])?;
        if let Some(row) = rows.next()? {
            let install_type_str: Option<String> = row.get(6)?;
            let last_checked_str: Option<String> = row.get(7)?;
            let created_at_str: String = row.get(8)?;

            Ok(Some(TrackedApp {
                id: row.get(0)?,
                repo_owner: row.get(1)?,
                repo_name: row.get(2)?,
                display_name: row.get(3)?,
                installed_version: row.get(4)?,
                latest_version: row.get(5)?,
                install_type: install_type_str.and_then(|s| InstallType::from_str(&s)),
                last_checked: last_checked_str
                    .and_then(|s| chrono::DateTime::parse_from_rfc3339(&s).ok())
                    .map(|dt| dt.with_timezone(&Utc)),
                created_at: chrono::DateTime::parse_from_rfc3339(&created_at_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now()),
            }))
        } else {
            Ok(None)
        }
    }

    /// Update only the install type for an app
    pub fn update_install_type(&self, id: i64, install_type: InstallType) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE apps SET install_type = ?1 WHERE id = ?2",
            params![install_type.as_str(), id],
        )?;
        Ok(())
    }
}

impl Clone for Database {
    fn clone(&self) -> Self {
        Self {
            conn: Arc::clone(&self.conn),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_database_operations() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("test.db");
        let db = Database::open(Some(db_path)).unwrap();

        // Add app
        let id = db.add_app("octocat", "hello-world", "Hello World").unwrap();
        assert!(id > 0);

        // Get app
        let app = db.get_app(id).unwrap().unwrap();
        assert_eq!(app.repo_owner, "octocat");
        assert_eq!(app.repo_name, "hello-world");

        // Update versions
        db.update_latest_version(id, "v1.0.0").unwrap();
        db.update_installed(id, "v1.0.0", InstallType::Deb).unwrap();

        let app = db.get_app(id).unwrap().unwrap();
        assert_eq!(app.latest_version, Some("v1.0.0".to_string()));
        assert_eq!(app.installed_version, Some("v1.0.0".to_string()));
        assert_eq!(app.install_type, Some(InstallType::Deb));

        // Delete
        db.delete_app(id).unwrap();
        assert!(db.get_app(id).unwrap().is_none());
    }
}
