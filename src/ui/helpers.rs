use crate::services::BookMetadata;
use crate::models::Chapter;
use std::path::Path;
use std::process::{Command, Stdio};

// Helper function to format time in milliseconds to HH:MM:SS or HH:MM:SS.mmm
pub fn format_time(milliseconds: u64, show_seconds: bool) -> String {
    let total_seconds = milliseconds / 1000;
    let hours = total_seconds / 3600;
    let minutes = (total_seconds % 3600) / 60;
    let seconds = total_seconds % 60;
    
    if show_seconds {
        let ms = milliseconds % 1000;
        format!("{:02}:{:02}:{:02}.{:03}", hours, minutes, seconds, ms)
    } else {
        format!("{:02}:{:02}:{:02}", hours, minutes, seconds)
    }
}

// Helper function to parse time string (HH:MM:SS or HH:MM:SS.mmm) to seconds
pub fn parse_time_string(time_str: &str) -> Result<u64, String> {
    let parts: Vec<&str> = time_str.split(':').collect();
    if parts.len() != 3 {
        return Err("Invalid time format. Use HH:MM:SS".to_string());
    }
    
    let hours: u64 = parts[0].parse().map_err(|_| "Invalid hours")?;
    let minutes: u64 = parts[1].parse().map_err(|_| "Invalid minutes")?;
    let seconds_part = parts[2];
    
    let (seconds, milliseconds) = if seconds_part.contains('.') {
        let sec_parts: Vec<&str> = seconds_part.split('.').collect();
        let secs: u64 = sec_parts[0].parse().map_err(|_| "Invalid seconds")?;
        let ms: u64 = sec_parts.get(1)
            .unwrap_or(&"0")
            .parse()
            .unwrap_or(0);
        (secs, ms)
    } else {
        (seconds_part.parse().map_err(|_| "Invalid seconds")?, 0)
    };
    
    Ok(hours * 3600 + minutes * 60 + seconds + (milliseconds / 1000))
}

pub fn parse_audiobook_file(path_str: &str) -> Result<BookMetadata, String> {
    println!("[DEBUG] parse_audiobook_file called with: '{}'", path_str);
    let path = Path::new(path_str);
    
    if !path.exists() {
        let error = format!("Path does not exist: {}", path_str);
        println!("[ERROR] {}", error);
        return Err(error);
    }
    
    if path.is_file() {
        // Single file (M4B, MP3, etc.)
        let extension = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        match extension.as_str() {
            "m4b" | "m4a" => {
                // Use existing get_file_metadata if available, or create basic metadata
                Ok(BookMetadata {
                    title: path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    subtitle: None,
                    author: "Unknown Author".to_string(),
                    isbn: None,
                    asin: None,
                    description: None,
                    cover_url: None,
                    duration: None,
                    narrator: None,
                    publisher: None,
                    publish_year: None,
                    series: None,
                    series_number: None,
                    genre: None,
                    tags: None,
                    language: None,
                    explicit: None,
                    abridged: None,
                })
            }
            _ => {
                Err(format!("Unsupported file type: {}", extension))
            }
        }
    } else if path.is_dir() {
        // Directory of audio files (MP3, etc.)
        // Check if directory contains audio files
        let mut audio_files = Vec::new();
        if let Ok(entries) = std::fs::read_dir(path) {
            for entry in entries.flatten() {
                if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                    let ext_lower = ext.to_lowercase();
                    if matches!(ext_lower.as_str(), "mp3" | "aac" | "wav" | "flac" | "m4b" | "m4a") {
                        audio_files.push(entry.path());
                    }
                }
            }
        }
        
        if audio_files.is_empty() {
            return Err("Directory does not contain any audio files".to_string());
        }
        
        // Sort files naturally
        audio_files.sort();
        
        Ok(BookMetadata {
            title: path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            subtitle: None,
            author: "Unknown Author".to_string(),
            isbn: None,
            asin: None,
            description: Some(format!("Directory with {} audio files", audio_files.len())),
            cover_url: None,
            duration: None,
            narrator: None,
            publisher: None,
            publish_year: None,
            series: None,
            series_number: None,
            genre: None,
            tags: None,
            language: None,
            explicit: None,
            abridged: None,
        })
    } else {
        Err("Path is neither a file nor a directory".to_string())
    }
}

