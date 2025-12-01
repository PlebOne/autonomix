use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Label, Orientation};
use libadwaita::prelude::*;
use libadwaita::ActionRow;

use crate::core::models::TrackedApp;

/// A row displaying an app in the list
pub struct AppRow {
    pub row: ActionRow,
    pub install_button: Button,
    pub delete_button: Button,
    pub app_id: i64,
}

impl AppRow {
    pub fn new(app: &TrackedApp) -> Self {
        let row = ActionRow::builder()
            .title(&app.display_name)
            .activatable(true)
            .build();

        // Set subtitle based on status
        let subtitle = Self::build_subtitle(app);
        row.set_subtitle(&subtitle);

        // Add status indicator and buttons
        let button_box = GtkBox::builder()
            .orientation(Orientation::Horizontal)
            .spacing(6)
            .valign(Align::Center)
            .build();

        // Version labels
        let version_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .valign(Align::Center)
            .margin_end(12)
            .build();

        if let Some(installed) = &app.installed_version {
            let installed_label = Label::builder()
                .label(&format!("Installed: {}", installed))
                .css_classes(["dim-label", "caption"])
                .halign(Align::End)
                .build();
            version_box.append(&installed_label);
        }

        if let Some(latest) = &app.latest_version {
            let latest_label = Label::builder()
                .label(&format!("Latest: {}", latest))
                .css_classes(["dim-label", "caption"])
                .halign(Align::End)
                .build();
            version_box.append(&latest_label);
        }

        row.add_suffix(&version_box);

        // Install/Update button
        let install_button = if app.has_update() {
            Button::builder()
                .label("Update")
                .css_classes(["suggested-action", "pill"])
                .valign(Align::Center)
                .build()
        } else if app.is_installed() {
            Button::builder()
                .label("Reinstall")
                .css_classes(["pill"])
                .valign(Align::Center)
                .build()
        } else {
            Button::builder()
                .label("Install")
                .css_classes(["suggested-action", "pill"])
                .valign(Align::Center)
                .build()
        };

        // Disable install if no latest version known
        if app.latest_version.is_none() {
            install_button.set_sensitive(false);
        }

        button_box.append(&install_button);

        // Delete button
        let delete_button = Button::builder()
            .icon_name("user-trash-symbolic")
            .css_classes(["flat", "circular"])
            .valign(Align::Center)
            .tooltip_text("Remove from tracking")
            .build();

        button_box.append(&delete_button);

        row.add_suffix(&button_box);

        // Add prefix icon based on status
        let status_icon = if app.has_update() {
            "software-update-available-symbolic"
        } else if app.is_installed() {
            "emblem-ok-symbolic"
        } else {
            "folder-download-symbolic"
        };

        row.add_prefix(&gtk4::Image::from_icon_name(status_icon));

        Self {
            row,
            install_button,
            delete_button,
            app_id: app.id,
        }
    }

    fn build_subtitle(app: &TrackedApp) -> String {
        let mut parts = vec![format!("{}/{}", app.repo_owner, app.repo_name)];

        if let Some(install_type) = &app.install_type {
            parts.push(install_type.display_name().to_string());
        }

        if app.has_update() {
            parts.push("Update available".to_string());
        }

        parts.join(" • ")
    }

    /// Connect to install button clicked
    pub fn connect_install<F: Fn(i64) + Clone + 'static>(&self, f: F) {
        let app_id = self.app_id;
        self.install_button.connect_clicked(move |_| {
            f(app_id);
        });
    }

    /// Connect to delete button clicked
    pub fn connect_delete<F: Fn(i64) + Clone + 'static>(&self, f: F) {
        let app_id = self.app_id;
        self.delete_button.connect_clicked(move |_| {
            f(app_id);
        });
    }
}
