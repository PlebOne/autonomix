use gtk4::prelude::*;
use gtk4::{gio, glib, Box as GtkBox, ListBox, Orientation, ScrolledWindow, SelectionMode};
use libadwaita::prelude::*;
use libadwaita::{
    Application, ApplicationWindow, HeaderBar, StatusPage, Toast, ToastOverlay,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::mpsc;

use crate::core::database::Database;
use crate::core::github_api::GitHubApi;
use crate::core::installer::Installer;
use crate::core::models::{InstallType, TrackedApp};

use super::add_dialog::AddAppDialog;
use super::app_row::AppRow;

/// Messages for async operations
#[derive(Debug)]
pub enum AppMessage {
    AppsLoaded(Vec<TrackedApp>),
    AppAdded(TrackedApp),
    AppUpdated(i64, Option<String>), // app_id, new latest version
    AppInstalled(i64, String, InstallType),
    AppUninstalled(i64),
    AppDeleted(i64),
    Error(String),
    RefreshComplete,
    InstallComplete(i64, Result<(), String>),
    UninstallComplete(i64, Result<(), String>),
}

/// Main application window
pub struct MainWindow {
    pub window: ApplicationWindow,
    list_box: ListBox,
    toast_overlay: ToastOverlay,
    status_page: StatusPage,
    db: Database,
    github: Arc<GitHubApi>,
    installer: Arc<Installer>,
    app_rows: Rc<RefCell<Vec<AppRow>>>,
    tx: mpsc::UnboundedSender<AppMessage>,
}

impl MainWindow {
    pub fn new(app: &Application) -> Self {
        // Create message channel
        let (tx, rx) = mpsc::unbounded_channel::<AppMessage>();

        // Initialize core components
        let db = Database::open(None).expect("Failed to open database");
        let github = Arc::new(GitHubApi::new().expect("Failed to create GitHub API client"));
        let installer = Arc::new(Installer::new().expect("Failed to create installer"));

        // Register self for updates
        let _ = db.register_self();

        // Create the main window
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Autonomix")
            .default_width(800)
            .default_height(600)
            .build();

        // Header bar
        let header = HeaderBar::new();

        // Add button
        let add_button = gtk4::Button::builder()
            .icon_name("list-add-symbolic")
            .tooltip_text("Add application")
            .build();

        // Refresh button
        let refresh_button = gtk4::Button::builder()
            .icon_name("view-refresh-symbolic")
            .tooltip_text("Check for updates")
            .build();

        // Menu button
        let menu_button = gtk4::MenuButton::builder()
            .icon_name("open-menu-symbolic")
            .tooltip_text("Menu")
            .build();

        // Create menu
        let menu = gio::Menu::new();
        menu.append(Some("Update All"), Some("app.update-all"));
        menu.append(Some("About Autonomix"), Some("app.about"));
        menu_button.set_menu_model(Some(&menu));

        header.pack_start(&add_button);
        header.pack_end(&menu_button);
        header.pack_end(&refresh_button);

        // Main content
        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Toast overlay for notifications
        let toast_overlay = ToastOverlay::new();

        // List box for apps
        let list_box = ListBox::builder()
            .selection_mode(SelectionMode::None)
            .css_classes(["boxed-list"])
            .margin_start(24)
            .margin_end(24)
            .margin_top(24)
            .margin_bottom(24)
            .build();

        // Empty state
        let status_page = StatusPage::builder()
            .icon_name("application-x-addon-symbolic")
            .title("No Applications")
            .description("Add a GitHub repository to start tracking releases")
            .build();

        let add_first_button = gtk4::Button::builder()
            .label("Add Application")
            .css_classes(["suggested-action", "pill"])
            .halign(gtk4::Align::Center)
            .build();

        status_page.set_child(Some(&add_first_button));

        // Scrolled window
        let scrolled = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();

        scrolled.set_child(Some(&list_box));

        content.append(&status_page);
        content.append(&scrolled);

        toast_overlay.set_child(Some(&content));

        // Main box with header
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        main_box.append(&header);
        main_box.append(&toast_overlay);

        window.set_content(Some(&main_box));

        let main_window = Self {
            window: window.clone(),
            list_box,
            toast_overlay,
            status_page,
            db: db.clone(),
            github,
            installer,
            app_rows: Rc::new(RefCell::new(Vec::new())),
            tx: tx.clone(),
        };

        // Set up message handling
        main_window.setup_message_handler(rx);

        // Connect button signals
        let tx_clone = tx.clone();
        let db_clone = db.clone();
        let window_clone = window.clone();
        add_button.connect_clicked(move |_| {
            Self::show_add_dialog(&window_clone, tx_clone.clone(), db_clone.clone());
        });

        let tx_clone = tx.clone();
        let db_clone = db.clone();
        let window_clone = window.clone();
        add_first_button.connect_clicked(move |_| {
            Self::show_add_dialog(&window_clone, tx_clone.clone(), db_clone.clone());
        });

        let tx_clone = tx.clone();
        let db_clone = db.clone();
        let github_clone = main_window.github.clone();
        refresh_button.connect_clicked(move |_| {
            Self::refresh_all_apps(tx_clone.clone(), db_clone.clone(), github_clone.clone());
        });

        // Set up actions
        main_window.setup_actions(app);

        // Load initial apps
        main_window.load_apps();

        main_window
    }

    /// Set up application actions
    fn setup_actions(&self, app: &Application) {
        // Update all action
        let tx = self.tx.clone();
        let db = self.db.clone();
        let github = self.github.clone();
        let installer = self.installer.clone();
        let update_all_action = gio::SimpleAction::new("update-all", None);
        update_all_action.connect_activate(move |_, _| {
            Self::update_all_apps(
                tx.clone(),
                db.clone(),
                github.clone(),
                installer.clone(),
            );
        });
        app.add_action(&update_all_action);

        // About action
        let window = self.window.clone();
        let about_action = gio::SimpleAction::new("about", None);
        about_action.connect_activate(move |_, _| {
            let dialog = gtk4::AboutDialog::builder()
                .program_name("Autonomix")
                .logo_icon_name("application-x-addon-symbolic")
                .version(env!("CARGO_PKG_VERSION"))
                .authors(vec!["PlebOne".to_string()])
                .website("https://github.com/PlebOne/autonomix")
                .license_type(gtk4::License::MitX11)
                .comments("A Linux package manager for GitHub releases")
                .transient_for(&window)
                .modal(true)
                .build();
            dialog.present();
        });
        app.add_action(&about_action);
    }

    /// Set up the async message handler
    fn setup_message_handler(&self, mut rx: mpsc::UnboundedReceiver<AppMessage>) {
        let list_box = self.list_box.clone();
        let status_page = self.status_page.clone();
        let toast_overlay = self.toast_overlay.clone();
        let app_rows = Rc::clone(&self.app_rows);
        let db = self.db.clone();
        let github = self.github.clone();
        let installer = self.installer.clone();
        let tx = self.tx.clone();

        glib::spawn_future_local(async move {
            while let Some(msg) = rx.recv().await {
                match msg {
                    AppMessage::AppsLoaded(apps) => {
                        // Clear existing rows
                        while let Some(child) = list_box.first_child() {
                            list_box.remove(&child);
                        }
                        app_rows.borrow_mut().clear();

                        // Show/hide empty state
                        status_page.set_visible(apps.is_empty());
                        list_box.set_visible(!apps.is_empty());

                        // Add rows for each app
                        for app in apps {
                            let row = AppRow::new(&app);

                            // Connect install button
                            let tx_clone = tx.clone();
                            let db_clone = db.clone();
                            let github_clone = github.clone();
                            let installer_clone = installer.clone();
                            row.connect_install(move |app_id| {
                                Self::install_app(
                                    app_id,
                                    tx_clone.clone(),
                                    db_clone.clone(),
                                    github_clone.clone(),
                                    installer_clone.clone(),
                                );
                            });

                            // Connect uninstall button
                            let tx_clone = tx.clone();
                            let db_clone = db.clone();
                            let installer_clone = installer.clone();
                            row.connect_uninstall(move |app_id| {
                                Self::uninstall_app(
                                    app_id,
                                    tx_clone.clone(),
                                    db_clone.clone(),
                                    installer_clone.clone(),
                                );
                            });

                            // Connect delete button
                            let tx_clone = tx.clone();
                            let db_clone = db.clone();
                            row.connect_delete(move |app_id| {
                                if let Err(e) = db_clone.delete_app(app_id) {
                                    let _ = tx_clone.send(AppMessage::Error(e.to_string()));
                                } else {
                                    let _ = tx_clone.send(AppMessage::AppDeleted(app_id));
                                }
                            });

                            list_box.append(&row.row);
                            app_rows.borrow_mut().push(row);
                        }
                    }

                    AppMessage::AppAdded(app) => {
                        status_page.set_visible(false);
                        list_box.set_visible(true);

                        let row = AppRow::new(&app);

                        // Connect install button
                        let tx_clone = tx.clone();
                        let db_clone = db.clone();
                        let github_clone = github.clone();
                        let installer_clone = installer.clone();
                        row.connect_install(move |app_id| {
                            Self::install_app(
                                app_id,
                                tx_clone.clone(),
                                db_clone.clone(),
                                github_clone.clone(),
                                installer_clone.clone(),
                            );
                        });

                        // Connect uninstall button
                        let tx_clone = tx.clone();
                        let db_clone = db.clone();
                        let installer_clone = installer.clone();
                        row.connect_uninstall(move |app_id| {
                            Self::uninstall_app(
                                app_id,
                                tx_clone.clone(),
                                db_clone.clone(),
                                installer_clone.clone(),
                            );
                        });

                        // Connect delete button
                        let tx_clone = tx.clone();
                        let db_clone = db.clone();
                        row.connect_delete(move |app_id| {
                            if let Err(e) = db_clone.delete_app(app_id) {
                                let _ = tx_clone.send(AppMessage::Error(e.to_string()));
                            } else {
                                let _ = tx_clone.send(AppMessage::AppDeleted(app_id));
                            }
                        });

                        list_box.append(&row.row);
                        app_rows.borrow_mut().push(row);

                        let toast = Toast::new("Application added");
                        toast_overlay.add_toast(toast);
                    }

                    AppMessage::AppDeleted(app_id) => {
                        // Remove the row
                        let mut rows = app_rows.borrow_mut();
                        if let Some(pos) = rows.iter().position(|r| r.app_id == app_id) {
                            let row = rows.remove(pos);
                            list_box.remove(&row.row);
                        }

                        // Show empty state if no apps left
                        if rows.is_empty() {
                            status_page.set_visible(true);
                            list_box.set_visible(false);
                        }

                        let toast = Toast::new("Application removed");
                        toast_overlay.add_toast(toast);
                    }

                    AppMessage::RefreshComplete => {
                        let toast = Toast::new("Update check complete");
                        toast_overlay.add_toast(toast);

                        // Reload to show updated versions
                        if let Ok(apps) = db.get_all_apps() {
                            let _ = tx.send(AppMessage::AppsLoaded(apps));
                        }
                    }

                    AppMessage::InstallComplete(_app_id, result) => {
                        match result {
                            Ok(()) => {
                                let toast = Toast::new("Installation complete");
                                toast_overlay.add_toast(toast);

                                // Reload to show updated status
                                if let Ok(apps) = db.get_all_apps() {
                                    let _ = tx.send(AppMessage::AppsLoaded(apps));
                                }
                            }
                            Err(e) => {
                                let toast = Toast::new(&format!("Installation failed: {}", e));
                                toast_overlay.add_toast(toast);
                            }
                        }
                    }

                    AppMessage::UninstallComplete(_app_id, result) => {
                        match result {
                            Ok(()) => {
                                let toast = Toast::new("Uninstallation complete");
                                toast_overlay.add_toast(toast);

                                // Reload to show updated status
                                if let Ok(apps) = db.get_all_apps() {
                                    let _ = tx.send(AppMessage::AppsLoaded(apps));
                                }
                            }
                            Err(e) => {
                                let toast = Toast::new(&format!("Uninstallation failed: {}", e));
                                toast_overlay.add_toast(toast);
                            }
                        }
                    }

                    AppMessage::Error(msg) => {
                        let toast = Toast::new(&msg);
                        toast_overlay.add_toast(toast);
                    }

                    AppMessage::AppUpdated(_, _) => {
                        // Reload to show updated versions
                        if let Ok(apps) = db.get_all_apps() {
                            let _ = tx.send(AppMessage::AppsLoaded(apps));
                        }
                    }

                    _ => {}
                }
            }
        });
    }

    /// Load apps from database
    fn load_apps(&self) {
        let tx = self.tx.clone();
        let db = self.db.clone();
        let github = self.github.clone();

        glib::spawn_future_local(async move {
            match db.get_all_apps() {
                Ok(apps) => {
                    let _ = tx.send(AppMessage::AppsLoaded(apps.clone()));

                    // Auto-refresh versions
                    for app in apps {
                        let tx = tx.clone();
                        let db = db.clone();
                        let github = github.clone();

                        tokio::spawn(async move {
                            if let Ok(release) =
                                github.get_latest_release(&app.repo_owner, &app.repo_name).await
                            {
                                let _ = db.update_latest_version(app.id, &release.tag_name);
                                let _ = tx.send(AppMessage::AppUpdated(
                                    app.id,
                                    Some(release.tag_name),
                                ));
                            }
                        });
                    }
                }
                Err(e) => {
                    let _ = tx.send(AppMessage::Error(format!("Failed to load apps: {}", e)));
                }
            }
        });
    }

    /// Show the add app dialog
    fn show_add_dialog(
        window: &ApplicationWindow,
        tx: mpsc::UnboundedSender<AppMessage>,
        db: Database,
    ) {
        let dialog = AddAppDialog::new(window);

        let dialog_window = dialog.window.clone();
        let url_entry = dialog.url_entry.clone();
        let error_label = dialog.error_label.clone();
        let tx_clone = tx.clone();
        let db_clone = db.clone();

        dialog.connect_add(move || {
            let text = url_entry.text();
            if let Some((owner, repo)) = crate::core::github_api::parse_github_url(text.as_str()) {
                // Check if already tracked
                if db_clone.is_tracked(&owner, &repo).unwrap_or(false) {
                    error_label.set_text("This repository is already being tracked");
                    error_label.set_visible(true);
                    return;
                }

                // Add to database
                match db_clone.add_app(&owner, &repo, &repo) {
                    Ok(id) => {
                        let app = TrackedApp {
                            id,
                            repo_owner: owner,
                            repo_name: repo.clone(),
                            display_name: repo,
                            installed_version: None,
                            latest_version: None,
                            install_type: None,
                            last_checked: None,
                            created_at: chrono::Utc::now(),
                        };
                        let _ = tx_clone.send(AppMessage::AppAdded(app));
                        dialog_window.close();
                    }
                    Err(e) => {
                        error_label.set_text(&format!("Failed to add: {}", e));
                        error_label.set_visible(true);
                    }
                }
            }
        });

        dialog.present();
    }

    /// Refresh all apps to check for updates
    fn refresh_all_apps(
        tx: mpsc::UnboundedSender<AppMessage>,
        db: Database,
        github: Arc<GitHubApi>,
    ) {
        glib::spawn_future_local(async move {
            if let Ok(apps) = db.get_all_apps() {
                for app in apps {
                    let db = db.clone();
                    let github = github.clone();

                    tokio::spawn(async move {
                        if let Ok(release) =
                            github.get_latest_release(&app.repo_owner, &app.repo_name).await
                        {
                            let _ = db.update_latest_version(app.id, &release.tag_name);
                        }
                    });
                }
            }

            // Small delay to let updates complete
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;
            let _ = tx.send(AppMessage::RefreshComplete);
        });
    }

    /// Install an app
    fn install_app(
        app_id: i64,
        tx: mpsc::UnboundedSender<AppMessage>,
        db: Database,
        github: Arc<GitHubApi>,
        installer: Arc<Installer>,
    ) {
        glib::spawn_future_local(async move {
            let result = Self::do_install(app_id, db.clone(), github, installer).await;
            let _ = tx.send(AppMessage::InstallComplete(app_id, result.map_err(|e| e.to_string())));
        });
    }

    async fn do_install(
        app_id: i64,
        db: Database,
        github: Arc<GitHubApi>,
        installer: Arc<Installer>,
    ) -> anyhow::Result<()> {
        let app = db
            .get_app(app_id)?
            .ok_or_else(|| anyhow::anyhow!("App not found"))?;

        // Get latest release
        let release = github
            .get_latest_release(&app.repo_owner, &app.repo_name)
            .await?;

        // Find best asset
        let asset = github
            .find_best_asset(&release.assets, app.install_type)
            .ok_or_else(|| anyhow::anyhow!("No compatible package found"))?;

        let install_type = asset
            .detect_install_type()
            .ok_or_else(|| anyhow::anyhow!("Unknown package type"))?;

        // Download
        let downloads_dir = installer.downloads_dir();
        let dest = downloads_dir.join(&asset.name);

        log::info!("Downloading {} to {:?}", asset.name, dest);
        github.download_asset(&asset.browser_download_url, &dest).await?;

        // Install
        log::info!("Installing {:?} as {:?}", dest, install_type);
        installer.install(&dest, install_type)?;

        // Update database
        db.update_installed(app_id, &release.tag_name, install_type)?;

        // Cleanup download
        let _ = std::fs::remove_file(&dest);

        Ok(())
    }

    /// Uninstall an app
    fn uninstall_app(
        app_id: i64,
        tx: mpsc::UnboundedSender<AppMessage>,
        db: Database,
        installer: Arc<Installer>,
    ) {
        glib::spawn_future_local(async move {
            let result = Self::do_uninstall(app_id, db.clone(), installer);
            let _ = tx.send(AppMessage::UninstallComplete(app_id, result.map_err(|e| e.to_string())));
        });
    }

    fn do_uninstall(
        app_id: i64,
        db: Database,
        installer: Arc<Installer>,
    ) -> anyhow::Result<()> {
        let app = db
            .get_app(app_id)?
            .ok_or_else(|| anyhow::anyhow!("App not found"))?;

        let install_type = app
            .install_type
            .ok_or_else(|| anyhow::anyhow!("App was not installed through Autonomix"))?;

        // Use the repo name as the package name for uninstallation
        let package_name = &app.repo_name;

        log::info!("Uninstalling {} as {:?}", package_name, install_type);
        installer.uninstall(package_name, install_type)?;

        // Clear installed status in database
        db.clear_installed(app_id)?;

        Ok(())
    }

    /// Update all apps that have updates available
    fn update_all_apps(
        tx: mpsc::UnboundedSender<AppMessage>,
        db: Database,
        github: Arc<GitHubApi>,
        installer: Arc<Installer>,
    ) {
        glib::spawn_future_local(async move {
            if let Ok(apps) = db.get_all_apps() {
                for app in apps {
                    if app.has_update() {
                        let db = db.clone();
                        let github = github.clone();
                        let installer = installer.clone();
                        let tx = tx.clone();

                        let result = Self::do_install(app.id, db, github, installer).await;
                        let _ = tx.send(AppMessage::InstallComplete(
                            app.id,
                            result.map_err(|e| e.to_string()),
                        ));
                    }
                }
            }
        });
    }
}
