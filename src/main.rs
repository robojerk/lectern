mod services;
mod models;

use qmetaobject::*;
use cstr::cstr;
use std::sync::mpsc;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

// Wrapper to allow sending raw pointer between threads
// This is safe because we only access it from the main thread via queued_callback
struct SendPtr(*mut LecternController);
unsafe impl Send for SendPtr {}
impl Clone for SendPtr {
    fn clone(&self) -> Self {
        SendPtr(self.0)
    }
}

#[derive(QObject, Default)]
struct LecternController {
    base: qt_base_class!(trait QObject),
    
    // Internal state for search results
    pending_search_results: Arc<Mutex<Option<QVariantList>>>,
    pending_search_error: Arc<Mutex<Option<QString>>>,
    
    // Properties
    is_processing: qt_property!(bool; NOTIFY is_processing_changed),
    status_message: qt_property!(QString; NOTIFY status_message_changed),
    current_folder: qt_property!(QString; NOTIFY current_folder_changed),
    progress_value: qt_property!(f64; NOTIFY progress_value_changed),
    
    abs_host: qt_property!(QString; NOTIFY abs_host_changed),
    abs_token: qt_property!(QString; NOTIFY abs_token_changed),
    abs_library_id: qt_property!(QString; NOTIFY abs_library_id_changed),
    local_library_path: qt_property!(QString; NOTIFY local_library_path_changed),
    path_template: qt_property!(QString; NOTIFY path_template_changed),
    
    // Metadata properties
    book_title: qt_property!(QString; NOTIFY book_title_changed),
    book_author: qt_property!(QString; NOTIFY book_author_changed),
    book_series: qt_property!(QString; NOTIFY book_series_changed),
    book_narrator: qt_property!(QString; NOTIFY book_narrator_changed),
    
    // Signals
    is_processing_changed: qt_signal!(),
    status_message_changed: qt_signal!(),
    current_folder_changed: qt_signal!(),
    progress_value_changed: qt_signal!(),
    abs_host_changed: qt_signal!(),
    abs_token_changed: qt_signal!(),
    abs_library_id_changed: qt_signal!(),
    local_library_path_changed: qt_signal!(),
    path_template_changed: qt_signal!(),
    
    book_title_changed: qt_signal!(),
    book_author_changed: qt_signal!(),
    book_series_changed: qt_signal!(),
    book_narrator_changed: qt_signal!(),
    
    error_occurred: qt_signal!(message: QString),
    log_message: qt_signal!(message: QString),
    metadata_changed: qt_signal!(),
    conversion_completed: qt_signal!(),
    search_results_ready: qt_signal!(results: QVariantList),
    
    // Methods
    set_folder_path: qt_method!(fn(&mut self, path: QString)),
    save_config: qt_method!(fn(&mut self, host: QString, token: QString, library_id: QString, local_path: QString, template: QString)),
    start_conversion: qt_method!(fn(&mut self)),
    search_metadata: qt_method!(fn(&self, query: QString, by_asin: bool)),
    apply_search_result: qt_method!(fn(&mut self, title: QString, author: QString, series: QString, narrator: QString)),
    check_search_results: qt_method!(fn(&mut self) -> QVariantList),
}


impl LecternController {
    
    fn set_folder_path(&mut self, path: QString) {
        let path_str = path.to_string();
        self.current_folder = path.clone();
        self.current_folder_changed();
        
        self.status_message = QString::from("📂 Loaded folder");
        self.status_message_changed();
        
        self.log_message(QString::from(format!("Folder loaded: {}", path_str)));
    }
    
    fn save_config(&mut self, host: QString, token: QString, library_id: QString, local_path: QString, template: QString) {
        self.abs_host = host;
        self.abs_token = token;
        self.abs_library_id = library_id;
        self.local_library_path = local_path;
        self.path_template = template;
        
        self.abs_host_changed();
        self.abs_token_changed();
        self.abs_library_id_changed();
        self.local_library_path_changed();
        self.path_template_changed();
        
        self.log_message(QString::from("Configuration saved"));
        self.status_message = QString::from("✓ Settings saved");
        self.status_message_changed();
    }
    
    fn start_conversion(&mut self) {
        self.is_processing = true;
        self.is_processing_changed();
        
        self.status_message = QString::from("🔄 Starting conversion...");
        self.status_message_changed();
        
        self.progress_value = 0.1;
        self.progress_value_changed();
        
        self.log_message(QString::from("Starting conversion process..."));
    }
    
