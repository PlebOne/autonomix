use qmetaobject::prelude::*;
use std::sync::Arc;
use once_cell::sync::Lazy;
use std::sync::Mutex;

use crate::core::database::Database;
use crate::core::github_api::{parse_github_url, GitHubApi};
use crate::core::installer::Installer;
use crate::core::models::TrackedApp;

// Global core components (initialized once)
static CORE: Lazy<Mutex<CoreComponents>> = Lazy::new(|| {
    let db = Database::open(None).expect("Failed to open database");
    let _ = db.register_self();
    let github = Arc::new(GitHubApi::new().expect("Failed to create GitHub API"));
    let installer = Arc::new(Installer::new().expect("Failed to create installer"));
    Mutex::new(CoreComponents { db, github, installer })
});

struct CoreComponents {
    db: Database,
    github: Arc<GitHubApi>,
    installer: Arc<Installer>,
}

/// Data for a single app in the model
#[derive(Default, Clone, Debug)]
pub struct AppData {
    pub id: i64,
    pub display_name: String,
    pub repo_owner: String,
    pub repo_name: String,
    pub installed_version: String,
    pub latest_version: String,
    pub install_type: String,
    pub has_update: bool,
    pub is_installed: bool,
}

impl AppData {
    fn from_tracked_app(app: &TrackedApp) -> Self {
        Self {
            id: app.id,
            display_name: app.display_name.clone(),
            repo_owner: app.repo_owner.clone(),
            repo_name: app.repo_name.clone(),
            installed_version: app.installed_version.clone().unwrap_or_default(),
            latest_version: app.latest_version.clone().unwrap_or_default(),
            install_type: app
                .install_type
                .map(|t| t.display_name().to_string())
                .unwrap_or_default(),
            has_update: app.has_update(),
            is_installed: app.is_installed(),
        }
    }
}

/// Qt model for the application list
#[derive(QObject, Default)]
pub struct AppModel {
    base: qt_base_class!(trait QAbstractListModel),

    // Properties exposed to QML
    count: qt_property!(i32; NOTIFY count_changed),
    loading: qt_property!(bool; NOTIFY loading_changed),
    error_message: qt_property!(QString; NOTIFY error_changed),

    // Signals
    count_changed: qt_signal!(),
    loading_changed: qt_signal!(),
    error_changed: qt_signal!(),
    apps_refreshed: qt_signal!(),

    // Methods exposed to QML
    add_app: qt_method!(fn(&mut self, url: QString) -> bool),
    remove_app: qt_method!(fn(&mut self, index: i32)),
    install_app: qt_method!(fn(&mut self, index: i32)),
    install_app_with_type: qt_method!(fn(&mut self, index: i32, install_type: QString)),
    get_available_packages: qt_method!(fn(&mut self, index: i32) -> QString),
    uninstall_app: qt_method!(fn(&mut self, index: i32)),
    refresh: qt_method!(fn(&mut self)),
    update_all: qt_method!(fn(&mut self)),
    get_app_id: qt_method!(fn(&self, index: i32) -> i64),
    get_display_name: qt_method!(fn(&self, index: i32) -> QString),
    get_repo_info: qt_method!(fn(&self, index: i32) -> QString),
    get_installed_version: qt_method!(fn(&self, index: i32) -> QString),
    get_latest_version: qt_method!(fn(&self, index: i32) -> QString),
    get_install_type: qt_method!(fn(&self, index: i32) -> QString),
    has_update: qt_method!(fn(&self, index: i32) -> bool),
    is_installed: qt_method!(fn(&self, index: i32) -> bool),

    // Internal data (use simple types that implement Default)
    apps: Vec<AppData>,
}

