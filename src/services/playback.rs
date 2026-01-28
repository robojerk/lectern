use std::path::Path;
use tokio::process::Command as TokioCommand;
use anyhow::{Result, anyhow};
use crate::services::ffprobe::get_audio_file_duration;

// Play a chapter from an audio file (headless, no window)
// Uses mpv, ffplay, or ffmpeg (in that order)
// Returns tokio::process::Child for proper lifecycle management
pub async fn play_chapter_headless(file_path: &str, start_time_ms: u64, duration_ms: Option<u64>) -> Result<tokio::process::Child> {
    let path = Path::new(file_path);
    
    if !path.exists() {
        return Err(anyhow!("Audio file not found: {}", file_path));
    }
    
    // Convert milliseconds to seconds for player
    let start_sec = start_time_ms as f64 / 1000.0;
    let duration_sec = duration_ms.map(|d| d as f64 / 1000.0);
    
    // Try mpv first (best for headless playback)
    let mut mpv_cmd = TokioCommand::new("mpv");
    mpv_cmd
        .arg("--no-config") // Ignore user config for stability
        .arg("--no-resume-playback") // Don't try to resume from previous position
        .arg(format!("--start={:.3}", start_sec))
        .arg("--no-video")
        .arg("--audio-display=no")
        .arg("--ao=pulse,pipewire,alsa") // Explicitly try common Linux audio outputs
        .arg("--idle=no")
        .arg("--really-quiet") // Minimal output
        .stdout(std::process::Stdio::null()) // Don't pipe stdout (causes mpv to exit)
        .stderr(std::process::Stdio::null()) // Don't pipe stderr (causes mpv to exit)
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true); // Prevent zombie processes
    
    if let Some(dur) = duration_sec {
        mpv_cmd.arg(format!("--length={:.3}", dur));
    }
    
    mpv_cmd.arg(file_path);
    
    match mpv_cmd.spawn() {
        Ok(child) => return Ok(child),
        Err(_) => {}
    }
    
    // Try ffplay second
    let mut ffplay_cmd = TokioCommand::new("ffplay");
    ffplay_cmd
        .arg("-nodisp")
        .arg("-autoexit")
        .arg("-ss")
        .arg(&format!("{:.3}", start_sec))
        .arg("-vn")
        .arg("-loglevel")
        .arg("warning")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    
    if let Some(dur) = duration_sec {
        ffplay_cmd.arg("-t").arg(&format!("{:.3}", dur));
    }
    
    ffplay_cmd.arg(file_path);
    
    match ffplay_cmd.spawn() {
        Ok(child) => return Ok(child),
        Err(_) => {}
    }
    
    // Fallback: Use ffmpeg directly to ALSA
    let mut ffmpeg_cmd = TokioCommand::new("ffmpeg");
    ffmpeg_cmd
        .arg("-i")
        .arg(file_path)
        .arg("-ss")
        .arg(&format!("{:.3}", start_sec))
        .arg("-f")
        .arg("alsa")
        .arg("default")
        .arg("-loglevel")
        .arg("warning")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::piped())
        .stdin(std::process::Stdio::null())
        .kill_on_drop(true);
    
    if let Some(dur) = duration_sec {
        ffmpeg_cmd.arg("-t").arg(&format!("{:.3}", dur));
    }
    
    match ffmpeg_cmd.spawn() {
        Ok(child) => Ok(child),
        Err(e) => Err(anyhow!("No headless audio player found and ffmpeg fallback failed: {}", e)),
    }
}

// Find the audio file to play for a given chapter and calculate the correct start time
pub fn find_audio_file_for_chapter(
    selected_file_path: Option<&String>,
    audio_file_paths: &[String],
    chapter_start_time_ms: u64,
) -> Option<(String, u64)> {
    // If single file, use that file and the chapter's start time
    if let Some(file_path) = selected_file_path {
        if Path::new(file_path).is_file() {
            return Some((file_path.clone(), chapter_start_time_ms));
        }
    }
    
    // If multi-file, find the file that contains this chapter's start time
    if !audio_file_paths.is_empty() {
        let mut cumulative_time = 0u64;
        
        for file_path in audio_file_paths {
            let file_duration_ms = get_audio_file_duration(file_path).unwrap_or(0);
            let file_end_time = cumulative_time + file_duration_ms;
            
            if chapter_start_time_ms >= cumulative_time && chapter_start_time_ms < file_end_time {
                let offset_in_file_ms = chapter_start_time_ms - cumulative_time;
                return Some((file_path.clone(), offset_in_file_ms));
            }
            
            // Special case for exact end
            if chapter_start_time_ms == file_end_time {
                // We'll let it fall through to the next file if there is one, 
                // but if it's the very end of the last file, we'll return the last file.
            }
            
            cumulative_time = file_end_time;
        }
        
        // Final fallback for exact end of entire book
        if let Some(last_file) = audio_file_paths.last() {
            if let Ok(duration) = get_audio_file_duration(last_file) {
                 return Some((last_file.clone(), duration));
            }
        }
    }
    
    None
}