// Helper function to get audio files from a directory
pub fn get_audio_files_from_directory(dir_path: &str) -> Vec<String> {
    let mut audio_files = Vec::new();
    let path = Path::new(dir_path);
    
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            if let Some(ext) = entry.path().extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if matches!(ext_lower.as_str(), "mp3" | "aac" | "wav" | "flac" | "m4b" | "m4a") {
                    if let Some(path_str) = entry.path().to_str() {
                        audio_files.push(path_str.to_string());
                    }
                }
            }
        }
    }
    
    // Sort files naturally (alphabetically)
    audio_files.sort();
    audio_files
}

// Get audio file duration using ffprobe (returns milliseconds)
pub fn get_audio_file_duration(file_path: &str) -> Result<u64, String> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            file_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}. Is ffprobe installed?", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe failed: {}", stderr));
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe JSON: {}", e))?;
    
    // Get duration from format.duration (decimal seconds)
    let duration_sec = json.get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .ok_or_else(|| "No duration field in ffprobe output".to_string())?;
    
    // Convert decimal seconds to milliseconds
    let duration_sec_f64: f64 = duration_sec.parse()
        .map_err(|e| format!("Failed to parse duration '{}': {}", duration_sec, e))?;
    
    let duration_ms = (duration_sec_f64 * 1000.0).round() as u64;
    
    println!("[DEBUG] File '{}' duration: {} seconds = {} ms", file_path, duration_sec, duration_ms);
    Ok(duration_ms)
}

// Extract chapters from audio file using ffprobe
pub fn extract_chapters_from_file(file_path: &str) -> Result<Vec<Chapter>, String> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_chapters",
            file_path,
        ])
        .output()
        .map_err(|e| format!("Failed to execute ffprobe: {}. Is ffprobe installed?", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("ffprobe failed: {}", stderr));
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Failed to parse ffprobe JSON: {}", e))?;
    
    let chapters_array = json.get("chapters")
        .and_then(|c| c.as_array())
        .ok_or_else(|| "No chapters found in file".to_string())?;
    
    let mut chapters = Vec::new();
    
    for chapter_json in chapters_array {
        // Get time_base (e.g., "1/1000" or "1/1000000")
        let time_base = chapter_json.get("time_base")
            .and_then(|t| t.as_str())
            .unwrap_or("1/1000");
        
        // Parse time_base (e.g., "1/1000" -> numerator=1, denominator=1000)
        let (numerator, denominator) = if let Some(slash_pos) = time_base.find('/') {
            let num_str = &time_base[..slash_pos];
            let den_str = &time_base[slash_pos + 1..];
            (
                num_str.parse::<u64>().unwrap_or(1),
                den_str.parse::<u64>().unwrap_or(1000)
            )
        } else {
            (1, 1000) // Default
        };
        
        // Get start and end in time_base units
        let start = chapter_json.get("start")
            .and_then(|s| s.as_u64())
            .unwrap_or(0);
        let end = chapter_json.get("end")
            .and_then(|e| e.as_u64())
            .unwrap_or(0);
        
        // Convert to milliseconds
        // Formula: ms = (value * numerator * 1000) / denominator
        let start_ms = (start * numerator * 1000) / denominator;
        let end_ms = (end * numerator * 1000) / denominator;
        let duration_ms = end_ms.saturating_sub(start_ms);
        
        // Get chapter title
        let title = chapter_json.get("tags")
            .and_then(|t| t.get("title"))
            .and_then(|title| title.as_str())
            .unwrap_or("Untitled Chapter")
            .to_string();
        
        chapters.push(Chapter {
            title,
            start_time: start_ms,
            duration: duration_ms,
            is_locked: false,
        });
    }
    
    println!("[DEBUG] Extracted {} chapters from file: {}", chapters.len(), file_path);
    Ok(chapters)
}

// Generate chapters from multiple files (one chapter per file)
pub fn generate_chapters_from_files(files: &[String]) -> Result<Vec<Chapter>, String> {
    let mut chapters = Vec::new();
    let mut cumulative_time = 0u64;
    
    for (index, file_path) in files.iter().enumerate() {
        // Get file name for chapter title
        let file_name = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&format!("Chapter {}", index + 1))
            .to_string();
        
        // Get actual duration using ffprobe
        let duration_ms = match get_audio_file_duration(file_path) {
            Ok(duration) => duration,
            Err(e) => {
                println!("[WARNING] Failed to get duration for '{}': {}. Using placeholder.", file_path, e);
                // Fallback to placeholder if ffprobe fails
                3600000u64 // 1 hour placeholder
            }
        };
        
        chapters.push(Chapter {
            title: file_name,
            start_time: cumulative_time,
            duration: duration_ms,
            is_locked: false,
        });
        
        cumulative_time += duration_ms;
    }
    
    println!("[DEBUG] Generated {} chapters from {} files (total duration: {} ms)", 
        chapters.len(), files.len(), cumulative_time);
    Ok(chapters)
}

