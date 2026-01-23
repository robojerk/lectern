#![allow(dead_code)]
#![allow(unused_variables)]

mod services;

use services::{AudioService, BookMetadata, ABSConfig};

use qmetaobject::*;
use std::cell::RefCell;
use std::path::PathBuf;
use url::Url;

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

    // Search result properties
    search_title: qt_property!(QString; NOTIFY search_result_changed),
    search_author: qt_property!(QString; NOTIFY search_result_changed),
    search_cover_url: qt_property!(QString; NOTIFY search_result_changed),
    search_result_changed: qt_signal!(),

    // ABS settings
    abs_host: qt_property!(QString; NOTIFY config_changed),
    abs_token: qt_property!(QString; NOTIFY config_changed),
    abs_library_id: qt_property!(QString; NOTIFY config_changed),
    config_changed: qt_signal!(),

    // Signals
    folder_changed: qt_signal!(),
    status_changed: qt_signal!(),
    progress_changed: qt_signal!(),
    processing_changed: qt_signal!(),
    metadata_changed: qt_signal!(),
    log_message: qt_signal!(message: QString),
    conversion_completed: qt_signal!(),
    error_occurred: qt_signal!(message: QString),

    // Qt Methods (callable from QML)
    save_config: qt_method!(fn(&mut self, url: QString, token: QString, id: QString)),
    set_folder_path: qt_method!(fn(&mut self, url_string: QString)),
    search_metadata: qt_method!(fn(&mut self, query: QString, by_asin: bool)),
    start_conversion: qt_method!(fn(&mut self)),
    cancel_conversion: qt_method!(fn(&mut self)),
    scan_chapters: qt_method!(fn(&mut self)),
}

impl LecternController {
    fn initialize(&mut self) {
        self.load_config();
    }

