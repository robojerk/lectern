use crate::ui::{Lectern, Message};
use crate::ui::views::ViewMode;
use crate::services::conversion::{ConversionConfig, ProcessingOptions, convert_to_m4b};
use iced::Command;

pub fn handle_convert(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::StartConversion => {
            let output_path = if let Some(ref lib_path) = app.local_library_path {
                Some(crate::ui::helpers::apply_media_template(
                    &app.media_management_template,
                    lib_path,
                    &app.metadata.editing_title,
                    &app.metadata.editing_author,
                    &app.metadata.editing_series,
                    &app.metadata.editing_series_number,
                    &app.metadata.editing_publish_year,
                    &app.metadata.editing_genre,
                    &app.metadata.editing_asin,
                    &app.metadata.editing_language,
                    &app.metadata.editing_tags,
                ))
            } else {
                app.output_path.clone()
            };
            
            if let Some(path) = output_path {
                app.is_converting = true;
                app.conversion_error = None;
                app.source_size = 0;
                app.output_size = 0;
                
                let selected_book = app.metadata.selected_book.clone();
                let cover_path = app.cover.cover_image_path.clone();
                let mut chapters = app.chapters.chapters.clone();
                let audio_files = app.file.audio_file_paths.clone();
                let selected_file_path = app.file.selected_file_path.clone();
                
                // Determine input path
                // If we have multiple audio files, use the directory containing them
                // Otherwise use the selected file or first audio file
                let input_path = if !audio_files.is_empty() && audio_files.len() > 1 {
                    // Multiple files - use the directory that contains all the files
                    // Find the common parent directory of all audio files
                    let first_path = std::path::Path::new(&audio_files[0]);
                    if let Some(common_dir) = first_path.parent() {
                        // Verify all files are in the same directory
                        let all_same_dir = audio_files.iter().all(|f| {
                            std::path::Path::new(f).parent() == Some(common_dir)
                        });
                        if all_same_dir {
                            println!("[DEBUG] Using common directory for {} files: {}", audio_files.len(), common_dir.display());
                            common_dir.to_string_lossy().to_string()
                        } else {
                            // Files in different directories - use first file's directory
                            println!("[DEBUG] Files in different directories, using first file's directory");
                            common_dir.to_string_lossy().to_string()
                        }
                    } else {
                        // Fallback: use directory from selected file if available
                        if let Some(ref file_path) = selected_file_path {
                            let path = std::path::Path::new(file_path);
                            if let Some(parent) = path.parent() {
                                parent.to_string_lossy().to_string()
                            } else {
                                file_path.clone()
                            }
                        } else {
                            // Last resort: use first file's directory
                            let path = std::path::Path::new(&audio_files[0]);
                            path.parent()
                                .map(|p| p.to_string_lossy().to_string())
                                .unwrap_or_else(|| audio_files[0].clone())
                        }
                    }
                } else if let Some(ref file_path) = selected_file_path {
                    file_path.clone()
                } else if !audio_files.is_empty() {
                    // Single audio file
                    audio_files[0].clone()
                } else {
                    return Some(Command::perform(
                        async move { Err("No input file or audio files found".to_string()) },
                        Message::ConversionCompleted,
                    ));
                };
                
                println!("[DEBUG] Determined input path: {}", input_path);
                
                // If rewrite_chapters is enabled and we have files but no chapters, generate them
                let mut processing_options = ProcessingOptions::default();
                processing_options.normalize_volume = app.conversion_normalize_volume;
                
                if processing_options.rewrite_chapters && !audio_files.is_empty() && chapters.is_empty() {
                    // Generate chapters from files
                    match crate::services::conversion::generate_chapters_from_files(&audio_files) {
                        Ok(generated) => {
                            chapters = generated;
                            println!("[DEBUG] Generated {} chapters from {} files", chapters.len(), audio_files.len());
                        }
                        Err(e) => {
                            return Some(Command::perform(
                                async move { Err(format!("Failed to generate chapters: {}", e)) },
                                Message::ConversionCompleted,
                            ));
                        }
                    }
                }
                
                if selected_book.is_none() {
                    return Some(Command::perform(
                        async move { Err("No book selected".to_string()) },
                        Message::ConversionCompleted,
                    ));
                }
                
                let book = selected_book.unwrap();
                let audio_bitrate = if app.conversion_bitrate == "auto" {
                    None
                } else {
                    Some(app.conversion_bitrate.clone())
                };

                let audio_channels = match app.conversion_channels.as_str() {
                    "1" => Some(1),
                    "2" => Some(2),
                    _ => None,
                };

                let config = ConversionConfig {
                    input_path,
                    output_path: path.clone(),
                    book_metadata: book,
                    cover_image_path: cover_path,
                    chapters,
                    audio_bitrate,
                    audio_codec: app.conversion_codec.clone(),
                    audio_channels,
                    processing_options,
                };
                
                // Calculate source size
                let mut source_size = 0u64;
                if !audio_files.is_empty() {
                    for file in &audio_files {
                        if let Ok(metadata) = std::fs::metadata(file) {
                            source_size += metadata.len();
                        }
                    }
                } else if let Some(ref file_path) = selected_file_path {
                    if let Ok(metadata) = std::fs::metadata(file_path) {
                        source_size += metadata.len();
                    }
                }
                
                return Some(Command::perform(
                    async move {
                        match convert_to_m4b(config).await {
                            Ok(output) => {
                                let output_size = std::fs::metadata(&output)
                                    .map(|m| m.len())
                                    .unwrap_or(0);
                                Ok((output, source_size, output_size))
                            },
                            Err(e) => Err(format!("Conversion failed: {}", e)),
                        }
                    },
                    Message::ConversionCompleted,
                ));
            } else {
                // Need to browse for output path
                return Some(Command::perform(async move {
                    let (tx, rx) = futures::channel::oneshot::channel();
                    std::thread::spawn(move || {
                        let dialog = rfd::FileDialog::new()
                            .add_filter("M4B Files", &["m4b"])
                            .set_file_name("output.m4b");
                        let result = dialog.save_file()
                            .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                    });
                    rx.await.unwrap_or(None)
                }, |path| {
                    Message::OutputPathSelected(path)
                }));
            }
        }
        Message::BrowseOutputPath => {
            let default_filename = app.metadata.selected_book.as_ref()
                .map(|b| format!("{}.m4b", b.title.replace("/", "-")))
                .unwrap_or_else(|| "output.m4b".to_string());
            Some(Command::perform(async move {
                let (tx, rx) = futures::channel::oneshot::channel();
                let filename = default_filename.clone();
                std::thread::spawn(move || {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("M4B Files", &["m4b"])
                        .set_file_name(&filename);
                    let result = dialog.save_file()
                        .map(|p| p.to_string_lossy().to_string());
                        let _ = tx.send(result);
                });
                rx.await.unwrap_or(None)
            }, |path| {
                Message::OutputPathSelected(path)
            }))
        }
        Message::OutputPathSelected(Some(path)) => {
            app.output_path = Some(path);
            // Don't auto-start - let user click the button
            Some(Command::none())
        }
        Message::OutputPathSelected(None) => {
            // User cancelled
            Some(Command::none())
        }
        Message::ConversionCompleted(Ok((path, src_size, out_size))) => {
            app.is_converting = false;
            app.conversion_error = None;
            app.source_size = src_size;
            app.output_size = out_size;
            println!("[DEBUG] Conversion completed: {} (Source: {} bytes, Output: {} bytes)", path, src_size, out_size);
            // TODO: Show success message, optionally upload to Audiobookshelf
            Some(Command::none())
        }
        Message::ConversionCompleted(Err(e)) => {
            app.is_converting = false;
            app.conversion_error = Some(e);
            Some(Command::none())
        }
        Message::SwitchToConvert => {
            app.view_mode = ViewMode::Convert;
            Some(Command::none())
        }
        Message::ConversionNormalizeVolumeToggled(normalize) => {
            app.conversion_normalize_volume = normalize;
            Some(Command::none())
        }
        Message::ConversionBitrateChanged(bitrate) => {
            app.conversion_bitrate = bitrate;
            Some(Command::none())
        }
        Message::ConversionCodecChanged(codec) => {
            app.conversion_codec = codec;
            Some(Command::none())
        }
        Message::ConversionChannelsChanged(channels) => {
            app.conversion_channels = channels;
            Some(Command::none())
        }
        _ => None,
    }
}