// Validate chapters for gaps, overlaps, and duration issues
pub fn validate_chapters(chapters: &[Chapter], total_duration_ms: Option<u64>) -> Vec<String> {
    let mut errors = Vec::new();
    
    if chapters.is_empty() {
        return errors; // Empty is valid
    }
    
    // Check chronological order and overlaps
    for i in 0..chapters.len() {
        let current = &chapters[i];
        
        // Check for negative start time
        if current.start_time == 0 && i > 0 {
            // First chapter can start at 0, but check if previous ended before this
            let prev = &chapters[i - 1];
            if prev.start_time + prev.duration > current.start_time {
                errors.push(format!("Chapter {} overlaps with previous chapter", i + 1));
            }
        }
        
        // Check if chapter exceeds total duration
        if let Some(total) = total_duration_ms {
            if current.start_time > total {
                errors.push(format!("Chapter {} starts after file ends", i + 1));
            }
            if current.start_time + current.duration > total {
                errors.push(format!("Chapter {} extends beyond file end", i + 1));
            }
        }
        
        // Check for gaps (warn, not error)
        if i > 0 {
            let prev = &chapters[i - 1];
            let expected_start = prev.start_time + prev.duration;
            if current.start_time > expected_start {
                let gap_ms = current.start_time - expected_start;
                let gap_sec = gap_ms as f64 / 1000.0;
                if gap_sec > 1.0 { // Only warn for gaps > 1 second
                    errors.push(format!("Gap of {:.1}s between chapters {} and {}", 
                        gap_sec, i, i + 1));
                }
            } else if current.start_time < expected_start {
                errors.push(format!("Chapter {} overlaps with previous chapter", i + 1));
            }
        }
        
        // Check for zero or negative duration
        if current.duration == 0 {
            errors.push(format!("Chapter {} has zero duration", i + 1));
        }
    }
    
    errors
}

// Shift individual chapter with ripple effect
pub fn shift_chapter_with_ripple(chapters: &mut [Chapter], index: usize, new_start_ms: u64) -> Result<(), String> {
    if index >= chapters.len() {
        return Err("Chapter index out of range".to_string());
    }
    
    let old_start = chapters[index].start_time;
    let offset = new_start_ms as i64 - old_start as i64;
    
    // Don't allow negative start times
    if new_start_ms > old_start && offset > 0 {
        // Moving forward - shift this and all subsequent unlocked chapters
        for i in index..chapters.len() {
            if !chapters[i].is_locked {
                chapters[i].start_time = (chapters[i].start_time as i64 + offset).max(0) as u64;
            }
        }
    } else if offset < 0 {
        // Moving backward - only shift if not locked and doesn't create overlap
        if !chapters[index].is_locked {
            // Check if this would overlap with previous chapter
            if index > 0 {
                let prev_end = chapters[index - 1].start_time + chapters[index - 1].duration;
                if new_start_ms < prev_end {
                    return Err(format!("Cannot shift chapter {}: would overlap with previous chapter", index + 1));
                }
            }
            
            chapters[index].start_time = new_start_ms;
            
            // Shift subsequent chapters to maintain spacing
            let new_duration = chapters[index].start_time + chapters[index].duration;
            for i in (index + 1)..chapters.len() {
                if !chapters[i].is_locked {
                    let old_gap = chapters[i].start_time as i64 - new_duration as i64;
                    chapters[i].start_time = (new_duration as u64).saturating_add(old_gap.max(0) as u64);
                }
            }
        }
    }
    
    Ok(())
}

