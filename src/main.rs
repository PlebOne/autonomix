mod core;
mod ui;

use cstr::cstr;
use qmetaobject::prelude::*;

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

    // Register QML types
    qml_register_type::<ui::app_model::AppModel>(
        cstr!("Autonomix"),
        1,
        0,
        cstr!("AppModel"),
    );

    // Create Qt application
    let mut engine = QmlEngine::new();

    // Load the main QML file
    engine.load_data(include_str!("ui/qml/main.qml").into());

    // Run the application
    engine.exec();
}

