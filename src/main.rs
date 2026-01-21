mod services;
mod models;
mod ui;
mod app_event;
mod player;

use gtk4::prelude::*;
use gtk4::{Application, gio, gdk, DropTarget, ResponseType};
use glib::{MainContext, Priority};
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
// use tokio::runtime::Runtime;

use models::settings::{AppState, load_config, save_config};
use models::project::Project;
use models::metadata::BookMetadata;
use models::chapter::ChapterObject;
use ui::window::LecternWindow;
use ui::dialogs::show_config_dialog;
use services::{AudioService, AudioMap};
use app_event::AppEvent;
use player::LecternPlayer;

fn main() {
    let app = Application::builder()
        .application_id("com.librarian.rs")
        .build();

    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    // Apply CSS
    let provider = gtk4::CssProvider::new();
    provider.load_from_data(include_str!("ui/style.css"));
    gtk4::style_context_add_provider_for_display(
        &gdk::Display::default().expect("Could not connect to a display."),
        &provider,
        gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
    );

    let (tx, rx) = MainContext::channel::<AppEvent>(Priority::default());
    let player = Arc::new(LecternPlayer::new());
    
    // Load config
    let initial_config = load_config();
    let state = Arc::new(Mutex::new(AppState {
        folder_path: None,
        config: initial_config,
        project: None,
        search_results: Vec::new(),
        original_cover_bytes: None,
    }));

    let window = LecternWindow::new(app, tx.clone());

    // Settings Button
    let state_clone = state.clone();
    let window_weak = window.window.downgrade();
    window.settings_btn.connect_clicked(move |_| {
        if let Some(win) = window_weak.upgrade() {
            let current_config = state_clone.lock().unwrap().config.clone();
            if let Some(config) = show_config_dialog(&win, current_config.as_ref()) {
                save_config(&config);
                state_clone.lock().unwrap().config = Some(config);
            }
        }
    });

    // Search Metadata Button (in Metadata Tab) -> Switches to Match Tag

    let editor_page_weak = window.editor_page.clone();
    window.editor_page.search_btn.connect_clicked(move |_| {
        let current_title = editor_page_weak.title_entry.text().to_string();
        editor_page_weak.match_search_entry.set_text(&current_title);
        editor_page_weak.notebook.set_current_page(Some(3)); // Match tab
    });

    let editor_page_weak = window.editor_page.clone();
    window.editor_page.title_search_btn.connect_clicked(move |_| {
        let current_title = editor_page_weak.title_entry.text().to_string();
        if !current_title.is_empty() {
            editor_page_weak.match_search_entry.set_text(&current_title);
            editor_page_weak.notebook.set_current_page(Some(3));
            editor_page_weak.match_search_btn.emit_clicked();
        }
    });

    let editor_page_weak = window.editor_page.clone();
    window.editor_page.asin_lookup_btn.connect_clicked(move |_| {
        let asin = editor_page_weak.asin_entry.text().to_string();
        if !asin.is_empty() {
            editor_page_weak.match_search_entry.set_text(&asin);
            editor_page_weak.notebook.set_current_page(Some(3));
            editor_page_weak.match_search_btn.emit_clicked();
        }
    });

    // Back Button
    let stack_clone = window.stack.clone();
    window.editor_page.back_btn.connect_clicked(move |_| {
        stack_clone.set_visible_child_name("drop_zone");
    });

    // Match Search Button
    let editor_page_weak = window.editor_page.clone();
    let tx_clone = tx.clone();
    window.editor_page.match_search_btn.connect_clicked(move |_| {
        let query = editor_page_weak.match_search_entry.text().to_string();
        if query.is_empty() { return; }
        
        let tx_inner = tx_clone.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                  let is_asin = query.len() == 10 && query.chars().all(|c| c.is_alphanumeric());
                  // Also support Audible URLs by extracting ASIN
                  let final_query = if query.contains("audible.") && query.contains("/pd/") {
                      query.split("/pd/").nth(1)
                          .and_then(|s| s.split('/').next())
                          .and_then(|s| s.split('?').next())
                          .unwrap_or(&query)
                          .to_string()
                  } else {
                      query.clone()
                  };
                  let is_asin = is_asin || (final_query.len() == 10 && final_query.chars().all(|c| c.is_alphanumeric()));

                  let _ = tx_inner.send(AppEvent::Status(format!("🔄 Searching for '{}'...", final_query)));
                  match AudioService::search_metadata(&final_query, is_asin).await {
                      Ok(results) => {
                          let _ = tx_inner.send(AppEvent::SearchResultsLoaded(results));
                      }
                      Err(e) => {
                          let _ = tx_inner.send(AppEvent::Error(format!("❌ Search failed: {}", e)));
                      }
                  }
            });
        });
    });

    // Restore Original Cover Button
    let editor_page_weak = window.editor_page.clone();
    let state_clone = state.clone();
    window.editor_page.restore_cover_btn.connect_clicked(move |_| {
        if let Some(bytes) = &state_clone.lock().unwrap().original_cover_bytes {
            editor_page_weak.load_cover_image(bytes);
        }
    });

    // Titles from Files Button
    let editor_page_weak = window.editor_page.clone();
    let state_clone = state.clone();
    window.editor_page.titles_from_files_btn.connect_clicked(move |_| {
        let state_guard = state_clone.lock().unwrap();
        if let Some(project) = &state_guard.project {
            let store = &editor_page_weak.chapter_store;
            let n_items = store.n_items();
            let n_files = project.files.len();
            
            if n_items == n_files as u32 {
                for i in 0..n_items {
                    if let Some(item) = store.item(i).and_downcast::<ChapterObject>() {
                        if !item.is_locked() {
                            if let Some(file) = project.files.get(i as usize) {
                                let title = file.file_stem().unwrap_or_default().to_string_lossy().to_string();
                                item.set_title(title);
                            }
                        }
                    }
                }
            }
        }
    });

    // Fetch Chapters from Audible Button
    let editor_page_weak = window.editor_page.clone();
    let tx_clone = tx.clone();
    window.editor_page.fetch_chapters_btn.connect_clicked(move |_| {
        let asin = editor_page_weak.asin_entry.text().to_string();
        if asin.is_empty() {
             let _ = tx_clone.send(AppEvent::Error("ASIN is required for Audible chapter lookup.".to_string()));
             return;
        }

        let locale_idx = editor_page_weak.audible_locale_combo.selected();
        let locales = ["us", "ca", "uk", "au", "de", "fr", "it", "es", "jp", "in"];
        let locale = locales.get(locale_idx as usize).unwrap_or(&"us").to_string();

        let tx_inner = tx_clone.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                 let _ = tx_inner.send(AppEvent::Status(format!("🔄 Fetching chapters for ASIN {}...", asin)));
                 match AudioService::fetch_chapters_from_audible(&asin, &locale).await {
                     Ok(chapters) => {
                         let _ = tx_inner.send(AppEvent::ChaptersLoaded(chapters));
                         let _ = tx_inner.send(AppEvent::Status("✓ Chapters updated from Audible".to_string()));
                     }
                     Err(e) => {
                         let _ = tx_inner.send(AppEvent::Error(format!("❌ Failed to fetch chapters: {}", e)));
                     }
                 }
            });
        });
    });

    // Fetch Chapters from Audnexus Button
    let editor_page_weak = window.editor_page.clone();
    let tx_clone = tx.clone();
    window.editor_page.fetch_audnexus_btn.connect_clicked(move |_| {
        let asin = editor_page_weak.asin_entry.text().to_string();
        if asin.is_empty() {
             let _ = tx_clone.send(AppEvent::Error("ASIN is required for Audnexus chapter lookup.".to_string()));
             return;
        }

        let locale_idx = editor_page_weak.audible_locale_combo.selected();
        let locales = ["us", "ca", "uk", "au", "de", "fr", "it", "es", "jp", "in"];
        let locale = locales.get(locale_idx as usize).unwrap_or(&"us").to_string();

        let tx_inner = tx_clone.clone();
        std::thread::spawn(move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async {
                 let _ = tx_inner.send(AppEvent::Status(format!("🔄 Fetching chapters from Audnexus for ASIN {} ({})...", asin, locale)));
                 match AudioService::fetch_chapters_from_audnexus(&asin, &locale).await {
                     Ok(chapters) => {
                         let _ = tx_inner.send(AppEvent::ChaptersLoaded(chapters));
                         let _ = tx_inner.send(AppEvent::Status("✓ Chapters updated from Audnexus".to_string()));
                     }
                     Err(e) => {
                         let _ = tx_inner.send(AppEvent::Error(format!("❌ Audnexus fetch failed: {}", e)));
                     }
                 }
            });
        });
    });

    // Match Results List Selection
    let state_clone = state.clone();
    let editor_page_weak = window.editor_page.clone();
    let tx_clone = tx.clone();
    window.editor_page.match_results_list.connect_row_activated(move |_, row| {
        let index = row.index() as usize;
        let mut state_guard = state_clone.lock().unwrap();
        
        if let Some(metadata) = state_guard.search_results.get(index).cloned() {
            println!("✅ Selected result {}: {}", index, metadata.title);
            
            // Update Metadata Tab UI
            editor_page_weak.set_metadata(&metadata);
            
            // Update Project State
            if let Some(project) = &mut state_guard.project {
                project.metadata = metadata.clone();
            }
            
            // Fetch Cover
            if let Some(cover_url) = &metadata.cover_url {
                let cover_url = cover_url.clone();
                let tx_inner = tx_clone.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                        let _ = tx_inner.send(AppEvent::Status("🖼️ Fetching cover art...".to_string()));
                        match AudioService::fetch_image(&cover_url).await {
                            Ok(bytes) => {
                                let _ = tx_inner.send(AppEvent::CoverImageLoaded(bytes));
                                let _ = tx_inner.send(AppEvent::Status("✓ Metadata and cover applied".to_string()));
                            }
                            Err(e) => {
                                let _ = tx_inner.send(AppEvent::Error(format!("Cover fetch failed: {}", e)));
                            }
                        }
                    });
                });
            } else {
                 let _ = tx_clone.send(AppEvent::Status("✓ Metadata applied".to_string()));
            }
            
            // Switch back to Metadata tab after selection
            editor_page_weak.notebook.set_current_page(Some(0));
        } else {
            println!("⚠️ Selection failed: No metadata at index {}", index);
        }
    });

    // Change Cover Button (Local File)
    let window_weak = window.window.downgrade();
    let tx_clone = tx.clone();
    window.editor_page.change_cover_btn.connect_clicked(move |_| {
        if let Some(win) = window_weak.upgrade() {
            let dialog = gtk4::FileChooserDialog::new(
                Some("Select Cover Image"),
                Some(&win),
                gtk4::FileChooserAction::Open,
                &[("Cancel", gtk4::ResponseType::Cancel), ("Open", gtk4::ResponseType::Accept)],
            );
            
            let filter = gtk4::FileFilter::new();
            filter.add_mime_type("image/*");
            filter.set_name(Some("Images"));
            dialog.add_filter(&filter);

            let tx_inner = tx_clone.clone();
            dialog.connect_response(move |d, resp| {
                if resp == gtk4::ResponseType::Accept {
                    if let Some(file) = d.file() {
                        if let Some(path) = file.path() {
                            if let Ok(bytes) = std::fs::read(path) {
                                let _ = tx_inner.send(AppEvent::CoverImageLoaded(bytes));
                            }
                        }
                    }
                }
                d.close();
            });
            dialog.show();
        }
    });

    // Convert Button
    let state_clone = state.clone();
    let tx_clone = tx.clone();
    let editor_page_weak = window.editor_page.clone();
    let window_handle = window.window.clone();
    
    window.editor_page.convert_btn.connect_clicked(move |btn| {
        btn.set_sensitive(false);
        let tx_inner = tx_clone.clone();
        
        let state_guard = state_clone.lock().unwrap();
        let project = state_guard.project.clone();
        let config = state_guard.config.clone();
        drop(state_guard);
        
        if let Some(mut proj) = project {
             // Update metadata from UI one last time
             let ui_metadata = editor_page_weak.get_metadata();
             proj.metadata = ui_metadata;
             
             // Extract chapter data from the editor (can't move GObjects to thread)
             let chapter_store = &editor_page_weak.chapter_store;
             let n_items = chapter_store.n_items();
             let mut chapter_data = Vec::new();
             for i in 0..n_items {
                 if let Some(item) = chapter_store.item(i).and_downcast::<ChapterObject>() {
                     chapter_data.push(services::SimpleChapter {
                         title: item.title(),
                         start_ms: item.start_time(),
                         duration_ms: item.duration(),
                     });
                 }
             }

             let folder = proj.files.first().and_then(|p| p.parent()).map(|p| p.to_path_buf());
             
             if let Some(_) = folder {
                 let mut output_path = None;
                 
                 if let Some(cfg) = &config {
                     output_path = AudioService::resolve_output_path(cfg, &proj.metadata);
                 }
                 
                 if output_path.is_none() {
                     // Force user to choose if no local library set
                     let dialog = gtk4::FileChooserDialog::new(
                         Some("Save Audiobook As"),
                         Some(&window_handle),
                         gtk4::FileChooserAction::Save,
                         &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Accept)],
                     );
                     dialog.set_current_name(&format!("{}.m4b", proj.metadata.title.replace("/", "-")));
                     
                     let response = std::rc::Rc::new(std::cell::Cell::new(None));
                     let response_clone = response.clone();
                     dialog.connect_response(move |_, resp| {
                         response_clone.set(Some(resp));
                     });
                     
                     dialog.show();
                     while response.get().is_none() {
                         glib::MainContext::default().iteration(true);
                     }
                     
                     if response.get() == Some(ResponseType::Accept) {
                         output_path = dialog.file().and_then(|f| f.path());
                     }
                     dialog.close();
                 }

                 if let Some(out_path) = output_path {
                     let output_path_str = out_path.to_string_lossy().to_string();
                     
                     // Ensure parent directories exist
                     if let Some(parent) = out_path.parent() {
                         let _ = std::fs::create_dir_all(parent);
                     }

                     // Spawn background task
                     let rt = tokio::runtime::Runtime::new().unwrap();
                     std::thread::spawn(move || {
                         rt.block_on(async {
                         let _ = tx_inner.send(AppEvent::Status("🚀 Starting conversion...".to_string()));
                         
                         // Build chapter metadata from extracted data
                         let mut metadata_content = String::from(";FFMETADATA1\n");
                         for ch in &chapter_data {
                             let end_ms = ch.start_ms + ch.duration_ms;
                             metadata_content.push_str("[CHAPTER]\n");
                             metadata_content.push_str("TIMEBASE=1/1000\n");
                             metadata_content.push_str(&format!("START={}\n", ch.start_ms));
                             metadata_content.push_str(&format!("END={}\n", end_ms));
                             metadata_content.push_str(&format!("title={}\n", ch.title));
                         }
                         
                         let metadata_file = "/tmp/ffmetadata.txt";
                         
                         if let Err(e) = tokio::fs::write(metadata_file, &metadata_content).await {
                             let _ = tx_inner.send(AppEvent::Error(format!("Failed to write metadata: {}", e)));
                             return;
                         }
                         
                         // Create concat file list
                         let concat_file = "/tmp/concat_list.txt";
                         let concat_content = proj.files
                             .iter()
                             .map(|p| format!("file '{}'", p.display()))
                             .collect::<Vec<_>>()
                             .join("\n");
                         
                         if let Err(e) = tokio::fs::write(concat_file, concat_content).await {
                             let _ = tx_inner.send(AppEvent::Error(format!("Failed to write concat file: {}", e)));
                             return;
                         }
                         
                         // Run FFmpeg
                         let _ = tx_inner.send(AppEvent::Status("🎬 Running FFmpeg conversion...".to_string()));
                         
                         let ffmpeg_result = AudioService::run_ffmpeg_with_logs(
                             vec![
                                 "-f".to_string(),
                                 "concat".to_string(),
                                 "-safe".to_string(),
                                 "0".to_string(),
                                 "-i".to_string(),
                                 concat_file.to_string(),
                                 "-i".to_string(),
                                 metadata_file.to_string(),
                                 "-map_metadata".to_string(),
                                 "1".to_string(),
                                 "-c:a".to_string(),
                                 "aac".to_string(),
                                 "-b:a".to_string(),
                                 "128k".to_string(),
                                 "-f".to_string(),
                                 "mp4".to_string(),
                                 output_path_str.clone(),
                             ],
                             tx_inner.clone(),
                         ).await;
                         
                         // Cleanup temp files
                         let _ = tokio::fs::remove_file(metadata_file).await;
                         let _ = tokio::fs::remove_file(concat_file).await;
                         
                         match ffmpeg_result {
                             Ok(_) => {
                                 let _ = tx_inner.send(AppEvent::Status("✓ Conversion successful".to_string()));
                                 
                                 // 2. Tag
                                 let _ = tx_inner.send(AppEvent::Status("🏷️ Tagging file...".to_string()));
                                 if let Err(e) = AudioService::apply_tags(&output_path_str, &proj.metadata).await {
                                     let _ = tx_inner.send(AppEvent::Error(format!("Tagging failed: {}", e)));
                                 }
                                 
                                 // 3. Upload (if configured)
                                 if let Some(abs_cfg) = config {
                                     let _ = tx_inner.send(AppEvent::Status("☁️ Uploading to Audiobookshelf...".to_string()));
                                     if let Err(e) = AudioService::upload_and_scan(&output_path_str, &abs_cfg).await {
                                         let _ = tx_inner.send(AppEvent::Error(format!("Upload failed: {}", e)));
                                     } else {
                                         let _ = tx_inner.send(AppEvent::Status("🎉 Upload Complete!".to_string()));
                                     }
                                 } else {
                                     let _ = tx_inner.send(AppEvent::Status("✅ Done! (Upload skipped)".to_string()));
                                 }
                             }
                             Err(e) => {
                                 let _ = tx_inner.send(AppEvent::Error(format!("Conversion failed: {}", e)));
                             }
                         }
                         
                         // Signal completion
                         let _ = tx_inner.send(AppEvent::Complete);
                     });
                 });
                 } else {
                     btn.set_sensitive(true);
                 }
             } else {
                 btn.set_sensitive(true);
             }
         } else {
             btn.set_sensitive(true);
         }
    });



    // Global Shift Button
    let editor_page_weak = window.editor_page.clone();
    window.editor_page.apply_shift_btn.connect_clicked(move |_| {
        let text = editor_page_weak.global_shift_entry.text();
        if let Ok(offset_ms) = text.parse::<i64>() {
            let store = &editor_page_weak.chapter_store;
            let n_items = store.n_items();
            
            for i in 0..n_items {
                if let Some(item) = store.item(i).and_downcast::<ChapterObject>() {
                    if !item.is_locked() {
                        let current = item.start_time() as i64;
                        let new_time = (current + offset_ms).max(0) as u64;
                        item.set_start_time(new_time);
                        // Changing the property should automatically update UI if bind is correct
                    }
                }
            }
        }
    });

    // Drop Target (attached to stack)
    let drop_target = DropTarget::builder()
        .actions(gdk::DragAction::COPY)
        .build();
    drop_target.set_types(&[gio::File::static_type(), gdk::FileList::static_type()]);
    
    let tx_clone = tx.clone();
    drop_target.connect_drop(move |_, value, _, _| {
        println!("💧 Drop received, value type: {}", value.type_());
        
        let mut target_path = None;

        // 1. Try single GFile
        if let Ok(file) = value.get::<gio::File>() {
            let path = file.path();
            let uri = file.uri();
            println!("  -> as GFile, path: {:?}, URI: {}", path, uri);
            
            if let Some(p) = path {
                target_path = Some(p);
            } else if uri.starts_with("file://") {
                let decoded_uri = urlencoding::decode(&uri).map(|c| c.into_owned()).unwrap_or_else(|_| uri.to_string());
                // Handle file://, file:///, etc.
                let clean_path = decoded_uri.strip_prefix("file://").unwrap_or(&decoded_uri);
                target_path = Some(PathBuf::from(clean_path));
            } else {
                // Remote protocol detection
                let protocol = uri.split("://").next().unwrap_or("unknown");
                println!("  -> Detected remote protocol: {}", protocol);
                let _ = tx_clone.send(AppEvent::Error(format!(
                    "Remote folders ({}) are not supported directly. Please use a local folder or a mounted share.", 
                    protocol
                )));
            }
        } 
        
        // 2. Try FileList fallback
        if target_path.is_none() {
            if let Ok(file_list) = value.get::<gdk::FileList>() {
                let files = file_list.files();
                println!("  -> as FileList ({} items)", files.len());
                if let Some(file) = files.first() {
                    target_path = file.path();
                }
            }
        }

        if let Some(p) = target_path {
            if p.is_dir() {
                println!("✅ Directory accepted: {:?}", p);
                let _ = tx_clone.send(AppEvent::FolderLoaded(p));
                return true;
            } else if p.extension().map(|e| e.to_string_lossy().to_lowercase() == "m4b").unwrap_or(false) {
                println!("✅ M4B file accepted: {:?}", p);
                let _ = tx_clone.send(AppEvent::FolderLoaded(p));
                return true;
            } else {
                println!("  -> resolved path is not a directory or m4b: {:?}", p);
                let _ = tx_clone.send(AppEvent::Error("Please drop a folder or an .m4b file.".to_string()));
            }
        }
        
        println!("❌ Drop not handled.");
        false
    });
    window.stack.add_controller(drop_target);

    // Event Loop
    let state_clone = state.clone();
    let stack = window.stack.clone();
    let editor_page = window.editor_page.clone();
    
    stack.set_visible_child_name("drop_zone");
    
    rx.attach(None, move |event| {
        match event {
            AppEvent::FolderLoaded(path) => {
                println!("📂 Item loaded: {:?}", path);
                
                let (files, metadata_path) = if path.is_dir() {
                    (AudioService::get_sorted_mp3_files(&path).unwrap_or_default(), path.clone())
                } else {
                    (vec![path.clone()], path.parent().unwrap().to_path_buf())
                };

                let folder_name = path.file_stem().unwrap_or_default().to_string_lossy().to_string();
                
                // 1. Detect local metadata or default
                let metadata = AudioService::find_local_metadata(&metadata_path).unwrap_or_else(|| {
                    BookMetadata {
                        title: folder_name.clone(),
                        authors: vec!["Unknown".to_string()],
                        ..Default::default()
                    }
                });

                // 2. Refresh state
                let project = Project {
                    files: files.clone(),
                    metadata: metadata.clone(),
                    chapters: vec![],
                };
                
                state_clone.lock().unwrap().project = Some(project.clone());
                state_clone.lock().unwrap().folder_path = Some(path.clone());
                
                // 3. Update UI
                editor_page.set_metadata(&metadata);
                editor_page.chapter_store.remove_all();

                let dir_name = path.file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("Unknown Item");
                editor_page.folder_path_label.set_text(dir_name);
                
                // 4. Try load local cover
                if let Some(cover_path) = AudioService::find_local_cover(&metadata_path) {
                    if let Ok(bytes) = std::fs::read(&cover_path) {
                        editor_page.load_cover_image(&bytes);
                        state_clone.lock().unwrap().original_cover_bytes = Some(bytes);
                    }
                }
                
                stack.set_visible_child_name("editor");
                
                // 5. Spawn background task to fetch chapters
                let tx_inner = tx.clone(); 
                let path_clone = path.clone();
                let files_clone = files.clone();
                std::thread::spawn(move || {
                    let rt = tokio::runtime::Runtime::new().unwrap();
                    rt.block_on(async {
                         // Fetch file durations for the audio map
                         let mut durations = Vec::new();
                         for file in &files_clone {
                             durations.push(AudioService::get_duration(file).await.unwrap_or(0));
                         }
                         let _ = tx_inner.send(AppEvent::FileDurationsLoaded(durations));

                         let _ = tx_inner.send(AppEvent::Status("⏱️ Analyzing chapters...".to_string()));
                         let fetch_result = if path_clone.is_dir() {
                             AudioService::get_chapters(&files_clone).await
                         } else {
                             AudioService::get_chapters_from_m4b(&path_clone).await
                         };

                         match fetch_result {
                             Ok(chapters) => {
                                 let _ = tx_inner.send(AppEvent::ChaptersLoaded(chapters));
                                 let _ = tx_inner.send(AppEvent::Status("✓ Chapters loaded".to_string()));
                             }
                             Err(e) => {
                                 let _ = tx_inner.send(AppEvent::Error(format!("Failed to load chapters: {}", e)));
                             }
                         }
                    });
                });
            }
            AppEvent::FileDurationsLoaded(durations) => {
                if let Ok(mut state) = state_clone.lock() {
                    if let Some(proj) = &mut state.project {
                        proj.file_durations = durations;
                    }
                }
            }
            AppEvent::PlayRequested(start_ms) => {
                let state_guard = state_clone.lock().unwrap();
                if let Some(project) = &state_guard.project {
                    let map = AudioMap {
                        files: project.files.clone(),
                        durations: project.file_durations.clone(),
                    };
                    if let Some((path, offset)) = map.resolve_timestamp(start_ms) {
                        player.play_at(path.to_str().unwrap_or(""), offset);
                    } else {
                        let _ = tx.send(AppEvent::Error("Could not resolve timestamp to a file location.".to_string()));
                    }
                }
            }
            AppEvent::StopRequested => {
                player.stop();
            }
            AppEvent::ChaptersLoaded(chapters) => {
                editor_page.chapter_store.remove_all();
                let mut chapter_objects = Vec::new();
                for ch in chapters {
                    let obj = ChapterObject::new(ch.title, ch.start_ms, ch.duration_ms);
                    editor_page.chapter_store.append(&obj);
                    chapter_objects.push(obj);
                }
                
                if let Ok(mut state) = state_clone.lock() {
                    if let Some(proj) = &mut state.project {
                        proj.chapters = chapter_objects;
                    }
                }
            }
            AppEvent::CoverImageLoaded(bytes) => {
                editor_page.load_cover_image(&bytes);
            }
            AppEvent::SearchResultsLoaded(results) => {
                state_clone.lock().unwrap().search_results = results.clone();
                editor_page.clear_search_results();
                for result in &results {
                    editor_page.add_search_result(result);
                }
                let _ = tx.send(AppEvent::Status(format!("✓ Found {} results", results.len())));
            }
            AppEvent::Log(msg) => {
                editor_page.append_log(&msg);
                println!("LOG: {}", msg);
            }
             AppEvent::Status(msg) => {
                editor_page.append_log(&msg);
                editor_page.set_progress(editor_page.progress_bar.fraction(), Some(&msg));
                println!("STATUS: {}", msg);
            }
            AppEvent::Error(msg) => {
                editor_page.append_log(&format!("❌ ERROR: {}", msg));
                eprintln!("ERROR: {}", msg);
            }
            AppEvent::Complete => {
                editor_page.set_progress(1.0, Some("✓ Done"));
                // Re-enable convert button
                editor_page.convert_btn.set_sensitive(true);
            }
        }
        glib::Continue(true)
    });

    window.present();
}
