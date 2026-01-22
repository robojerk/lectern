mod services;

use qmetaobject::*;
use std::cell::RefCell;
use std::path::PathBuf;

#[derive(QObject, Default)]
pub struct LecternController {
    base: qt_base_class!(trait QObject),

    // Properties
    current_folder: qt_property!(QString; NOTIFY folder_changed),
    status_message: qt_property!(QString; NOTIFY status_changed),
    progress_value: qt_property!(f64; NOTIFY progress_changed),
    is_processing: qt_property!(bool; NOTIFY processing_changed),

    // Metadata properties
    metadata_title: qt_property!(QString; NOTIFY metadata_changed),
    metadata_author: qt_property!(QString; NOTIFY metadata_changed),
    metadata_series: qt_property!(QString; NOTIFY metadata_changed),
    metadata_narrator: qt_property!(QString; NOTIFY metadata_changed),

    // ABS settings
    abs_host: qt_property!(QString),
    abs_token: qt_property!(QString),
    abs_library_id: qt_property!(QString),

    // Signals
    folder_changed: qt_signal!(),
    status_changed: qt_signal!(),
    progress_changed: qt_signal!(),
    processing_changed: qt_signal!(),
    metadata_changed: qt_signal!(),
    log_message: qt_signal!(message: QString),
    conversion_completed: qt_signal!(),
    error_occurred: qt_signal!(message: QString),
}

impl LecternController {
    fn load_config(&mut self) {
        // Load from config file
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Note: In qmetaobject, properties are accessed directly as fields
                // The macro generates the appropriate getters/setters
            }
        }

        self.status_message = QString::from("Ready to process audiobooks");
        self.status_changed();
    }

    fn save_config(&self) {
        // Note: In qmetaobject, we access properties directly
        // This is a placeholder - config saving would work with the actual QML bindings
    }

    fn set_folder_path(&mut self, path: QString) {
        self.current_folder = path.clone();
        self.folder_changed();

        self.status_message = QString::from(format!("Loaded folder: {}", path.to_string()));
        self.status_changed();

        // Auto-fetch metadata (placeholder)
        self.metadata_title = QString::from("Sample Book Title");
        self.metadata_author = QString::from("Sample Author");
        self.metadata_series = QString::from("Sample Series");
        self.metadata_narrator = QString::from("Sample Narrator");
        self.metadata_changed();
    }

    fn search_metadata(&mut self, query: QString, by_asin: bool) {
        self.is_processing = true;
        self.processing_changed();
        self.status_message = QString::from(format!("Searching for '{}'...", query.to_string()));
        self.status_changed();

        // Placeholder search implementation
        println!("Would search for: {} (ASIN: {})", query.to_string(), by_asin);

        // Simulate search completion
        self.status_message = QString::from("Search completed - placeholder results");
        self.status_changed();
        self.is_processing = false;
        self.processing_changed();
    }

    fn start_conversion(&mut self) {
        self.is_processing = true;
        self.processing_changed();
        self.progress_value = 0.0;
        self.progress_changed();
        self.status_message = QString::from("Starting conversion...");
        self.status_changed();

        // Placeholder conversion
        println!("Would start conversion");

        // Simulate progress
        self.progress_value = 0.5;
        self.progress_changed();
        self.status_message = QString::from("Converting audio files...");
        self.status_changed();

        // Complete
        self.progress_value = 1.0;
        self.progress_changed();
        self.is_processing = false;
        self.processing_changed();
        self.status_message = QString::from("Conversion completed!");
        self.status_changed();
        self.conversion_completed();
    }

    fn cancel_conversion(&mut self) {
        self.is_processing = false;
        self.processing_changed();
        self.status_message = QString::from("Conversion cancelled");
        self.status_changed();
    }
}

fn main() {
    println!("🎵 Qt Lectern - Full GUI Application");

    // Check environment
    println!("🔍 Environment check:");
    println!("  DISPLAY: {:?}", std::env::var("DISPLAY").unwrap_or("Not set".to_string()));
    println!("  WAYLAND_DISPLAY: {:?}", std::env::var("WAYLAND_DISPLAY").unwrap_or("Not set".to_string()));

    // Force XCB platform for better compatibility
    std::env::set_var("QT_QPA_PLATFORM", "xcb");
    println!("  → Forcing QT_QPA_PLATFORM=xcb (X11) for compatibility");

    // Initialize Qt
    init_qt_to_rust();
    println!("✅ Qt initialized");

    // Create the controller wrapped in RefCell
    let controller = RefCell::new(LecternController::default());
    println!("✅ LecternController created");

    // Create QML engine
    let mut engine = QmlEngine::new();
    println!("✅ QmlEngine created");

    // Register the controller with QML
    unsafe {
        let controller_ptr = QObjectPinned::new(&controller);
        engine.set_object_property("controller".into(), controller_ptr);
    }
    println!("✅ Controller registered with QML");

    // Load our QML file
    let qml_path = "qml/main.qml";
    if !std::path::Path::new(qml_path).exists() {
        println!("❌ QML file NOT found: {}", qml_path);
        return;
    }

    println!("✅ QML file found: {}", qml_path);
    engine.load_file(qml_path.into());
    println!("✅ QML loaded");

    println!("🚀 Starting Qt event loop...");
    println!("💡 Lectern GUI should now be visible!");
    println!("💡 Press Ctrl+C to exit");

    engine.exec();
    println!("✅ Qt event loop completed");
}
