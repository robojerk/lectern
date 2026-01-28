use std::process::Command;
use crate::models::chapters::Chapter;
use std::path::Path;
use anyhow::{Result, anyhow};

// Get audio file duration using ffprobe (returns milliseconds)
pub fn get_audio_file_duration(file_path: &str) -> Result<u64> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            file_path,
        ])
        .output()
        .map_err(|e| anyhow!("Failed to execute ffprobe: {}. Is ffprobe installed?", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffprobe failed: {}", stderr));
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("Failed to parse ffprobe JSON: {}", e))?;
    
    // Get duration from format.duration (decimal seconds)
    let duration_sec = json.get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .ok_or_else(|| anyhow!("No duration field in ffprobe output"))?;
    
    // Convert decimal seconds to milliseconds
    let duration_sec_f64: f64 = duration_sec.parse()
        .map_err(|e| anyhow!("Failed to parse duration '{}': {}", duration_sec, e))?;
    
    let duration_ms = (duration_sec_f64 * 1000.0).round() as u64;
    
    Ok(duration_ms)
}

// Extract chapters from audio file using ffprobe
pub fn extract_chapters_from_file(file_path: &str) -> Result<Vec<Chapter>> {
    let output = Command::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_chapters",
            file_path,
        ])
        .output()
        .map_err(|e| anyhow!("Failed to execute ffprobe: {}. Is ffprobe installed?", e))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow!("ffprobe failed: {}", stderr));
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .map_err(|e| anyhow!("Failed to parse ffprobe JSON: {}", e))?;
    
    let chapters_array = json.get("chapters")
        .and_then(|c| c.as_array())
        .ok_or_else(|| anyhow!("No chapters found in file"))?;
    
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
        
        chapters.push(Chapter::new(title, start_ms, duration_ms));
    }
    
    Ok(chapters)
}

// Generate chapters from multiple files (one chapter per file)
pub fn generate_chapters_from_files(files: &[String]) -> Result<Vec<Chapter>> {
    let mut chapters = Vec::new();
    let mut cumulative_time = 0u64;
    
    for (index, file_path) in files.iter().enumerate() {
        let file_name = Path::new(file_path)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or(&format!("Chapter {}", index + 1))
            .to_string();
        
        let duration_ms = get_audio_file_duration(file_path)?;
        
        chapters.push(Chapter::new(file_name, cumulative_time, duration_ms));
        cumulative_time += duration_ms;
    }
    
    Ok(chapters)
}
