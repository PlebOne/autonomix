mod core;
mod ui;

use gtk4::prelude::*;

fn main() {
    // Initialize logging
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    log::info!("Starting Autonomix v{}", env!("CARGO_PKG_VERSION"));

    // Initialize tokio runtime for async operations
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed to create tokio runtime");

    // Keep runtime alive
    let _guard = rt.enter();

    // Create the GTK application
    let app = libadwaita::Application::builder()
        .application_id("io.github.plebone.autonomix")
        .build();

    app.connect_activate(|app| {
        let window = ui::main_window::MainWindow::new(app);
        window.window.present();
    });

    // Run the application
    let args: Vec<String> = std::env::args().collect();
    app.run_with_args(&args);
}

