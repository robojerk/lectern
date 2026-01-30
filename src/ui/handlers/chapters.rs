use crate::ui::{Lectern, Message, ChapterPlaybackState, ChapterPlaybackProcess};
use crate::utils::time::{parse_time_string, format_time};
use crate::services::ffprobe::{get_audio_file_duration, extract_chapters_from_file, generate_chapters_from_files};
use crate::services::playback::{play_chapter_headless, find_audio_file_for_chapter};
use crate::models::Chapter;
use crate::services::AudioService;
use crate::ui::views::ViewMode;
use std::path::Path;
use std::time::{Duration, Instant};
use std::sync::Arc;
use tokio::sync::Mutex;
use iced::Command;

pub fn handle_chapters(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::ChapterTitleChanged(index, new_title) => {
            if let Some(chapter) = app.chapters.chapters.get_mut(index) {
                if !chapter.is_locked {
                    chapter.title = new_title;
                }
            }
            Some(Command::none())
        }
        Message::ChapterTimeChanged(index, time_str) => {
            if app.chapters.chapters.get(index).map(|c| c.is_locked).unwrap_or(true) {
                return Some(Command::none());
            }
            // Store the raw input value while editing
            app.chapters.chapter_time_editing.insert(index, time_str.clone());
            
            // Try to parse and validate, but don't update chapter until valid
            if let Ok(seconds) = parse_time_string(&time_str) {
                let new_start_time = (seconds * 1000) as u64;
                // Validate: new start time must be >= previous chapter's start time
                if index > 0 {
                    if let Some(prev_chapter) = app.chapters.chapters.get(index - 1) {
                        if new_start_time < prev_chapter.start_time {
                            app.chapters.lookup_error = Some(format!(
                                "Invalid start time: must be greater than or equal to previous chapter start time ({})",
                                format_time(prev_chapter.start_time, app.chapters.show_seconds)
                            ));
                            return Some(Command::none());
                        }
                    }
                }
                // Now update the chapter (after validation)
                if let Some(chapter) = app.chapters.chapters.get_mut(index) {
                    chapter.start_time = new_start_time;
                    app.chapters.lookup_error = None; // Clear error on valid input
                }
            }
            // If parsing fails, just keep the raw input stored - don't show error while typing
            Some(Command::none())
        }
        Message::ChapterTimeAdjusted(index, adjustment_seconds) => {
            if let Some(chapter) = app.chapters.chapters.get_mut(index) {
                if !chapter.is_locked {
                    let current_seconds = (chapter.start_time / 1000) as i64;
                    let new_seconds = (current_seconds + adjustment_seconds).max(0);
                    chapter.start_time = (new_seconds * 1000) as u64;
                    // Keep displayed value in sync so the text field and +1/-1 stay correct
                    app.chapters.chapter_time_editing.insert(index, format_time(chapter.start_time, app.chapters.show_seconds));
                }
            }
            Some(Command::none())
        }
        Message::ChapterLockToggled(index) => {
            if app.chapters.shift_held {
                if let Some(anchor) = app.chapters.last_lock_clicked_index {
                    let (lo, hi) = (anchor.min(index), anchor.max(index));
                    let target_locked = app.chapters.chapters.get(index).map(|c| !c.is_locked).unwrap_or(true);
                    for j in lo..=hi {
                        if let Some(ch) = app.chapters.chapters.get_mut(j) {
                            ch.is_locked = target_locked;
                        }
                    }
                    app.chapters.last_lock_clicked_index = Some(index);
                } else {
                    if let Some(chapter) = app.chapters.chapters.get_mut(index) {
                        chapter.is_locked = !chapter.is_locked;
                    }
                    app.chapters.last_lock_clicked_index = Some(index);
                }
            } else {
                if let Some(chapter) = app.chapters.chapters.get_mut(index) {
                    chapter.is_locked = !chapter.is_locked;
                }
                app.chapters.last_lock_clicked_index = None;
            }
            Some(Command::none())
        }
        Message::ShiftModifierChanged(held) => {
            app.chapters.shift_held = held;
            if !held {
                app.chapters.last_lock_clicked_index = None;
            }
            Some(Command::none())
        }
        Message::ChapterDelete(index) => {
            if index < app.chapters.chapters.len()
                && !app.chapters.chapters.get(index).map(|c| c.is_locked).unwrap_or(true)
            {
                app.chapters.chapters.remove(index);
            }
            Some(Command::none())
        }
        Message::ChapterInsertBelow(index) => {
            let new_start_time = if index < app.chapters.chapters.len() {
                let current = &app.chapters.chapters[index];
                current.start_time + current.duration
            } else if let Some(last) = app.chapters.chapters.last() {
                last.start_time + last.duration
            } else {
                0
            };
            let new_chapter = Chapter::new(
                format!("Chapter {}", app.chapters.chapters.len() + 1),
                new_start_time,
                0, // Duration will be calculated
            );
            if index + 1 < app.chapters.chapters.len() {
                app.chapters.chapters.insert(index + 1, new_chapter);
            } else {
                app.chapters.chapters.push(new_chapter);
            }
            Some(Command::none())
        }
        Message::ChapterRemoveAll => {
            app.chapters.chapters.clear();
            Some(Command::none())
        }
        Message::ChapterShiftTimes(seconds) => {
            for chapter in &mut app.chapters.chapters {
                if !chapter.is_locked {
                    let current_seconds = (chapter.start_time / 1000) as i64;
                    let new_seconds = (current_seconds + seconds).max(0);
                    chapter.start_time = (new_seconds * 1000) as u64;
                }
            }
            Some(Command::none())
        }
        Message::ChapterLookup => {
            app.chapters.is_looking_up_chapters = true;
            app.chapters.lookup_error = None;

            // Use manual input, then metadata ASIN, then selected book ASIN, then metadata ISBN, then selected book ISBN
            let identifier = if !app.chapters.asin_input.trim().is_empty() {
                Some(app.chapters.asin_input.trim().to_string())
            } else if !app.metadata.editing_asin.trim().is_empty() {
                Some(app.metadata.editing_asin.trim().to_string())
            } else {
                app.metadata.selected_book.as_ref().and_then(|b| b.asin.clone())
                    .or_else(|| {
                        if !app.metadata.editing_isbn.trim().is_empty() {
                            Some(app.metadata.editing_isbn.trim().to_string())
                        } else {
                            app.metadata.selected_book.as_ref().and_then(|b| b.isbn.clone())
                        }
                    })
            };

            let region = app.chapters.selected_region.to_string().to_lowercase();

            if let Some(asin_val) = identifier {
                println!("[DEBUG] Looking up chapters for ASIN/ISBN: {} (Region: {})", asin_val, region);
                let gen = app.chapters.load_generation;
                return Some(Command::perform(
                    async move {
                        AudioService::fetch_chapters_by_asin(&asin_val, &region).await
                    },
                    move |result| Message::ChapterLookupCompleted(gen, result),
                ));
            } else {
                app.chapters.is_looking_up_chapters = false;
                app.chapters.lookup_error = Some("No ASIN or ISBN available. Enter one in the lookup field above or fill in Metadata (ISBN/ASIN) first.".to_string());
            }
            Some(Command::none())
        }
        Message::ChapterToggleAsinInput => {
            app.chapters.show_asin_input = !app.chapters.show_asin_input;
            // When opening the lookup panel, pre-populate from metadata: ASIN first, then ISBN
            if app.chapters.show_asin_input && app.chapters.asin_input.trim().is_empty() {
                if !app.metadata.editing_asin.trim().is_empty() {
                    app.chapters.asin_input = app.metadata.editing_asin.trim().to_string();
                } else if let Some(ref book) = app.metadata.selected_book {
                    if let Some(ref asin) = book.asin {
                        app.chapters.asin_input = asin.clone();
                    } else if let Some(ref isbn) = book.isbn {
                        app.chapters.asin_input = isbn.clone();
                    }
                } else if !app.metadata.editing_isbn.trim().is_empty() {
                    app.chapters.asin_input = app.metadata.editing_isbn.trim().to_string();
                }
            }
            Some(Command::none())
        }
        Message::ChapterRegionChanged(region) => {
            app.chapters.selected_region = region;
            Some(Command::none())
        }
        Message::ChapterListViewportChanged { offset_y, viewport_height, content_height } => {
            app.chapters.chapter_list_viewport = Some((offset_y, viewport_height, content_height));
            Some(Command::none())
        }
        Message::ChapterRemoveAudibleToggled(remove) => {
            app.chapters.remove_audible_intro_outro = remove;
            Some(Command::none())
        }
        Message::ChapterAsinChanged(asin) => {
            app.chapters.asin_input = asin;
            Some(Command::none())
        }
        Message::MapChaptersFromFiles => {
            if app.file.audio_file_paths.is_empty() {
                app.chapters.lookup_error = Some("No audio files found. Please select a directory with audio files first.".to_string());
                Some(Command::none())
            } else {
                app.chapters.is_mapping_from_files = true;
                app.chapters.lookup_error = None;
                let paths = app.file.audio_file_paths.clone();
                let gen = app.chapters.load_generation;
                Some(Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            generate_chapters_from_files(&paths).map_err(|e| e.to_string())
                        })
                        .await
                        .unwrap_or_else(|_| Err("Task failed".to_string()))
                    },
                    move |result| Message::MapChaptersFromFilesCompleted(gen, result),
                ))
            }
        }
        Message::MapChaptersFromFilesCompleted(gen, result) => {
            app.chapters.is_mapping_from_files = false;
            if gen != app.chapters.load_generation {
                return Some(Command::none());
            }
            match result {
                Ok(chapters) => {
                    app.chapters.chapters = chapters;
                    println!("[DEBUG] Mapped {} chapters from {} audio files", app.chapters.chapters.len(), app.file.audio_file_paths.len());
                }
                Err(e) => {
                    app.chapters.lookup_error = Some(format!("Failed to generate chapters: {}", e));
                    println!("[ERROR] Failed to generate chapters: {}", e);
                }
            }
            Some(Command::none())
        }
        Message::BookDurationComputed(gen, result) => {
            if gen == app.chapters.load_generation {
                app.chapters.book_duration_ms = result.ok();
            }
            Some(Command::none())
        }
        Message::ChapterExtractFromFile => {
            if let Some(ref file_path) = app.file.selected_file_path {
                app.chapters.is_looking_up_chapters = true;
                app.chapters.lookup_error = None;
                let path_clone = file_path.clone();
                let gen = app.chapters.load_generation;
                return Some(Command::perform(
                    async move {
                        tokio::task::spawn_blocking(move || {
                            extract_chapters_from_file(&path_clone).map_err(|e| e.to_string())
                        }).await.unwrap_or_else(|_| Err("Task join error".to_string()))
                    },
                    move |result| Message::ChapterExtractCompleted(gen, result),
                ));
            } else {
                app.chapters.lookup_error = Some("No file selected. Please select an audio file first.".to_string());
            }
            Some(Command::none())
        }
        Message::ChapterExtractCompleted(gen, Ok(chapters)) => {
            app.chapters.is_looking_up_chapters = false;
            if gen != app.chapters.load_generation {
                return Some(Command::none());
            }
            if !chapters.is_empty() {
                app.chapters.chapters = chapters;
                println!("[DEBUG] Extracted {} chapters from file", app.chapters.chapters.len());
            } else {
                app.chapters.lookup_error = Some("No chapters found in file".to_string());
            }
            Some(Command::none())
        }
        Message::ChapterExtractCompleted(gen, Err(e)) => {
            app.chapters.is_looking_up_chapters = false;
            if gen == app.chapters.load_generation {
                app.chapters.lookup_error = Some(format!("Failed to extract chapters: {}", e));
                println!("[ERROR] Failed to extract chapters: {}", e);
            }
            Some(Command::none())
        }
        Message::ChapterShiftAll(offset_ms) => {
            let chapters_count = app.chapters.chapters.len();
            for chapter in &mut app.chapters.chapters {
                if !chapter.is_locked {
                    if offset_ms < 0 && chapter.start_time < (-offset_ms) as u64 {
                        chapter.start_time = 0;
                    } else {
                        chapter.start_time = (chapter.start_time as i64 + offset_ms).max(0) as u64;
                    }
                }
            }
            println!("[DEBUG] Shifted {} unlocked chapters by {} ms", chapters_count, offset_ms);
            Some(Command::none())
        }
        Message::ChapterShiftAmountChanged(s) => {
            app.chapters.shift_all_input = s;
            Some(Command::none())
        }
        Message::ChapterShiftAllApply => {
            let s = app.chapters.shift_all_input.trim();
            if let Ok(secs) = s.parse::<f64>() {
                let offset_ms = (secs * 1000.0).round() as i64;
                if offset_ms != 0 {
                    for chapter in &mut app.chapters.chapters {
                        if !chapter.is_locked {
                            if offset_ms < 0 && chapter.start_time < (-offset_ms) as u64 {
                                chapter.start_time = 0;
                            } else {
                                chapter.start_time = (chapter.start_time as i64 + offset_ms).max(0) as u64;
                            }
                        }
                    }
                    println!("[DEBUG] Shifted unlocked chapters by {} s ({} ms)", secs, offset_ms);
                }
            }
            Some(Command::none())
        }
        Message::ChapterValidate => {
            let total_duration = app.file.selected_file_path.as_ref()
                .and_then(|path| {
                    if Path::new(path).is_file() {
                        get_audio_file_duration(path).map_err(|e| e.to_string()).ok()
                    } else if !app.file.audio_file_paths.is_empty() {
                        let mut total = 0u64;
                        for file in &app.file.audio_file_paths {
                            if let Ok(dur) = get_audio_file_duration(file).map_err(|e| e.to_string()) {
                                total += dur;
                            }
                        }
                        if total > 0 { Some(total) } else { None }
                    } else {
                        None
                    }
                });
            
            let errors = Chapter::validate_list(&app.chapters.chapters, total_duration);
            if errors.is_empty() {
                app.chapters.lookup_error = None;
                println!("[DEBUG] Chapter validation passed");
            } else {
                let error_msg = format!("Validation issues: {}", errors.join("; "));
                app.chapters.lookup_error = Some(error_msg.clone());
                println!("[WARNING] Chapter validation: {}", error_msg);
            }
            Some(Command::none())
        }
        Message::ChapterShiftWithRipple(index, new_start_ms) => {
            match Chapter::shift_with_ripple(&mut app.chapters.chapters, index, new_start_ms) {
                Ok(()) => {
                    println!("[DEBUG] Shifted chapter {} to {} ms with ripple effect", index + 1, new_start_ms);
                },
                Err(e) => {
                    app.chapters.lookup_error = Some(e.clone());
                    println!("[ERROR] Failed to shift chapter: {}", e);
                }
            }
            Some(Command::none())
        }
        Message::ChapterPlay(index) => {
            // Stop any existing playback first
            if let Some(ref state) = app.chapter_playback_state {
                if state.is_playing {
                    if let Some(process_handle) = app.chapter_playback_process.take() {
                        let process = process_handle.clone();
                        return Some(Command::perform(
                            async move {
                                let mut proc = process.lock().await;
                                let _ = proc.process.kill().await;
                            },
                            move |_| Message::ChapterPlay(index), // Retry after stopping
                        ));
                    }
                    app.chapter_playback_state = None;
                }
            }
            
            if index >= app.chapters.chapters.len() {
                app.chapters.lookup_error = Some("Invalid chapter index".to_string());
                return Some(Command::none());
            }
            
            let chapter = &app.chapters.chapters[index];
            
            if let Some((file_path, start_time_ms)) = find_audio_file_for_chapter(
                app.file.selected_file_path.as_ref(),
                &app.file.audio_file_paths,
                chapter.start_time,
            ) {
                // Calculate preview duration, but ensure it doesn't exceed remaining file duration
                let file_duration_ms = crate::services::ffprobe::get_audio_file_duration(&file_path)
                    .unwrap_or(chapter.duration);
                let remaining_duration_ms = file_duration_ms.saturating_sub(start_time_ms);
                
                // Preview: at least 10s so short chapters don't stop after 1–2s, cap at 30s
                let effective_duration = if chapter.duration > 0 {
                    chapter.duration.max(10000).min(30000)
                } else {
                    30000
                };
                let preview_duration_ms = Some(effective_duration.min(remaining_duration_ms).max(1000));
                
                println!("[DEBUG] Chapter {} playback: start={}ms, file_duration={}ms, remaining={}ms, chapter_duration={}ms, effective={}ms, preview={:?}ms", 
                    index + 1, start_time_ms, file_duration_ms, remaining_duration_ms, chapter.duration, effective_duration, preview_duration_ms);
                
                let path_clone = file_path.clone();
                let start_clone = start_time_ms;
                let duration_clone = preview_duration_ms;
                
                return Some(Command::perform(
                    async move {
                        let result = play_chapter_headless(&path_clone, start_clone, duration_clone).await.map_err(|e| e.to_string());
                        match result {
                            Ok(child) => {
                                let process_id_opt = child.id();
                                let process_handle = Arc::new(Mutex::new(ChapterPlaybackProcess { process: child }));
                                Ok((process_id_opt, process_handle))
                            },
                            Err(e) => Err(e),
                        }
                    },
                    move |result| {
                        match result {
                            Ok((process_id_opt, process_handle)) => {
                                Message::ChapterPlaybackStarted(index, process_id_opt, process_handle)
                            },
                            Err(e) => Message::ChapterPlaybackError(format!("Failed to play chapter: {}", e)),
                        }
                    },
                ));
            } else {
                app.chapters.lookup_error = Some("No audio file found to play".to_string());
                println!("[ERROR] No audio file found for chapter {}", index + 1);
            }
            Some(Command::none())
        }
        Message::ChapterPlaybackStarted(index, process_id_opt, process_handle) => {
            app.chapter_playback_state = Some(ChapterPlaybackState {
                chapter_index: index,
                start_time: Instant::now(),
                elapsed_ms: 0,
                is_playing: true,
                process_id: process_id_opt,
                was_manually_stopped: false,
            });
            app.chapter_playback_process = Some(process_handle.clone());
            
            if let Some(pid) = process_id_opt {
                println!("[DEBUG] Started playing chapter {} (process ID: {})", index + 1, pid);
            } else {
                println!("[DEBUG] Started playing chapter {} (process ID: unknown)", index + 1);
            }
            
            // Spawn a background task to monitor process exit
            let process_monitor = process_handle.clone();
            tokio::spawn(async move {
                let mut proc = process_monitor.lock().await;
                // Wait for the process to exit
                let _ = proc.process.wait().await;
                println!("[DEBUG] Background monitor detected process exit");
            });
            
            Some(Command::perform(
                async move {
                    tokio::time::sleep(Duration::from_millis(100)).await;
                },
                |_| Message::ChapterPlaybackTick,
            ))
        }
        Message::ChapterPlaybackError(e) => {
            app.chapters.lookup_error = Some(e.clone());
            println!("[ERROR] {}", e);
            Some(Command::none())
        }
        Message::ChapterPlaybackTick => {
            if let Some(ref mut state) = app.chapter_playback_state {
                if state.is_playing {
                    // Update elapsed time
                    state.elapsed_ms = state.start_time.elapsed().as_millis() as u64;
                    
                    // Check if process has exited (non-blocking). Use try_lock() so we don't block
                    // on the background monitor that holds the lock in wait(). Schedule next tick
                    // after 100ms so the timer advances.
                    if let Some(ref process_handle) = app.chapter_playback_process {
                        let process_check = process_handle.clone();
                        let elapsed = state.elapsed_ms;
                        
                        return Some(Command::perform(
                            async move {
                                let exit_status = if let Ok(mut proc) = process_check.try_lock() {
                                    proc.process.try_wait()
                                } else {
                                    Ok(None)
                                };
                                tokio::time::sleep(Duration::from_millis(100)).await;
                                exit_status
                            },
                            move |exit_status| {
                                if exit_status.is_ok_and(|opt| opt.is_some()) {
                                    println!("[DEBUG] Process exited detected in tick handler (after {} ms)", elapsed);
                                    Message::ChapterPlaybackProcessExited
                                } else {
                                    Message::ChapterPlaybackTick
                                }
                            },
                        ));
                    }
                    
                    // Stop after same duration we told the player (min 10s, max 30s)
                    let chapter = &app.chapters.chapters[state.chapter_index];
                    let max_duration = if chapter.duration > 0 {
                        chapter.duration.max(10000).min(30000)
                    } else {
                        30000
                    };
                    if state.elapsed_ms >= max_duration {
                        if let Some(process_handle) = app.chapter_playback_process.take() {
                            let process = process_handle.clone();
                            return Some(Command::perform(
                                async move {
                                    let mut proc = process.lock().await;
                                    let _ = proc.process.kill().await;
                                },
                                |_| Message::ChapterPlaybackProcessExited,
                            ));
                        }
                        state.is_playing = false;
                        app.chapter_playback_state = None;
                        return Some(Command::none());
                    }
                    
                    // Continue ticking - always schedule next update
                    return Some(Command::perform(
                        async move {
                            tokio::time::sleep(Duration::from_millis(100)).await;
                        },
                        |_| Message::ChapterPlaybackTick,
                    ));
                }
            }
            Some(Command::none())
        }
        Message::ChapterLoadingTick => {
            if app.chapters.is_mapping_from_files || app.chapters.is_looking_up_chapters {
                app.chapters.loading_spinner_phase = (app.chapters.loading_spinner_phase + 1) % 4;
                app.chapters.loading_spinner_rotation =
                    (app.chapters.loading_spinner_rotation + 12.0) % 360.0;
            }
            Some(Command::none())
        }
        Message::ChapterStopPlayback => {
            // Mark as manually stopped before killing process
            let process_id = app.chapter_playback_state.as_ref().and_then(|s| s.process_id);
            
            if let Some(ref mut state) = app.chapter_playback_state {
                state.was_manually_stopped = true;
                state.is_playing = false;
            }
            
            println!("[DEBUG] User manually stopped chapter playback");
            
            // Try to kill via process handle first
            if let Some(process_handle) = app.chapter_playback_process.take() {
                let process = process_handle.clone();
                // Also try kill command as immediate fallback
                if let Some(pid) = process_id {
                    let _ = std::process::Command::new("kill")
                        .arg("-TERM")
                        .arg(&pid.to_string())
                        .output();
                    println!("[DEBUG] Sent TERM signal to process {}", pid);
                }
                return Some(Command::perform(
                    async move {
                        let mut proc = process.lock().await;
                        // Use start_kill() for immediate termination, then wait
                        let _ = proc.process.start_kill();
                        // Give it a moment to die, then force kill if needed
                        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                        let _ = proc.process.kill().await;
                    },
                    |_| Message::ChapterPlaybackProcessExited,
                ));
            }
            
            // Fallback: kill via process ID if we don't have the handle
            if let Some(pid) = process_id {
                let _ = std::process::Command::new("kill")
                    .arg("-TERM")
                    .arg(&pid.to_string())
                    .output();
                println!("[DEBUG] Sent TERM signal to process {} (fallback)", pid);
                // Also try SIGKILL if TERM doesn't work
                tokio::spawn(async move {
                    tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
                    let _ = std::process::Command::new("kill")
                        .arg("-KILL")
                        .arg(&pid.to_string())
                        .output();
                });
            }
            
            app.chapter_playback_state = None;
            println!("[DEBUG] Stopped chapter playback (process killed)");
            Some(Command::none())
        }
        Message::ChapterPlaybackProcessExited => {
            // Check if process exited prematurely (before expected duration)
            // Only report error if it wasn't manually stopped
            if let Some(ref state) = app.chapter_playback_state {
                if state.is_playing && !state.was_manually_stopped {
                    let chapter = &app.chapters.chapters[state.chapter_index];
                    let expected_duration = chapter.duration.min(30000);
                    if state.elapsed_ms < expected_duration / 2 {
                        // Process exited before half the expected duration - likely an error
                        app.chapters.lookup_error = Some(format!(
                            "Playback failed for chapter {} (exited after {} ms, expected ~{} ms). Check player stderr for details.",
                            state.chapter_index + 1, state.elapsed_ms, expected_duration
                        ));
                        eprintln!("[ERROR] Chapter {} playback exited prematurely after {} ms", 
                            state.chapter_index + 1, state.elapsed_ms);
                    }
                }
            }
            if let Some(ref mut state) = app.chapter_playback_state {
                state.is_playing = false;
            }
            app.chapter_playback_state = None;
            println!("[DEBUG] Chapter playback process exited");
            Some(Command::none())
        }
        Message::ChapterLookupCompleted(gen, Ok(chapters)) => {
            app.chapters.is_looking_up_chapters = false;
            if gen != app.chapters.load_generation {
                return Some(Command::none());
            }
            app.chapters.show_asin_input = false; // Hide lookup section after successful search
            let count = chapters.len();
            if !chapters.is_empty() {
                // Store as pending; user chooses Apply (replace) or Map titles only
                app.chapters.lookup_result = Some(chapters.clone());
                let last = chapters.last().unwrap();
                app.chapters.lookup_duration_ms = Some(last.start_time + last.duration);
                println!("[DEBUG] Lookup found {} chapters from Audible; duration {:?} ms", count, app.chapters.lookup_duration_ms);
            }
            Some(Command::none())
        }
        Message::ChapterLookupApply => {
            if let Some(chapters) = app.chapters.lookup_result.take() {
                app.chapters.chapters = chapters;
                app.chapters.chapter_time_editing.clear();
                println!("[DEBUG] Applied looked-up chapters (replaced)");
            }
            Some(Command::none())
        }
        Message::MapChapterTitlesOnly => {
            if let Some(lookup) = app.chapters.lookup_result.take() {
                let n = lookup.len().min(app.chapters.chapters.len());
                for i in 0..n {
                    if let Some(ch) = app.chapters.chapters.get_mut(i) {
                        ch.title = lookup[i].title.clone();
                    }
                }
                println!("[DEBUG] Mapped {} titles to existing chapters (timestamps unchanged)", n);
            }
            Some(Command::none())
        }
        Message::ChapterLookupCancel => {
            app.chapters.lookup_result = None;
            app.chapters.lookup_duration_ms = None;
            Some(Command::none())
        }
        Message::ChapterLookupCompleted(gen, Err(e)) => {
            app.chapters.is_looking_up_chapters = false;
            if gen == app.chapters.load_generation {
                app.chapters.lookup_error = Some(e);
            }
            Some(Command::none())
        }
        Message::ChaptersShowSecondsToggled(show) => {
            app.chapters.show_seconds = show;
            Some(Command::none())
        }
        Message::ChaptersGlobalLockToggled => {
            let locked = !app.chapters.global_locked;
            app.chapters.global_locked = locked;
            for chapter in &mut app.chapters.chapters {
                chapter.is_locked = locked;
            }
            Some(Command::none())
        }
        Message::SwitchToChapters => {
            app.view_mode = ViewMode::Chapters;
            Some(Command::none())
        }
        Message::ChapterSetTimeFromPlayback(index) => {
            // Set chapter start time to current playback position in the file
            // When playing, elapsed_ms is how long we've been playing from chapter.start_time
            // So current position = chapter.start_time + elapsed_ms
            if let Some(ref state) = app.chapter_playback_state {
                if state.chapter_index == index && state.is_playing {
                    // Get current chapter start time first (before mutable borrow)
                    let current_chapter_start = app.chapters.chapters.get(index)
                        .map(|c| c.start_time)
                        .unwrap_or(0);
                    let is_locked = app.chapters.chapters.get(index)
                        .map(|c| c.is_locked)
                        .unwrap_or(true);
                    
                    if !is_locked {
                        // Calculate current absolute position in the file
                        let current_position_ms = current_chapter_start + state.elapsed_ms;
                        
                        // Validate: new start time must be >= previous chapter's start time
                        if index > 0 {
                            if let Some(prev_chapter) = app.chapters.chapters.get(index - 1) {
                                if current_position_ms < prev_chapter.start_time {
                                    app.chapters.lookup_error = Some(format!(
                                        "Invalid start time: must be greater than or equal to previous chapter start time ({})",
                                        format_time(prev_chapter.start_time, app.chapters.show_seconds)
                                    ));
                                    return Some(Command::none());
                                }
                            }
                        }
                        
                        // Now update the chapter (after validation)
                        if let Some(chapter) = app.chapters.chapters.get_mut(index) {
                            chapter.start_time = current_position_ms;
                            // Keep displayed value in sync with model
                            app.chapters.chapter_time_editing.insert(index, format_time(chapter.start_time, app.chapters.show_seconds));
                            app.chapters.lookup_error = None; // Clear error on valid input
                            println!("[DEBUG] Set chapter {} start time to {} ms (current playback position)", index + 1, current_position_ms);
                        }
                        
                        // Stop playback since we've adjusted the chapter start
                        // This prevents confusion about what we're playing
                        if let Some(process_handle) = app.chapter_playback_process.take() {
                            let process = process_handle.clone();
                            return Some(Command::perform(
                                async move {
                                    let mut proc = process.lock().await;
                                    let _ = proc.process.kill().await;
                                },
                                |_| Message::ChapterPlaybackProcessExited,
                            ));
                        }
                    } else {
                        println!("[DEBUG] Cannot set chapter {} start time - chapter is locked", index + 1);
                    }
                }
            }
            Some(Command::none())
        }
        _ => None,
    }
}
