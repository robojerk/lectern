use crate::models::BookMetadata;
use std::path::Path;

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
                Ok(BookMetadata {
                    title: path.file_stem()
                        .and_then(|s| s.to_str())
                        .unwrap_or("Unknown")
                        .to_string(),
                    ..Default::default()
                })
            }
            _ => {
                Err(format!("Unsupported file type: {}", extension))
            }
        }
    } else if path.is_dir() {
        // Directory of audio files (MP3, etc.)
        let audio_files = get_audio_files_from_directory(path_str);
        
        if audio_files.is_empty() {
            return Err("Directory does not contain any audio files".to_string());
        }
        
        Ok(BookMetadata {
            title: path.file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown")
                .to_string(),
            description: Some(format!("Directory with {} audio files", audio_files.len())),
            ..Default::default()
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