// Play a chapter from an audio file (headless, no window)
// Uses mpv, paplay, or ffmpeg piped to aplay
pub fn play_chapter_headless(file_path: &str, start_time_ms: u64, duration_ms: Option<u64>) -> Result<std::process::Child, String> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(format!("Audio file not found: {}", file_path));
    }
    
    // Convert milliseconds to seconds for player
    let start_sec = start_time_ms as f64 / 1000.0;
    let duration_sec = duration_ms.map(|d| d as f64 / 1000.0);
    
    // Try mpv first (best for headless playback)
    let mut mpv_args = vec![
        "--start".to_string(),
        format!("{:.3}", start_sec),
        "--no-video".to_string(),
        "--no-terminal".to_string(), // No terminal output
        "--really-quiet".to_string(), // Suppress all output
        file_path.to_string(),
    ];
    
    if let Some(dur) = duration_sec {
        mpv_args.push("--length".to_string());
        mpv_args.push(format!("{:.3}", dur));
    }
    
    let mpv_result = Command::new("mpv")
        .args(&mpv_args)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .stdin(Stdio::null())
        .spawn();
    
    if let Ok(child) = mpv_result {
        println!("[DEBUG] Playing chapter from {} at {:.3}s using mpv (headless)", file_path, start_sec);
        return Ok(child);
    }
    
    // Fallback: Use ffmpeg piped to paplay (PulseAudio)
    let mut ffmpeg_args = vec![
        "-ss".to_string(),
        format!("{:.3}", start_sec),
        "-i".to_string(),
        file_path.to_string(),
        "-f".to_string(),
        "wav".to_string(), // Output format
        "-".to_string(), // Output to stdout
    ];
    
    if let Some(dur) = duration_sec {
        ffmpeg_args.push("-t".to_string());
        ffmpeg_args.push(format!("{:.3}", dur));
    }
    
    let ffmpeg_child = Command::new("ffmpeg")
        .args(&ffmpeg_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    
    if let Ok(mut ffmpeg) = ffmpeg_child {
        if let Some(stdout) = ffmpeg.stdout.take() {
            let paplay_result = Command::new("paplay")
                .stdin(stdout)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            if let Ok(paplay_child) = paplay_result {
                println!("[DEBUG] Playing chapter from {} at {:.3}s using ffmpeg+paplay (headless)", file_path, start_sec);
                // Return the paplay process (ffmpeg will finish when paplay finishes)
                return Ok(paplay_child);
            }
        }
    }
    
    // Last fallback: Try aplay (ALSA)
    let mut ffmpeg_args2 = vec![
        "-ss".to_string(),
        format!("{:.3}", start_sec),
        "-i".to_string(),
        file_path.to_string(),
        "-f".to_string(),
        "wav".to_string(),
        "-".to_string(),
    ];
    
    if let Some(dur) = duration_sec {
        ffmpeg_args2.push("-t".to_string());
        ffmpeg_args2.push(format!("{:.3}", dur));
    }
    
    let ffmpeg_child2 = Command::new("ffmpeg")
        .args(&ffmpeg_args2)
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn();
    
    if let Ok(mut ffmpeg) = ffmpeg_child2 {
        if let Some(stdout) = ffmpeg.stdout.take() {
            let aplay_result = Command::new("aplay")
                .stdin(stdout)
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .spawn();
            
            if let Ok(aplay_child) = aplay_result {
                println!("[DEBUG] Playing chapter from {} at {:.3}s using ffmpeg+aplay (headless)", file_path, start_sec);
                return Ok(aplay_child);
            }
        }
    }
    
    Err("No headless audio player found. Please install mpv, or ffmpeg with paplay/aplay.".to_string())
}

// Find the audio file to play for a given chapter and calculate the correct start time
// Returns (file_path, start_time_ms_in_file)
// For single files: start_time is the chapter's start_time
// For multi-file: start_time is 0 (always start from beginning of the file)
pub fn find_audio_file_for_chapter(
    selected_file_path: Option<&String>,
    audio_file_paths: &[String],
    chapter_index: usize,
    chapter_start_time_ms: u64,
) -> Option<(String, u64)> {
    // If single file, use that file and the chapter's start time
    if let Some(file_path) = selected_file_path {
        if Path::new(file_path).is_file() {
            return Some((file_path.clone(), chapter_start_time_ms));
        }
    }
    
    // If multi-file, find the file that contains this chapter
    // For multi-file books, each file is one chapter, so start at 0
    if !audio_file_paths.is_empty() {
        if chapter_index < audio_file_paths.len() {
            // For multi-file, always start at 0 (beginning of the file)
            return Some((audio_file_paths[chapter_index].clone(), 0));
        }
        // Fallback to first file
        return Some((audio_file_paths[0].clone(), 0));
    }
    
    None
}
