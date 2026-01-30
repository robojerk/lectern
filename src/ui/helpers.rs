use crate::models::BookMetadata;
use std::path::{Path, PathBuf};

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

/// Look for a local cover image in an audiobook directory.
/// Checks common names (folder.jpg, cover.jpg, etc.) and returns the first found path.
pub fn find_local_cover_in_directory(dir_path: &str) -> Option<String> {
    let path = Path::new(dir_path);
    if !path.is_dir() {
        return None;
    }
    let base_names = [
        "folder.jpg",
        "folder.jpeg",
        "folder.png",
        "cover.jpg",
        "cover.jpeg",
        "cover.png",
        "AlbumArt.jpg",
        "AlbumArt.jpeg",
        "AlbumArt.png",
    ];
    for name in base_names {
        let candidate = path.join(name);
        if candidate.exists() {
            if let Some(s) = candidate.to_str() {
                return Some(s.to_string());
            }
        }
    }
    None
}

/// Look for metadata or chapter timestamp files in an audiobook directory.
/// Returns a list of (filename, full_path) for found files. Does not read contents.
pub fn find_metadata_or_chapter_files(dir_path: &str) -> Vec<(String, String)> {
    let path = Path::new(dir_path);
    if !path.is_dir() {
        return Vec::new();
    }
    let mut found = Vec::new();
    let names = [
        "chapters.txt",
        "Chapters.txt",
        "chapter.txt",
        "metadata.json",
        "metadata.xml",
        "book.json",
        "chapters.json",
        "description.txt",
        "info.txt",
    ];
    for name in names {
        let candidate = path.join(name);
        if candidate.is_file() {
            if let (Some(name_os), Some(path_str)) = (candidate.file_name(), candidate.to_str()) {
                if let Some(n) = name_os.to_str() {
                    found.push((n.to_string(), path_str.to_string()));
                }
            }
        }
    }
    // Common extensions for metadata/chapter files (single file per name)
    if let Ok(entries) = std::fs::read_dir(path) {
        for entry in entries.flatten() {
            let p = entry.path();
            if p.is_file() {
                if let Some(ext) = p.extension() {
                    let ext_lower = ext.to_string_lossy().to_lowercase();
                    if matches!(ext_lower.as_str(), "nfo" | "sfv" | "cue") {
                        if let (Some(name_os), Some(path_str)) = (p.file_name(), p.to_str()) {
                            if let Some(n) = name_os.to_str() {
                                if !found.iter().any(|(_, fp)| fp == path_str) {
                                    found.push((n.to_string(), path_str.to_string()));
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    found.sort_by(|a, b| a.0.cmp(&b.0));
    found
}

/// Sanitize a string for use in a path component (no slashes or backslashes).
fn sanitize_path_component(s: &str) -> String {
    s.replace(['/', '\\'], "-").trim().to_string()
}

/// Collapse repeated path separators and trim leading/trailing slashes.
/// So {Author}/{Series}/{Title}.m4b with empty Series becomes Author/Title.m4b.
fn normalize_path_slashes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut prev_slash = false;
    for c in s.chars() {
        let is_sep = c == '/' || c == std::path::MAIN_SEPARATOR;
        if is_sep {
            if !prev_slash {
                out.push(std::path::MAIN_SEPARATOR);
            }
            prev_slash = true;
        } else {
            out.push(c);
            prev_slash = false;
        }
    }
    out.trim_matches(std::path::MAIN_SEPARATOR).to_string()
}

/// Expand {SeriesNumber} and {SeriesNumberX} (X = single char suffix, only when non-empty).
/// E.g. {SeriesNumber-} → "4-", {SeriesNumber.} → "4.", {SeriesNumber } → "4 ".
fn expand_series_number(template: &str, series_number_s: &str) -> String {
    const TAG: &str = "{SeriesNumber";
    let mut result = template.to_string();
    while let Some(i) = result.find(TAG) {
        let after = &result[i + TAG.len()..];
        if after.is_empty() {
            break;
        }
        let mut it = after.chars();
        let first = it.next().unwrap();
        if first == '}' {
            result.replace_range(i..i + TAG.len() + 1, series_number_s);
        } else if it.next() == Some('}') {
            let suffix_char = first;
            let end = i + TAG.len() + suffix_char.len_utf8() + 1; // TAG + suffix char + '}'
            let replacement = if series_number_s.is_empty() {
                String::new()
            } else {
                format!("{}{}", series_number_s, suffix_char)
            };
            result.replace_range(i..end, &replacement);
        } else {
            result.replace_range(i..i + TAG.len() + 1, series_number_s);
        }
    }
    result
}

/// Expand media management template with metadata and join to library path.
/// Placeholders: {Author}, {Title}, {Series}, {SeriesNumber}, {Year}, {Genre}, {ASIN}, {Language}, {Tags}
/// Optional suffix on SeriesNumber: {SeriesNumber-} → "4-", {SeriesNumber.} → "4.", {SeriesNumber } → "4 "
/// (suffix only when value is non-empty). PathBuf used for correct separators.
pub fn apply_media_template(
    template: &str,
    lib_path: &str,
    title: &str,
    author: &str,
    series: &str,
    series_number: &str,
    year: &str,
    genre: &str,
    asin: &str,
    language: &str,
    tags: &str,
) -> String {
    let author_s = sanitize_path_component(if author.is_empty() { "Unknown Author" } else { author });
    let title_s = sanitize_path_component(if title.is_empty() { "Unknown Title" } else { title });
    let series_s = sanitize_path_component(series);
    let series_number_s = sanitize_path_component(series_number);
    let year_s = sanitize_path_component(year);
    let genre_s = sanitize_path_component(genre);
    let asin_s = sanitize_path_component(asin);
    let language_s = sanitize_path_component(language);
    let tags_s = sanitize_path_component(tags);

    let expanded = expand_series_number(template, &series_number_s)
        .replace("{Author}", &author_s)
        .replace("{Title}", &title_s)
        .replace("{Series}", &series_s)
        .replace("{SeriesNumber}", &series_number_s)
        .replace("{Year}", &year_s)
        .replace("{Genre}", &genre_s)
        .replace("{ASIN}", &asin_s)
        .replace("{Language}", &language_s)
        .replace("{Tags}", &tags_s);

    let expanded = normalize_path_slashes(&expanded);

    let lib = lib_path.trim_end_matches('/').trim_end_matches('\\');
    if lib.is_empty() {
        expanded
    } else {
        PathBuf::from(lib)
            .join(&expanded)
            .to_string_lossy()
            .to_string()
    }
}