impl AppModel {
    fn load_apps(&mut self) {
        let core = CORE.lock().unwrap();
        if let Ok(tracked_apps) = core.db.get_all_apps() {
            let app_data: Vec<AppData> = tracked_apps
                .iter()
                .map(AppData::from_tracked_app)
                .collect();

            let count = app_data.len() as i32;

            (self as &mut dyn QAbstractListModel).begin_reset_model();
            self.apps = app_data;
            (self as &mut dyn QAbstractListModel).end_reset_model();

            self.count = count;
            self.count_changed();
        }
    }

    fn add_app(&mut self, url: QString) -> bool {
        let url_str = url.to_string();
        if let Some((owner, repo)) = parse_github_url(&url_str) {
            let core = CORE.lock().unwrap();
            
            // Check if already tracked
            if core.db.is_tracked(&owner, &repo).unwrap_or(true) {
                self.error_message = QString::from("Repository is already being tracked");
                self.error_changed();
                return false;
            }

            match core.db.add_app(&owner, &repo, &repo) {
                Ok(_) => {
                    drop(core);
                    self.load_apps();
                    self.refresh();
                    return true;
                }
                Err(e) => {
                    self.error_message = QString::from(format!("Failed to add app: {}", e));
                    self.error_changed();
                }
            }
        } else {
            self.error_message = QString::from("Invalid GitHub URL format");
            self.error_changed();
        }
        false
    }

    fn remove_app(&mut self, index: i32) {
        let app_id = self.apps.get(index as usize).map(|a| a.id);

        if let Some(id) = app_id {
            let core = CORE.lock().unwrap();
            if core.db.delete_app(id).is_ok() {
                drop(core);
                self.load_apps();
            }
        }
    }

    /// Get available package types for an app as JSON array
    /// Returns: [{"type": "deb", "name": "Debian Package", "filename": "app.deb", "size": 1234}, ...]
    fn get_available_packages(&mut self, index: i32) -> QString {
        let app_data = self.apps.get(index as usize).cloned();
        
        if let Some(app) = app_data {
            let core = CORE.lock().unwrap();
            let gh = core.github.clone();
            drop(core);
            
            let owner = app.repo_owner.clone();
            let repo = app.repo_name.clone();
            
            // Use blocking runtime to fetch release info
            let rt = tokio::runtime::Runtime::new().unwrap();
            if let Ok(release) = rt.block_on(gh.get_latest_release(&owner, &repo)) {
                let mut packages: Vec<serde_json::Value> = Vec::new();
                
                for asset in &release.assets {
                    if asset.is_linux() && asset.matches_architecture() {
                        if let Some(install_type) = asset.detect_install_type() {
                            packages.push(serde_json::json!({
                                "type": install_type.as_str(),
                                "name": install_type.display_name(),
                                "filename": asset.name,
                                "size": asset.size,
                                "url": asset.browser_download_url
                            }));
                        }
                    }
                }
                
                if let Ok(json) = serde_json::to_string(&packages) {
                    return QString::from(json);
                }
            }
        }
        QString::from("[]")
    }

    /// Install app with a specific package type
    fn install_app_with_type(&mut self, index: i32, install_type_str: QString) {
        let app_data = self.apps.get(index as usize).cloned();
        let type_str = install_type_str.to_string();
        
        if let Some(app) = app_data {
            self.loading = true;
            self.loading_changed();

            let core = CORE.lock().unwrap();
            let db = core.db.clone();
            let gh = core.github.clone();
            let inst = core.installer.clone();
            drop(core);

            let app_id = app.id;
            let owner = app.repo_owner.clone();
            let repo = app.repo_name.clone();

            // Use blocking runtime to ensure install completes before UI updates
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                if let Ok(release) = gh.get_latest_release(&owner, &repo).await {
                    // Find the asset matching the requested type
                    let requested_type = crate::core::models::InstallType::from_str(&type_str);
                    
                    let asset = release.assets.iter().find(|a| {
                        a.is_linux() && a.matches_architecture() && a.detect_install_type() == requested_type
                    });
                    
                    if let Some(asset) = asset {
                        if let Some(install_type) = asset.detect_install_type() {
                            let dest = inst.downloads_dir().join(&asset.name);

                            if gh.download_asset(&asset.browser_download_url, &dest).await.is_ok() {
                                if inst.install(&dest, install_type).is_ok() {
                                    let _ = db.update_installed(app_id, &release.tag_name, install_type);
                                }
                                let _ = std::fs::remove_file(&dest);
                                return true;
                            }
                        }
                    }
                }
                false
            });

