use gtk4::prelude::*;
use gtk4::{Align, Box as GtkBox, Button, Entry, Label, Orientation};
use libadwaita::prelude::*;
use libadwaita::{ActionRow, HeaderBar, PreferencesGroup, StatusPage};

use crate::core::github_api::parse_github_url;

/// Dialog for adding a new GitHub repository to track
pub struct AddAppDialog {
    pub window: libadwaita::Window,
    pub url_entry: Entry,
    add_button: Button,
    pub error_label: Label,
}

impl AddAppDialog {
    pub fn new(parent: &impl IsA<gtk4::Window>) -> Self {
        // Create the dialog window
        let window = libadwaita::Window::builder()
            .title("Add Application")
            .default_width(450)
            .default_height(350)
            .modal(true)
            .transient_for(parent)
            .build();

        // Main container
        let main_box = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .build();

        // Header bar with close button
        let header = HeaderBar::builder()
            .show_end_title_buttons(false)
            .show_start_title_buttons(false)
            .title_widget(&gtk4::Label::new(Some("Add Application")))
            .build();

        let cancel_button = Button::builder()
            .label("Cancel")
            .build();

        let add_button = Button::builder()
            .label("Add")
            .css_classes(["suggested-action"])
            .sensitive(false)
            .build();

        header.pack_start(&cancel_button);
        header.pack_end(&add_button);

        // Content
        let content = GtkBox::builder()
            .orientation(Orientation::Vertical)
            .margin_start(24)
            .margin_end(24)
            .margin_top(12)
            .margin_bottom(24)
            .spacing(18)
            .vexpand(true)
            .build();

        // Status page with icon
        let status = StatusPage::builder()
            .icon_name("list-add-symbolic")
            .title("Add GitHub Repository")
            .description("Enter a GitHub repository URL or owner/repo format")
            .build();

        // URL entry group
        let url_group = PreferencesGroup::new();

        let url_entry = Entry::builder()
            .placeholder_text("github.com/owner/repo or owner/repo")
            .hexpand(true)
            .build();

        let url_row = ActionRow::builder()
            .title("Repository")
            .build();
        url_row.add_suffix(&url_entry);

        url_group.add(&url_row);

        // Error label
        let error_label = Label::builder()
            .css_classes(["error"])
            .halign(Align::Start)
            .visible(false)
            .build();

        content.append(&status);
        content.append(&url_group);
        content.append(&error_label);

        main_box.append(&header);
        main_box.append(&content);

        window.set_content(Some(&main_box));

        // Validate input on text change
        let add_btn = add_button.clone();
        let entry = url_entry.clone();
        let err_label = error_label.clone();
        url_entry.connect_changed(move |_| {
            let text = entry.text();
            let is_valid = parse_github_url(text.as_str()).is_some();
            add_btn.set_sensitive(is_valid);
            err_label.set_visible(false);
        });

        // Handle Enter key
        let add_btn = add_button.clone();
        url_entry.connect_activate(move |_| {
            if add_btn.is_sensitive() {
                add_btn.emit_clicked();
            }
        });

        // Cancel button closes dialog
        let win = window.clone();
        cancel_button.connect_clicked(move |_| {
            win.close();
        });

        Self {
            window,
            url_entry,
            add_button,
            error_label,
        }
    }

    /// Get the parsed repository info (owner, repo) if valid
    pub fn get_repo_info(&self) -> Option<(String, String)> {
        let text = self.url_entry.text();
        parse_github_url(text.as_str())
    }

    /// Show an error message
    pub fn show_error(&self, message: &str) {
        self.error_label.set_text(message);
        self.error_label.set_visible(true);
    }

    /// Connect to the add button clicked signal
    pub fn connect_add<F: Fn() + 'static>(&self, f: F) {
        self.add_button.connect_clicked(move |_| f());
    }

    /// Present the dialog
    pub fn present(&self) {
        self.window.present();
    }

    /// Close the dialog
    pub fn close(&self) {
        self.window.close();
    }
}
