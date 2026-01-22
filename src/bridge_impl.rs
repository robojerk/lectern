use crate::services::AudioService;
use cxx_qt_lib::QString;
use std::pin::Pin;
use tokio::runtime::Runtime;

impl Default for LecternController {
    fn default() -> Self {
        Self {
            current_folder: QString::from(""),
            status_message: QString::from("Ready to process audiobooks"),
            progress_value: 0.0,
            is_processing: false,
            metadata_title: QString::from(""),
            metadata_author: QString::from(""),
            metadata_series: QString::from(""),
            metadata_narrator: QString::from(""),
            abs_host: QString::from(""),
            abs_token: QString::from(""),
            abs_library_id: QString::from(""),
        }
    }
}

impl LecternController {
    pub fn load_config(self: Pin<&mut Self>) {
        // Load ABS configuration from disk
        let config_path = dirs::config_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("."))
            .join("lectern")
            .join("config.json");

        if let Ok(content) = std::fs::read_to_string(config_path) {
            // For now, just log that we would load config
            // TODO: Implement proper config loading
            println!("Would load config: {}", content.len());
        }
    }

    pub fn save_config(self: Pin<&mut Self>) {
        // TODO: Implement config saving
        println!("Would save config: {} / {} / {}",
            self.abs_host().to_string(),
            self.abs_token().to_string(),
            self.abs_library_id().to_string());
    }

    pub fn set_folder_path(self: Pin<&mut Self>, path: QString) {
        unsafe {
            self.as_mut().set_current_folder(path.clone());
        }

        // Auto-fetch metadata for the folder
        let path_str = path.to_string();
        self.auto_fetch_metadata(path_str);
    }

    pub fn search_metadata(self: Pin<&mut Self>, query: QString, by_asin: bool) {
        let query_str = query.to_string();
        unsafe {
            self.as_mut().set_is_processing(true);
            self.as_mut().set_status_message(QString::from(&format!("Searching for '{}'...", query_str)));
        }

        // Perform search in background
        let controller_ptr = unsafe { self.get_unchecked_mut() };
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match AudioService::search_metadata(&query_str, by_asin).await {
                    Ok(results) => {
                        // TODO: Update search results model
                        controller_ptr.set_status_message(QString::from(&format!("Found {} results", results.len())));
                        controller_ptr.search_results_updated();
                    }
                    Err(e) => {
                        controller_ptr.set_status_message(QString::from(&format!("Search failed: {}", e)));
                        controller_ptr.error_occurred(QString::from(&e.to_string()));
                    }
                }
                controller_ptr.set_is_processing(false);
            });
        });
    }

    pub fn use_search_result(self: Pin<&mut Self>, index: usize) {
        // TODO: Get result from search results model and populate metadata fields
        unsafe {
            // For now, just emit metadata loaded signal
            self.as_mut().metadata_loaded();
        }
    }

    pub fn start_conversion(self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().set_is_processing(true);
            self.as_mut().set_progress_value(0.0);
            self.as_mut().set_status_message(QString::from("Starting conversion..."));
        }

        let folder_path = self.current_folder().to_string();
        let title = self.metadata_title().to_string();
        let author = self.metadata_author().to_string();
        let series = self.metadata_series().to_string();
        let narrator = self.metadata_narrator().to_string();

        let abs_config = Some(crate::ABSConfig {
            host: self.abs_host().to_string(),
            token: self.abs_token().to_string(),
            library_id: self.abs_library_id().to_string(),
        });

        let controller_ptr = unsafe { self.get_unchecked_mut() };

        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                // TODO: Implement full conversion pipeline
                controller_ptr.log_message(QString::from("Conversion started..."));
                tokio::time::sleep(std::time::Duration::from_secs(2)).await;
                controller_ptr.set_progress_value(1.0);
                controller_ptr.set_status_message(QString::from("Conversion completed!"));
                controller_ptr.set_is_processing(false);
                controller_ptr.conversion_completed();
            });
        });
    }

    pub fn cancel_conversion(self: Pin<&mut Self>) {
        unsafe {
            self.as_mut().set_is_processing(false);
            self.as_mut().set_status_message(QString::from("Conversion cancelled"));
        }
    }

    pub fn add_chapter(self: Pin<&mut Self>, title: QString, start_time: f64) {
        // TODO: Add chapter to chapters model
        self.log_message(QString::from(&format!("Added chapter: {}", title.to_string())));
    }

    pub fn remove_chapter(self: Pin<&mut Self>, index: usize) {
        // TODO: Remove chapter from chapters model
        self.log_message(QString::from(&format!("Removed chapter at index {}", index)));
    }

    pub fn update_chapter(self: Pin<&mut Self>, index: usize, title: QString, start_time: f64) {
        // TODO: Update chapter in chapters model
        self.log_message(QString::from(&format!("Updated chapter {}: {}", index, title.to_string())));
    }

    pub fn lock_chapter(self: Pin<&mut Self>, index: usize, locked: bool) {
        // TODO: Lock/unlock chapter
        let action = if locked { "locked" } else { "unlocked" };
        self.log_message(QString::from(&format!("Chapter {} {}", index, action)));
    }

    pub fn shift_chapters(self: Pin<&mut Self>, offset: f64) {
        // TODO: Shift all unlocked chapters by offset
        self.log_message(QString::from(&format!("Shifted chapters by {:.2} seconds", offset)));
    }

    pub fn play_chapter(self: Pin<&mut Self>, index: usize) {
        // TODO: Play chapter audio for preview
        self.log_message(QString::from(&format!("Playing chapter {}", index)));
    }

    pub fn pause_playback(self: Pin<&mut Self>) {
        // TODO: Pause audio playback
        self.log_message(QString::from("Playback paused"));
    }

    pub fn stop_playback(self: Pin<&mut Self>) {
        // TODO: Stop audio playback
        self.log_message(QString::from("Playback stopped"));
    }

    // Helper methods
    fn auto_fetch_metadata(&self, path_str: String) {
        let folder_name = std::path::Path::new(&path_str)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        unsafe {
            self.set_status_message(QString::from(&format!("Loading: {}", folder_name)));
        }

        let controller_ptr = unsafe { self.get_unchecked_mut() };
        std::thread::spawn(move || {
            let rt = Runtime::new().unwrap();
            rt.block_on(async {
                match AudioService::fetch_metadata(&folder_name).await {
                    Ok(metadata) => {
                        controller_ptr.set_metadata_title(QString::from(&metadata.title));
                        controller_ptr.set_metadata_author(QString::from(&metadata.authors.join(", ")));

                        if let Some(series) = &metadata.series_name {
                            controller_ptr.set_metadata_series(QString::from(series));
                        }

                        if let Some(narrators) = &metadata.narrator_names {
                            controller_ptr.set_metadata_narrator(QString::from(&narrators.join(", ")));
                        }

                        controller_ptr.metadata_loaded();
                        controller_ptr.log_message(QString::from(&format!("✓ Loaded metadata: {}", metadata.title)));
                    }
                    Err(e) => {
                        controller_ptr.log_message(QString::from(&format!("⚠️  Metadata fetch failed: {}", e)));
                        // Still emit metadata loaded to enable manual entry
                        controller_ptr.metadata_loaded();
                    }
                }
            });
        });
    }
}