    fn search_metadata(&self, query: QString, by_asin: bool) {
        let query_str = query.to_string();
        
        println!("🔍 Search requested: '{}' (ASIN: {})", query_str, by_asin);
        
        // Clear any pending results
        {
            let mut pending = self.pending_search_results.lock().unwrap();
            *pending = None;
        }
        {
            let mut pending_err = self.pending_search_error.lock().unwrap();
            *pending_err = None;
        }
        
        // Get references to shared state
        let results_shared = Arc::clone(&self.pending_search_results);
        let error_shared = Arc::clone(&self.pending_search_error);
        
        // Spawn async search in background thread
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            let result = rt.block_on(services::AudioService::search_metadata(&query_str, by_asin));
            
            // Convert results and store in shared state
            match result {
                Ok(results) => {
                    println!("✓ Search completed: {} results", results.len());
                    
                    // Convert results to QVariantList
                    let qml_results: QVariantList = results.iter().map(|book| {
                        let mut map = qmetaobject::QVariantMap::default();
                        
                        map.insert("title".into(), QVariant::from(QString::from(book.title.as_str())));
                        
                        if let Some(cover_url) = &book.cover_url {
                            map.insert("image_url".into(), QVariant::from(QString::from(cover_url.as_str())));
                        } else {
                            map.insert("image_url".into(), QVariant::from(QString::from("")));
                        }
                        
                        if let Some(asin) = &book.asin {
                            map.insert("asin".into(), QVariant::from(QString::from(asin.as_str())));
                        } else {
                            map.insert("asin".into(), QVariant::from(QString::from("")));
                        }
                        
                        let authors_list = QVariantList::from_iter(
                            std::iter::once(QVariant::from(QString::from(book.author.as_str())))
                        );
                        map.insert("authors".into(), QVariant::from(authors_list));
                        
                        if let Some(narrator) = &book.narrator {
                            let narrators_list = QVariantList::from_iter(
                                std::iter::once(QVariant::from(QString::from(narrator.as_str())))
                            );
                            map.insert("narrator_names".into(), QVariant::from(narrators_list));
                        } else {
                            // Always include narrator_names, even if empty
                            map.insert("narrator_names".into(), QVariant::from(QVariantList::default()));
                        }
                        
                        // Always include series_name (empty if not available)
                        map.insert("series_name".into(), QVariant::from(QString::from("")));
                        
                        if let Some(year) = &book.publish_year {
                            map.insert("release_date".into(), QVariant::from(QString::from(year.as_str())));
                        }
                        
                        QVariant::from(map)
                    }).collect();
                    
                    // Store results in shared state
                    {
                        let mut pending = results_shared.lock().unwrap();
                        *pending = Some(qml_results);
                    }
                    
                    // Results are stored, QML will poll via check_search_results
                    println!("[DEBUG] Results stored, ready for polling");
                }
                Err(e) => {
                    println!("❌ Search failed: {}", e);
                    let error_msg = QString::from(format!("Search failed: {}", e));
                    {
                        let mut pending_err = error_shared.lock().unwrap();
                        *pending_err = Some(error_msg.clone());
                    }
                    
                    // Error stored, QML will poll via check_search_results
                    println!("[DEBUG] Error stored, ready for polling");
                }
            }
        });
    }
    
    fn check_search_results(&mut self) -> QVariantList {
        // Check for pending results and return them
        // NOTE: This should only emit once per search, not repeatedly
        let mut pending = self.pending_search_results.lock().unwrap();
        if let Some(results) = pending.take() {
            println!("[DEBUG] check_search_results: Found {} results", results.len());
            println!("[DEBUG] About to emit signal with {} items", results.len());
            
            // Validate the results before emitting
            if results.len() > 0 {
                println!("[DEBUG] Results contain {} items, validating...", results.len());
            }
            
            println!("[DEBUG] Emitting signal now...");
            // Emit signal with results - we're already on Qt thread (qt_method!)
            self.search_results_ready(results.clone());
            println!("[DEBUG] Signal emitted successfully");
            
            println!("[DEBUG] check_search_results: Returning empty list to prevent re-emission");
            // Return empty list so timer doesn't keep re-emitting
            QVariantList::default()
        } else {
            // Check for errors
            let mut pending_err = self.pending_search_error.lock().unwrap();
            if let Some(err) = pending_err.take() {
                println!("[DEBUG] check_search_results: Found error, emitting error signal");
                self.error_occurred(err);
            }
            QVariantList::default()
        }
    }
    
    fn apply_search_result(&mut self, title: QString, author: QString, series: QString, narrator: QString) {
        // #region agent log
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/home/rob/Documents/Projects/lectern/.cursor/debug.log") {
            use std::io::Write;
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"A","location":"main.rs:229","message":"apply_search_result called","data":{{"title":"{}","author":"{}","series":"{}","narrator":"{}"}},"timestamp":{}}}"#, 
                title.to_string().replace("\"", "\\\""), author.to_string().replace("\"", "\\\""), 
                series.to_string().replace("\"", "\\\""), narrator.to_string().replace("\"", "\\\""),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        }
        // #endregion
        
        println!("[DEBUG] Applying search result");
        println!("[DEBUG] Received - title: '{}', author: '{}', series: '{}', narrator: '{}'", 
                 title.to_string(), author.to_string(), series.to_string(), narrator.to_string());
        
        // #region agent log
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/home/rob/Documents/Projects/lectern/.cursor/debug.log") {
            use std::io::Write;
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"B","location":"main.rs:235","message":"Before setting properties","data":{{"current_title":"{}","current_author":"{}"}},"timestamp":{}}}"#, 
                self.book_title.to_string().replace("\"", "\\\""), self.book_author.to_string().replace("\"", "\\\""),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        }
        // #endregion
        
        // Set title (always set, even if empty, to clear previous values)
        self.book_title = title.clone();
        self.book_title_changed();
        
        // #region agent log
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/home/rob/Documents/Projects/lectern/.cursor/debug.log") {
            use std::io::Write;
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"C","location":"main.rs:242","message":"After setting title and emitting signal","data":{{"book_title":"{}"}},"timestamp":{}}}"#, 
                self.book_title.to_string().replace("\"", "\\\""),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        }
        // #endregion
        
        println!("[DEBUG] Set title property to: '{}'", self.book_title.to_string());
        
        // Set author
        self.book_author = author.clone();
        self.book_author_changed();
        println!("[DEBUG] Set author property to: '{}'", self.book_author.to_string());
        
        // Set series
        self.book_series = series.clone();
        self.book_series_changed();
        println!("[DEBUG] Set series property to: '{}'", self.book_series.to_string());
        
        // Set narrator
        self.book_narrator = narrator.clone();
        self.book_narrator_changed();
        println!("[DEBUG] Set narrator property to: '{}'", self.book_narrator.to_string());
        
        // #region agent log
        if let Ok(mut file) = std::fs::OpenOptions::new().create(true).append(true).open("/home/rob/Documents/Projects/lectern/.cursor/debug.log") {
            use std::io::Write;
            let _ = writeln!(file, r#"{{"sessionId":"debug-session","runId":"run1","hypothesisId":"D","location":"main.rs:260","message":"All properties set and signals emitted","data":{{"final_title":"{}","final_author":"{}","final_series":"{}","final_narrator":"{}"}},"timestamp":{}}}"#, 
                self.book_title.to_string().replace("\"", "\\\""), self.book_author.to_string().replace("\"", "\\\""),
                self.book_series.to_string().replace("\"", "\\\""), self.book_narrator.to_string().replace("\"", "\\\""),
                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis());
        }
        // #endregion
        
        self.log_message(QString::from("Metadata applied from search result"));
        self.status_message = QString::from("✓ Metadata applied");
        self.status_message_changed();
        self.metadata_changed();
    }
}

fn main() {
    println!("🎵 Starting Lectern...");
    
    qml_register_type::<LecternController>(cstr!("Lectern"), 1, 0, cstr!("LecternController"));
    
    let mut engine = QmlEngine::new();
    let mut controller = LecternController::default();
    // Initialize Arc<Mutex> fields that can't be in Default
    controller.pending_search_results = Arc::new(Mutex::new(None));
    controller.pending_search_error = Arc::new(Mutex::new(None));
    let controller = QObjectBox::new(controller);
    
    engine.set_object_property("controller".into(), controller.pinned());
    
    let qml_path = std::env::current_dir()
        .unwrap()
        .join("qml")
        .join("main.qml");
    
    println!("Loading QML from: {:?}", qml_path);
    
    if !qml_path.exists() {
        eprintln!("❌ Error: main.qml not found at {:?}", qml_path);
        std::process::exit(1);
    }
    
    engine.load_file(qml_path.to_string_lossy().to_string().into());
    
    println!("Starting Qt event loop...");
    engine.exec();
}