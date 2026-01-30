use crate::ui::{Lectern, Message};
use crate::ui::helpers::{parse_audiobook_file, get_audio_files_from_directory, find_local_cover_in_directory, find_metadata_or_chapter_files};
use crate::ui::cover_search::download_image;
use crate::ui::views::ViewMode;
use crate::ui::state::CoverState;
use crate::utils::chapter_file::{parse_chapters_from_path, is_chapter_file_name};
use crate::services::ffprobe::{extract_chapters_from_file, generate_chapters_from_files};
use std::path::Path;
use iced::Command;

pub fn handle_file(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::CloseBook => {
            app.metadata.selected_book = None;
            app.file.selected_file_path = None;
            app.file.audio_file_paths.clear();
            app.file.found_metadata_chapter_files.clear();
            app.file.file_parse_error = None;
            // Stop any in-flight chapter loading and wipe chapter state
            app.chapters.is_mapping_from_files = false;
            app.chapters.is_looking_up_chapters = false;
            app.chapters.chapters.clear();
            app.chapters.lookup_error = None;
            app.chapters.book_duration_ms = None;
            app.chapters.chapter_time_editing.clear();
            app.chapters.lookup_result = None;
            app.chapters.lookup_duration_ms = None;
            app.chapters.load_generation = app.chapters.load_generation.wrapping_add(1);
            // Wipe cover state for the previous book
            app.cover = CoverState::default();
            // Stop chapter playback if running
            app.chapter_playback_state = None;
            let _ = app.chapter_playback_process.take();
            app.view_mode = ViewMode::Metadata;
            Some(Command::none())
        }
        Message::BrowseFiles => {
            Some(Command::perform(async move {
                let (tx, rx) = futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("Audiobook Files", &["m4b", "m4a"])
                        .add_filter("Audio Files", &["mp3", "aac", "wav", "flac", "m4b", "m4a"])
                        .add_filter("All Files", &["*"]);
                    
                    let result = dialog.pick_file()
                        .map(|p| p.to_string_lossy().to_string());
                    let _ = tx.send(result);
                });
                rx.await.unwrap_or(None)
            }, |path| {
                if let Some(p) = path {
                    Message::FileSelected(Some(p))
                } else {
                    Message::FileSelected(None)
                }
            }))
        }
        Message::BrowseFolder => {
            Some(Command::perform(async move {
                let (tx, rx) = futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let dialog = rfd::FileDialog::new();
                    let result = dialog.pick_folder()
                        .map(|p| p.to_string_lossy().to_string());
                    let _ = tx.send(result);
                });
                rx.await.unwrap_or(None)
            }, |path| {
                if let Some(p) = path {
                    Message::FileSelected(Some(p))
                } else {
                    Message::FileSelected(None)
                }
            }))
        }
        Message::FileSelected(Some(path)) => {
            app.file.selected_file_path = Some(path.clone());
            app.file.is_parsing_file = true;
            app.file.file_parse_error = None;
            app.source_size = 0;
            app.output_size = 0;
            
            let path_clone = path.clone();
            Some(Command::perform(
                async move {
                    parse_audiobook_file(&path_clone)
                },
                Message::FileParsed,
            ))
        }
        Message::FileDropped(paths) => {
            println!("[DEBUG] FileDropped handler - received {} paths: {:?}", paths.len(), paths);
            
            // Filter out invalid paths and reconstruct if needed
            let valid_paths: Vec<String> = if paths.len() > 1 && paths.first().map(|p| p.as_str()) == Some("/") {
                // Path might have been split into components - try to reconstruct
                let mut reconstructed = String::new();
                for (i, component) in paths.iter().enumerate() {
                    if i == 0 {
                        reconstructed.push_str(component);
                    } else {
                        if !reconstructed.ends_with('/') {
                            reconstructed.push('/');
                        }
                        reconstructed.push_str(component);
                    }
                }
                println!("[DEBUG] Attempting to reconstruct path from {} components: '{}'", paths.len(), reconstructed);
                if Path::new(&reconstructed).exists() {
                    vec![reconstructed]
                } else {
                    let alt_reconstructed = paths.join("");
                    println!("[DEBUG] Trying alternative reconstruction: '{}'", alt_reconstructed);
                    if Path::new(&alt_reconstructed).exists() {
                        vec![alt_reconstructed]
                    } else {
                        let mut filtered: Vec<String> = paths.iter()
                            .filter(|p| {
                                let path = Path::new(p);
                                !p.is_empty() && p.as_str() != "/" && path.exists()
                            })
                            .cloned()
                            .collect();
                        filtered.sort_by(|a, b| b.len().cmp(&a.len()));
                        filtered
                    }
                }
            } else {
                let mut filtered: Vec<String> = paths.iter()
                    .filter(|p| {
                        let path = Path::new(p);
                        !p.is_empty() && p.as_str() != "/" && path.exists()
                    })
                    .cloned()
                    .collect();
                filtered.sort_by(|a, b| b.len().cmp(&a.len()));
                filtered
            };
            
            println!("[DEBUG] Filtered to {} valid paths: {:?}", valid_paths.len(), valid_paths);
            
            if let Some(path) = valid_paths.first() {
                println!("[DEBUG] Processing dropped path: '{}'", path);
                app.file.selected_file_path = Some(path.clone());
                app.file.is_parsing_file = true;
                app.file.file_parse_error = None;
                app.source_size = 0;
                app.output_size = 0;
                
                let path_clone = path.clone();
                let path_obj = Path::new(&path_clone);
                if path_obj.is_dir() {
                    println!("[DEBUG] Path is a directory, scanning for audio files...");
                    let audio_files = get_audio_files_from_directory(&path_clone);
                    println!("[DEBUG] Found {} audio files in directory", audio_files.len());
                    if !audio_files.is_empty() {
                        app.file.audio_file_paths = audio_files.clone();
                        println!("[DEBUG] Parsing directory metadata for: '{}'", path_clone);
                        return Some(Command::perform(
                            async move {
                                let result = parse_audiobook_file(&path_clone);
                                match &result {
                                    Ok(meta) => println!("[DEBUG] Directory parsed successfully: '{}' by '{}' ({} files)", 
                                        meta.title, meta.author, audio_files.len()),
                                    Err(e) => println!("[ERROR] Directory parse error: {}", e),
                                }
                                result
                            },
                            Message::FileParsed,
                        ));
                    } else {
                        app.file.is_parsing_file = false;
                        let error_msg = format!("No audio files found in directory: {}", path_clone);
                        println!("[ERROR] {}", error_msg);
                        app.file.file_parse_error = Some(error_msg);
                        return Some(Command::none());
                    }
                } else {
                    println!("[DEBUG] Path is a file, parsing directly...");
                    return Some(Command::perform(
                        async move {
                            let result = parse_audiobook_file(&path_clone);
                            match &result {
                                Ok(meta) => println!("[DEBUG] File parsed successfully: '{}' by '{}'", meta.title, meta.author),
                                Err(e) => println!("[DEBUG] File parse error: {}", e),
                            }
                            result
                        },
                        Message::FileParsed,
                    ));
                }
            } else {
                let error_msg = format!("No valid paths in dropped files. Received {} paths, but none were valid.", paths.len());
                println!("[ERROR] {}", error_msg);
                app.file.file_parse_error = Some(error_msg);
            }
            Some(Command::none())
        }
        Message::FileSelected(None) => {
            // User cancelled file selection
            Some(Command::none())
        }
        Message::FileParsed(Ok(metadata)) => {
            println!("[DEBUG] FileParsed(Ok) - Successfully parsed file/directory");
            app.file.is_parsing_file = false;
            app.metadata.selected_book = Some(metadata.clone());
            // Populate editing fields
            app.metadata.editing_title = metadata.title.clone();
            app.metadata.editing_subtitle = metadata.subtitle.unwrap_or_default();
            app.metadata.editing_author = metadata.author.clone();
            app.metadata.editing_series = metadata.series.unwrap_or_default();
            app.metadata.editing_series_number = metadata.series_number.unwrap_or_default();
            app.metadata.editing_narrator = metadata.narrator.unwrap_or_default();
            app.metadata.editing_description = metadata.description.unwrap_or_default();
            app.metadata.editing_description_content = iced::widget::text_editor::Content::with_text(&app.metadata.editing_description);
            app.metadata.editing_isbn = metadata.isbn.unwrap_or_default();
            app.metadata.editing_asin = metadata.asin.unwrap_or_default();
            app.metadata.editing_publisher = metadata.publisher.unwrap_or_default();
            app.metadata.editing_publish_year = metadata.publish_year.unwrap_or_default();
            app.metadata.editing_genre = metadata.genre.unwrap_or_default();
            app.metadata.editing_tags = metadata.tags.unwrap_or_default();
            app.metadata.editing_language = metadata.language.unwrap_or_default();
            app.metadata.editing_explicit = metadata.explicit.unwrap_or(false);
            app.metadata.editing_abridged = metadata.abridged.unwrap_or(false);
            
            // Initialize cover image path and handle caching
            app.cover.cover_image_path = metadata.cover_url.clone();
            if let Some(ref cover_url) = metadata.cover_url {
                if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
                    if app.cover.cover_image_url_cached.as_ref() != Some(cover_url) {
                        app.cover.cover_image_data = None;
                        app.cover.cover_image_handle = None;
                        app.cover.cover_image_url_cached = None;
                        app.cover.is_downloading_cover = true;
                        let url_clone = cover_url.clone();
                        return Some(Command::perform(
                            async move {
                                match download_image(&url_clone).await {
                                    Ok((url, data)) => {
                                        if let Ok(img) = ::image::load_from_memory(&data) {
                                            let rgba = img.to_rgba8();
                                            let (width, height) = rgba.dimensions();
                                            let pixels: Vec<u8> = rgba.into_raw();
                                            let handle = iced::widget::image::Handle::from_pixels(width, height, pixels);
                                            Ok((url, data, handle))
                                        } else {
                                            Err("Failed to decode image".to_string())
                                        }
                                    },
                                    Err(e) => Err(e),
                                }
                            },
                            Message::CoverImageDownloaded,
                        ));
                    } else {
                        println!("[DEBUG] Cover image already cached for URL: {}", cover_url);
                    }
                } else {
                    // Local file - load handle
                    app.cover.cover_image_data = None;
                    app.cover.cover_image_url_cached = None;
                    app.cover.cover_image_handle = None;
                    
                    let path = std::path::Path::new(cover_url);
                    if path.exists() {
                        if let Ok(img_data) = std::fs::read(path) {
                            if let Ok(img) = ::image::load_from_memory(&img_data) {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let pixels: Vec<u8> = rgba.into_raw();
                                app.cover.cover_image_handle = Some(iced::widget::image::Handle::from_pixels(width, height, pixels));
                            }
                        }
                    }
                }
            } else {
                app.cover.cover_image_data = None;
                app.cover.cover_image_handle = None;
                app.cover.cover_image_url_cached = None;
            }
            app.chapters.book_duration_ms = None;
            println!("[DEBUG] FileParsed - Switching to Metadata view");
            app.view_mode = crate::ui::views::ViewMode::Metadata;
            // If directory and no cover from metadata, look for local cover (folder.jpg, cover.jpg, etc.)
            if app.cover.cover_image_handle.is_none() {
                if let Some(ref file_path) = app.file.selected_file_path {
                    if Path::new(file_path).is_dir() {
                        if let Some(local_cover) = find_local_cover_in_directory(file_path) {
                            app.cover.cover_image_path = Some(local_cover.clone());
                            app.cover.cover_image_data = None;
                            app.cover.cover_image_url_cached = None;
                            let path = std::path::Path::new(&local_cover);
                            if path.exists() {
                                if let Ok(img_data) = std::fs::read(path) {
                                    if let Ok(img) = ::image::load_from_memory(&img_data) {
                                        let rgba = img.to_rgba8();
                                        let (width, height) = rgba.dimensions();
                                        let pixels: Vec<u8> = rgba.into_raw();
                                        app.cover.cover_image_handle = Some(iced::widget::image::Handle::from_pixels(width, height, pixels));
                                    }
                                }
                            }
                        }
                    }
                }
            }
            
            // Store audio file paths and scan for metadata/chapter files if directory was selected
            if let Some(ref file_path) = app.file.selected_file_path {
                if Path::new(file_path).is_dir() {
                    let audio_files = get_audio_files_from_directory(file_path);
                    app.file.audio_file_paths = audio_files;
                    app.file.found_metadata_chapter_files = find_metadata_or_chapter_files(file_path);
                    println!("[DEBUG] Stored {} audio file paths, {} metadata/chapter files", app.file.audio_file_paths.len(), app.file.found_metadata_chapter_files.len());
                    if !app.file.found_metadata_chapter_files.is_empty() {
                        let names: Vec<&str> = app.file.found_metadata_chapter_files.iter().map(|(n, _)| n.as_str()).collect();
                        println!("[DEBUG] Found in folder: {}", names.join(", "));
                    }

                    // Auto-load chapters from a chapter file in the directory (txt, json, cue, ini)
                    for (name, path) in &app.file.found_metadata_chapter_files {
                        if is_chapter_file_name(name) {
                            if let Ok(chapters) = parse_chapters_from_path(path) {
                                if !chapters.is_empty() {
                                    app.chapters.chapters = chapters;
                                    app.chapters.lookup_error = None;
                                    println!("[DEBUG] Loaded {} chapters from file {}", app.chapters.chapters.len(), name);
                                    break;
                                }
                            }
                        }
                    }
                } else {
                    app.file.audio_file_paths.clear();
                    app.file.found_metadata_chapter_files.clear();

                    // Auto-extract chapters from single file (e.g. M4B with embedded chapters)
                    app.chapters.is_looking_up_chapters = true;
                    app.chapters.lookup_error = None;
                    let path_clone = file_path.clone();
                    let gen = app.chapters.load_generation;
                    return Some(Command::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                extract_chapters_from_file(&path_clone).map_err(|e| e.to_string())
                            })
                            .await
                            .unwrap_or_else(|_| Err("Task join error".to_string()))
                        },
                        move |result| Message::ChapterExtractCompleted(gen, result),
                    ));
                }
            }

            // If directory with audio files but no chapters yet, auto-map one chapter per file
            let need_duration = !app.file.audio_file_paths.is_empty();
            let need_map_chapters = need_duration && app.chapters.chapters.is_empty();
            if need_map_chapters {
                app.chapters.is_mapping_from_files = true;
                app.chapters.lookup_error = None;
                let paths = app.file.audio_file_paths.clone();
                let gen = app.chapters.load_generation;
                let map_cmd = Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            generate_chapters_from_files(&paths).map_err(|e| e.to_string())
                        })
                        .await
                        .unwrap_or_else(|_| Err("Task failed".to_string()))
                    },
                    move |result| Message::MapChaptersFromFilesCompleted(gen, result),
                );
                if need_duration {
                    let duration_paths = app.file.audio_file_paths.clone();
                    let duration_gen = app.chapters.load_generation;
                    let duration_cmd = Command::perform(
                        async move {
                            tokio::task::spawn_blocking(move || {
                                crate::services::conversion::get_total_duration(&duration_paths).map_err(|e| e.to_string())
                            })
                            .await
                            .unwrap_or(Err("Task failed".into()))
                        },
                        move |result| Message::BookDurationComputed(duration_gen, result),
                    );
                    return Some(Command::batch([duration_cmd, map_cmd]));
                }
                return Some(map_cmd);
            }
            // Compute total book duration in background (for chapters validation)
            if need_duration {
                let paths = app.file.audio_file_paths.clone();
                let duration_gen = app.chapters.load_generation;
                return Some(Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            crate::services::conversion::get_total_duration(&paths).map_err(|e| e.to_string())
                        })
                        .await
                        .unwrap_or(Err("Task failed".into()))
                    },
                    move |result| Message::BookDurationComputed(duration_gen, result),
                ));
            }
            Some(Command::none())
        }
        Message::FileParsed(Err(e)) => {
            println!("[DEBUG] FileParsed(Err) - Error: {}", e);
            app.file.is_parsing_file = false;
            app.file.file_parse_error = Some(e.clone());
            println!("[ERROR] Failed to parse file/directory: {}", e);
            Some(Command::none())
        }
        _ => None,
    }
}