    fn load_config(&mut self) {
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            if let Ok(config) = serde_json::from_str::<serde_json::Value>(&content) {
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

    fn save_config(&mut self, url: QString, token: QString, id: QString) {
        self.abs_host = url;
        self.abs_token = token;
        self.abs_library_id = id;
        self.config_changed();

        let config_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Some(parent) = config_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }

        let config = serde_json::json!({
            "abs_host": self.abs_host.to_string(),
            "abs_token": self.abs_token.to_string(),
            "abs_library_id": self.abs_library_id.to_string(),
        });

        if let Ok(json) = serde_json::to_string_pretty(&config) {
            if let Err(e) = std::fs::write(&config_path, json) {
                eprintln!("Failed to save config: {}", e);
            } else {
                println!("⚙️ Settings saved");
            }
        }
    }

    fn set_folder_path(&mut self, url_string: QString) {
        let raw_url = url_string.to_string();

        let path = if let Ok(parsed_url) = Url::parse(&raw_url) {
            if parsed_url.scheme() == "file" {
                parsed_url.to_file_path().unwrap_or_else(|_| PathBuf::from(&raw_url))
            } else {
                PathBuf::from(&raw_url)
            }
        } else {
            PathBuf::from(&raw_url)
        };

        let path_str = path.to_string_lossy().to_string();
        self.current_folder = QString::from(path_str.clone());
        self.folder_changed();

        self.status_message = QString::from(format!("Loaded folder: {}", path_str));
        self.status_changed();

        println!("📂 Folder set to: {:?}", path);
    }

    fn search_metadata(&mut self, query: QString, _by_asin: bool) {
        let qptr = QPointer::from(&*self);
        let query_str = query.to_string();

        self.is_processing = true;
        self.status_message = QString::from(format!("Searching for '{}'...", query_str));
        self.status_changed();
        self.processing_changed();

        let on_complete = queued_callback(move |results: Vec<BookMetadata>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut s = pinned.borrow_mut();
                s.is_processing = false;

                if let Some(book) = results.first() {
                    s.search_title = book.title.clone().into();
                    let author_str = book.authors.join(", ");
                    s.search_author = author_str.into();
                    s.search_cover_url = book.image_url.clone().into();
                    s.status_message = QString::from("Search completed");
                } else {
                    s.status_message = QString::from("No results found");
                }

                s.search_result_changed();
                s.status_changed();
                s.processing_changed();
            }
        });

        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                match AudioService::fetch_metadata(&query_str).await {
                    Ok(metadata) => on_complete(vec![metadata]),
                    Err(_) => on_complete(vec![]),
                }
            });
        });
    }

    fn scan_chapters(&mut self) {
        self.status_message = QString::from("Chapter scanning not yet implemented");
        self.status_changed();
    }

    fn start_conversion(&mut self) {
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
            image_url: self.metadata_cover_url.to_string(),
            asin: "".to_string(),
            duration_minutes: None,
            release_date: None,
        };

        let abs_config = ABSConfig {
            host: self.abs_host.to_string(),
            token: self.abs_token.to_string(),
            library_id: self.abs_library_id.to_string(),
        };

        self.is_processing = true;
        self.status_message = QString::from("Starting audio conversion...");
        self.progress_value = 0.0;
        self.processing_changed();
        self.status_changed();
        self.progress_changed();

        let qptr = QPointer::from(&*self);
        let update_progress = queued_callback(move |update: Result<String, String>| {
            if let Some(pinned) = qptr.as_pinned() {
                let mut controller = pinned.borrow_mut();
                match update {
                    Ok(message) => {
                        controller.status_message = QString::from(message.as_str());
                        controller.is_processing = false;
                        controller.processing_changed();
                        controller.status_changed();
                        
                        if message.contains("completed") {
                            controller.conversion_completed();
                        }
                    }
                    Err(error) => {
                        controller.status_message = QString::from(error.as_str());
                        controller.error_occurred(QString::from("Conversion failed"));
                        controller.status_changed();
                        controller.is_processing = false;
                        controller.processing_changed();
                    }
                }
            }
        });

        std::thread::spawn(move || {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            let result = runtime.block_on(async {
                let input_dir = PathBuf::from(folder);
                if !input_dir.exists() {
                    return Err("Input directory does not exist".to_string());
                }

                let output_path = input_dir
                    .parent()
                    .unwrap_or(&input_dir)
                    .join(format!("{}.m4b", metadata.title.replace(":", "").replace("/", "")));

                AudioService::convert_to_m4b_with_chapters(&input_dir, &output_path.to_string_lossy(), &metadata).await?;
                AudioService::apply_tags(&output_path.to_string_lossy(), &metadata).await?;

                if !abs_config.host.is_empty() && !abs_config.token.is_empty() {
                    match AudioService::upload_and_scan(&output_path.to_string_lossy(), &abs_config).await {
                        Ok(_) => Ok("✓ Conversion and upload completed!".to_string()),
                        Err(e) => {
                            println!("Upload error: {}", e);
                            Ok("✓ Conversion completed, upload failed".to_string())
                        }
                    }
                } else {
                    Ok("✓ Conversion completed!".to_string())
                }
            });

            update_progress(result);
        });
    }

    fn cancel_conversion(&mut self) {
        self.is_processing = false;
        self.processing_changed();
        self.status_message = QString::from("Operation cancelled");
        self.status_changed();
    }
}

fn main() {
    println!("🎵 Starting Lectern...");

    // Set Qt platform if not set
    if std::env::var("QT_QPA_PLATFORM").is_err() {
        std::env::set_var("QT_QPA_PLATFORM", "xcb");
    }

    // Create controller
    let mut controller = LecternController::default();
    controller.initialize();
    let controller = RefCell::new(controller);
    let controller_pinned = unsafe { QObjectPinned::new(&controller) };

    // Create QML engine
    let mut engine = QmlEngine::new();
    engine.set_object_property("controller".into(), controller_pinned);

    // Load QML - try absolute path first
    let qml_path = std::path::Path::new("qml/main.qml");
    if qml_path.exists() {
        println!("Loading QML from: {:?}", qml_path.canonicalize().unwrap_or(qml_path.to_path_buf()));
        engine.load_file("qml/main.qml".into());
    } else {
        eprintln!("❌ ERROR: qml/main.qml not found!");
        eprintln!("Current directory: {:?}", std::env::current_dir());
        std::process::exit(1);
    }

    println!("Starting Qt event loop...");
    engine.exec();
    println!("Qt event loop ended");
}