            self.loading = false;
            self.loading_changed();
            if result {
                self.load_apps();
            }
        }
    }

    fn install_app(&mut self, index: i32) {
        let app_data = self.apps.get(index as usize).cloned();

        if let Some(app) = app_data {
            log::info!("Starting install/update for app: {} (index: {})", app.display_name, index);
            self.loading = true;
            self.loading_changed();

            let core = CORE.lock().unwrap();
            let db = core.db.clone();
            let gh = core.github.clone();
            let inst = core.installer.clone();
            drop(core);

            let app_id = app.id;
            let owner = app.repo_owner.clone();
            let repo = app.repo_name.clone();
            let display_name = app.display_name.clone();

            // Use blocking runtime to ensure install completes before UI updates
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(async {
                log::info!("Fetching latest release for {}/{}", owner, repo);
                match gh.get_latest_release(&owner, &repo).await {
                    Ok(release) => {
                        log::info!("Got release: {} with {} assets", release.tag_name, release.assets.len());
                        
                        // Get the tracked app to find preferred install type
                        let install_type_pref = db
                            .get_app(app_id)
                            .ok()
                            .flatten()
                            .and_then(|a| a.install_type);
                        
                        log::info!("Preferred install type: {:?}", install_type_pref);

                        if let Some(asset) = gh.find_best_asset(&release.assets, install_type_pref) {
                            log::info!("Found asset: {} ({})", asset.name, asset.browser_download_url);
                            
                            if let Some(install_type) = asset.detect_install_type() {
                                let dest = inst.downloads_dir().join(&asset.name);
                                log::info!("Downloading to: {:?}", dest);

                                match gh.download_asset(&asset.browser_download_url, &dest).await {
                                    Ok(_) => {
                                        log::info!("Download complete, installing as {:?}", install_type);
                                        match inst.install(&dest, install_type) {
                                            Ok(_) => {
                                                log::info!("Installation successful!");
                                                let _ = db.update_installed(app_id, &release.tag_name, install_type);
                                                let _ = std::fs::remove_file(&dest);
                                                return true;
                                            }
                                            Err(e) => {
                                                log::error!("Installation failed: {}", e);
                                                let _ = std::fs::remove_file(&dest);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        log::error!("Download failed: {}", e);
                                    }
                                }
                            } else {
                                log::error!("Could not detect install type for asset: {}", asset.name);
                            }
                        } else {
                            log::error!("No compatible asset found for {}", display_name);
                            for asset in &release.assets {
                                log::debug!("  Asset: {} (linux={}, arch_match={})", 
                                    asset.name, asset.is_linux(), asset.matches_architecture());
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Failed to get release: {}", e);
                    }
                }
                false
            });

            self.loading = false;
            self.loading_changed();
            if result {
                self.load_apps();
            }
        }
    }

    fn uninstall_app(&mut self, index: i32) {
        let app_data = self.apps.get(index as usize).cloned();

        if let Some(app) = app_data {
            let core = CORE.lock().unwrap();
            if let Ok(Some(tracked_app)) = core.db.get_app(app.id) {
                if let Some(install_type) = tracked_app.install_type {
                    if core.installer.uninstall(&app.repo_name, install_type).is_ok() {
                        let _ = core.db.clear_installed(app.id);
                        drop(core);
                        self.load_apps();
                    }
                }
            }
        }
    }

    fn refresh(&mut self) {
        self.loading = true;
        self.loading_changed();

        let core = CORE.lock().unwrap();
        if let Ok(apps) = core.db.get_all_apps() {
            let db = core.db.clone();
            let gh = core.github.clone();
            drop(core);

            // Use blocking runtime to fetch all updates
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                for app in apps {
                    let owner = app.repo_owner.clone();
                    let repo = app.repo_name.clone();
                    let app_id = app.id;

                    if let Ok(release) = gh.get_latest_release(&owner, &repo).await {
                        let _ = db.update_latest_version(app_id, &release.tag_name);
                    }
                }
            });
        } else {
            drop(core);
        }

        self.loading = false;
        self.loading_changed();
        self.load_apps();
        self.apps_refreshed();
    }

    fn update_all(&mut self) {
        let apps: Vec<AppData> = self.apps.clone();
        for (index, app) in apps.iter().enumerate() {
            if app.has_update {
                self.install_app(index as i32);
            }
        }
    }

    fn get_app_id(&self, index: i32) -> i64 {
        self.apps.get(index as usize).map(|a| a.id).unwrap_or(-1)
    }

    fn get_display_name(&self, index: i32) -> QString {
        self.apps
            .get(index as usize)
            .map(|a| QString::from(a.display_name.as_str()))
            .unwrap_or_default()
    }

    fn get_repo_info(&self, index: i32) -> QString {
        self.apps
            .get(index as usize)
            .map(|a| QString::from(format!("{}/{}", a.repo_owner, a.repo_name)))
            .unwrap_or_default()
    }

    fn get_installed_version(&self, index: i32) -> QString {
        self.apps
            .get(index as usize)
            .map(|a| QString::from(a.installed_version.as_str()))
            .unwrap_or_default()
    }

    fn get_latest_version(&self, index: i32) -> QString {
        self.apps
            .get(index as usize)
            .map(|a| QString::from(a.latest_version.as_str()))
            .unwrap_or_default()
    }

    fn get_install_type(&self, index: i32) -> QString {
        self.apps
            .get(index as usize)
            .map(|a| QString::from(a.install_type.as_str()))
            .unwrap_or_default()
    }

    fn has_update(&self, index: i32) -> bool {
        self.apps
            .get(index as usize)
            .map(|a| a.has_update)
            .unwrap_or(false)
    }

    fn is_installed(&self, index: i32) -> bool {
        self.apps
            .get(index as usize)
            .map(|a| a.is_installed)
            .unwrap_or(false)
    }
}

impl QAbstractListModel for AppModel {
    fn row_count(&self) -> i32 {
        self.apps.len() as i32
    }

    fn data(&self, index: QModelIndex, role: i32) -> QVariant {
        let row = index.row() as usize;

        if let Some(app) = self.apps.get(row) {
            match role {
                0 => QVariant::from(app.id as i32), // Qt doesn't support i64 directly
                1 => QVariant::from(QString::from(app.display_name.as_str())),
                2 => QVariant::from(QString::from(format!("{}/{}", app.repo_owner, app.repo_name))),
                3 => QVariant::from(QString::from(app.installed_version.as_str())),
                4 => QVariant::from(QString::from(app.latest_version.as_str())),
                5 => QVariant::from(QString::from(app.install_type.as_str())),
                6 => QVariant::from(app.has_update),
                7 => QVariant::from(app.is_installed),
                _ => QVariant::default(),
            }
        } else {
            QVariant::default()
        }
    }

    fn role_names(&self) -> std::collections::HashMap<i32, QByteArray> {
        let mut roles = std::collections::HashMap::new();
        roles.insert(0, QByteArray::from("appId"));
        roles.insert(1, QByteArray::from("displayName"));
        roles.insert(2, QByteArray::from("repoInfo"));
        roles.insert(3, QByteArray::from("installedVersion"));
        roles.insert(4, QByteArray::from("latestVersion"));
        roles.insert(5, QByteArray::from("installType"));
        roles.insert(6, QByteArray::from("hasUpdate"));
        roles.insert(7, QByteArray::from("isInstalled"));
        roles
    }
}
