mod services;

use services::{AudioService, BookMetadata, ABSConfig};

use qmetaobject::*;
use qmetaobject::prelude::*;
use std::cell::RefCell;
use std::path::PathBuf;
use tokio::sync::mpsc;
use std::sync::Arc;

// --- Thread Communication Types ---

/// Messages sent from background tasks to the main UI thread
#[derive(Debug, Clone)]
pub enum UiMessage {
    StatusUpdate(String),
    ProgressUpdate(f64),
    ProcessingState(bool),
    MetadataLoaded {
        title: String,
        author: String,
        series: String,
        narrator: String,
        cover_url: String,
    },
    LogMessage(String),
    ConversionComplete,
    Error(String),
}

/// Updates sent during conversion process
#[derive(Debug, Clone)]
pub enum ConversionUpdate {
    Progress(f64, String),
    Log(String),
    Complete(bool, String),
}

/// Background task handle for managing async operations
pub struct BackgroundTask {
    sender: mpsc::UnboundedSender<UiMessage>,
    _handle: tokio::task::JoinHandle<()>,
}

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
    metadata_cover_url: qt_property!(QString; NOTIFY metadata_changed),

    // ABS settings
    abs_host: qt_property!(QString),
    abs_token: qt_property!(QString),
    abs_library_id: qt_property!(QString),

    // Thread communication
    ui_sender: Option<mpsc::UnboundedSender<UiMessage>>,
    ui_receiver: Option<mpsc::UnboundedReceiver<UiMessage>>,

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
    /// Initialize the controller with communication channels
    fn initialize(&mut self, ui_sender: mpsc::UnboundedSender<UiMessage>, ui_receiver: mpsc::UnboundedReceiver<UiMessage>) {
        self.ui_sender = Some(ui_sender);
        self.ui_receiver = Some(ui_receiver);

        // Load config
        self.load_config();
    }

    fn load_config(&mut self) {
        // Load from config file
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
                // Load ABS settings if available
                if let Some(host) = config.get("abs_host").and_then(|v| v.as_str()) {
                    self.abs_host = QString::from(host);
                }
                if let Some(token) = config.get("abs_token").and_then(|v| v.as_str()) {
                    self.abs_token = QString::from(token);
                }
                if let Some(library_id) = config.get("abs_library_id").and_then(|v| v.as_str()) {
                    self.abs_library_id = QString::from(library_id);
                }
            }
        }

        self.status_message = QString::from("Ready to process audiobooks");
        self.status_changed();
    }

    fn save_config(&self) {
        // Save current settings to config file
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        let config = serde_json::json!({
            "abs_host": self.abs_host.to_string(),
            "abs_token": self.abs_token.to_string(),
            "abs_library_id": self.abs_library_id.to_string(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            let _ = std::fs::write(config_path, json);
        }
    }

    /// Send a message to the UI update handler
    fn send_ui_message(&self, message: UiMessage) {
        if let Some(sender) = &self.ui_sender {
            let _ = sender.send(message);
        }
    }

    /// Public method to process pending UI messages (call this regularly)
    pub fn update_ui(&mut self) {
        self.process_ui_messages();
    }

    fn set_folder_path(&mut self, path: QString) {
        self.current_folder = path.clone();
        self.folder_changed();
        self.send_ui_message(UiMessage::StatusUpdate(format!("Loaded folder: {}", path.to_string())));
    }

    fn search_metadata(&mut self, query: QString, _by_asin: bool) {
        let query_str = query.to_string();

        // Set loading state immediately on main thread
        self.is_processing = true;
        self.status_message = QString::from(format!("Searching for '{}'...", query_str));
        self.progress_value = 0.1;
        self.processing_changed();
        self.status_changed();
        self.progress_changed();

        // Create thread-safe callback to update UI from background thread
        let qptr = QPointer::from(&*self);
        let update_ui = queued_callback(move |result: Result<BookMetadata, String>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut controller = pinned.borrow_mut();
                match result {
                    Ok(metadata) => {
                        controller.progress_value = 1.0;
                        controller.metadata_title = QString::from(metadata.title);
                        controller.metadata_author = QString::from(metadata.authors.join(", "));
                        controller.metadata_series = QString::from(metadata.series_name.unwrap_or_default());
                        controller.metadata_narrator = QString::from(metadata.narrator_names.map(|n| n.join(", ")).unwrap_or_default());
                        controller.metadata_cover_url = QString::from(metadata.cover_url.unwrap_or_default());
                        controller.status_message = QString::from("Metadata search completed");
                        controller.metadata_changed();
                        controller.status_changed();
                        controller.progress_changed();
                    }
                    Err(error) => {
                        controller.status_message = QString::from(format!("Search failed: {}", error));
                        controller.error_occurred(QString::from(format!("Search failed: {}", error)));
                        controller.status_changed();
                    }
                }
                controller.is_processing = false;
                controller.processing_changed();
            }
        });

        // Start background task
        std::thread::spawn(move || {
            // Perform the actual API search on background thread
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(AudioService::fetch_metadata(&query_str));
            update_ui(result);
        });
    }

    fn start_conversion(&mut self) {
        // Get current metadata and folder
        let folder = self.current_folder.to_string();
        let metadata = BookMetadata {
            title: self.metadata_title.to_string(),
            authors: vec![self.metadata_author.to_string()],
            narrator_names: if self.metadata_narrator.to_string().is_empty() {
                None
            } else {
                Some(vec![self.metadata_narrator.to_string()])
            },
            series_name: if self.metadata_series.to_string().is_empty() {
                None
            } else {
                Some(self.metadata_series.to_string())
            },
            cover_url: if self.metadata_cover_url.to_string().is_empty() {
                None
            } else {
                Some(self.metadata_cover_url.to_string())
            },
            asin: None, // TODO: extract from search
            duration_minutes: None,
            release_date: None,
        };

        let abs_config = ABSConfig {
            host: self.abs_host.to_string(),
            token: self.abs_token.to_string(),
            library_id: self.abs_library_id.to_string(),
        };

        // Set initial loading state on main thread
        self.is_processing = true;
        self.status_message = QString::from("Starting audio conversion...");
        self.progress_value = 0.0;
        self.processing_changed();
        self.status_changed();
        self.progress_changed();

        // Create thread-safe callback for conversion updates
        let qptr = QPointer::from(&*self);
        let update_progress = queued_callback(move |update: ConversionUpdate| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut controller = pinned.borrow_mut();
                match update {
                    ConversionUpdate::Progress(progress, message) => {
                        controller.progress_value = progress;
                        controller.status_message = QString::from(message);
                        controller.progress_changed();
                        controller.status_changed();
                    }
                    ConversionUpdate::Log(message) => {
                        controller.log_message(QString::from(message));
                    }
                    ConversionUpdate::Complete(success, message) => {
                        controller.progress_value = 1.0;
                        controller.status_message = QString::from(message.clone());
                        controller.is_processing = false;
                        if success {
                            controller.conversion_completed();
                        } else {
                            controller.error_occurred(QString::from(message));
                        }
                        controller.progress_changed();
                        controller.status_changed();
                        controller.processing_changed();
                    }
                }
            }
        });

        // Start background conversion
        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            runtime.block_on(async {
                update_progress(ConversionUpdate::Progress(0.1, "Scanning audio files...".to_string()));

                // Validate input
                let input_dir = PathBuf::from(folder);
                if !input_dir.exists() {
                    update_progress(ConversionUpdate::Complete(false, "Input directory does not exist".to_string()));
                    return;
                }

                // Generate output path
                let output_path = input_dir
                    .parent()
                    .unwrap_or(&input_dir)
                    .join(format!("{}.m4b", metadata.title.replace(":", "").replace("/", "")));

                update_progress(ConversionUpdate::Progress(0.2, "Converting audio files...".to_string()));

                // Convert to M4B
                match AudioService::convert_to_m4b_with_chapters(&input_dir, &output_path.to_string_lossy(), &metadata).await {
                    Ok(_) => {
                        update_progress(ConversionUpdate::Progress(0.7, "Applying metadata...".to_string()));

                        // Apply metadata
                        match AudioService::apply_tags(&output_path.to_string_lossy(), &metadata).await {
                            Ok(_) => {
                                update_progress(ConversionUpdate::Progress(0.9, "Uploading to Audiobookshelf...".to_string()));

                                // Upload to ABS if configured
                                if !abs_config.host.is_empty() && !abs_config.token.is_empty() {
                                    match AudioService::upload_and_scan(&output_path.to_string_lossy(), &abs_config).await {
                                        Ok(_) => {
                                            update_progress(ConversionUpdate::Complete(true, "Conversion and upload completed!".to_string()));
                                        }
                                        Err(e) => {
                                            update_progress(ConversionUpdate::Log(format!("Upload error: {}", e)));
                                            update_progress(ConversionUpdate::Complete(true, "Conversion completed, upload failed".to_string()));
                                        }
                                    }
                                } else {
                                    update_progress(ConversionUpdate::Complete(true, "Conversion completed!".to_string()));
                                }
                            }
                            Err(e) => {
                                update_progress(ConversionUpdate::Complete(false, format!("Metadata application failed: {}", e)));
                            }
                        }
                    }
                    Err(e) => {
                        update_progress(ConversionUpdate::Complete(false, format!("Conversion failed: {}", e)));
                    }
                }
            });
        });
    }

    fn cancel_conversion(&mut self) {
        // For now, just reset the state (background task cancellation will be implemented later)
        self.is_processing = false;
        self.processing_changed();
        self.status_message = QString::from("Operation cancelled");
        self.status_changed();
    }

    /// Process UI messages from background threads (call this regularly from Qt event loop)
    fn process_ui_messages(&mut self) {
        let mut messages = Vec::new();

        // First, collect all available messages to avoid holding the mutable borrow
        if let Some(receiver) = &mut self.ui_receiver {
            while let Ok(message) = receiver.try_recv() {
                messages.push(message);
            }
        }

        // Then process the messages without holding the receiver borrow
        for message in messages {
            match message {
                UiMessage::StatusUpdate(msg) => {
                    self.status_message = QString::from(msg);
                    self.status_changed();
                }
                UiMessage::ProgressUpdate(progress) => {
                    self.progress_value = progress;
                    self.progress_changed();
                }
                UiMessage::ProcessingState(processing) => {
                    self.is_processing = processing;
                    self.processing_changed();
                }
                UiMessage::MetadataLoaded { title, author, series, narrator, cover_url } => {
                    self.metadata_title = QString::from(title);
                    self.metadata_author = QString::from(author);
                    self.metadata_series = QString::from(series);
                    self.metadata_narrator = QString::from(narrator);
                    self.metadata_cover_url = QString::from(cover_url);
                    self.metadata_changed();
                }
                UiMessage::LogMessage(msg) => {
                    self.log_message(QString::from(msg));
                }
                UiMessage::ConversionComplete => {
                    self.conversion_completed();
                }
                UiMessage::Error(msg) => {
                    self.error_occurred(QString::from(msg));
                }
            }
        }
    }
}

fn main() {
    println!("🎵 Qt Lectern - Full GUI Application with Thread-Safe Updates");

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

    // Set up thread communication channels
    let (ui_sender, mut ui_receiver) = mpsc::unbounded_channel();
    println!("✅ Thread communication channels created");

    // Create the controller wrapped in Arc<RefCell<>> and initialize it
    let controller = Arc::new(RefCell::new(LecternController::default()));
    controller.borrow_mut().initialize(ui_sender, ui_receiver);
    println!("✅ LecternController created and initialized");

    // Create QML engine
    let mut engine = QmlEngine::new();
    println!("✅ QmlEngine created");

    // Register the controller with QML
    unsafe {
        let controller_ptr = QObjectPinned::new(&*controller);
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
    println!("💡 Background tasks will update UI thread-safely");
    println!("💡 Press Ctrl+C to exit");

    // Start the Qt event loop
    println!("🚀 Starting Qt event loop...");

    // For now, we'll process messages synchronously in the main thread
    // In a production app, you'd integrate this with Qt's signal-slot system
    engine.exec();

    println!("✅ Qt event loop completed");
}
