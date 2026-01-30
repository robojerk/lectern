//! Parse chapter lists from external files (txt, json, cue, ini).
//! Used when the user selects a directory; the app looks for chapter files automatically.

use crate::models::chapters::Chapter;
use std::path::Path;
use std::fs;

/// Try to parse chapters from a file. Detects format by extension.
pub fn parse_chapters_from_path(path: &str) -> Result<Vec<Chapter>, String> {
    let p = Path::new(path);
    if !p.is_file() {
        return Err("Not a file".to_string());
    }
    let content = fs::read_to_string(path).map_err(|e| e.to_string())?;
    let ext = p
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();
    match ext.as_str() {
        "txt" => parse_txt(&content),
        "json" => parse_json(&content),
        "cue" => parse_cue(&content),
        "ini" => parse_ini(&content),
        _ => Err(format!("Unknown chapter file extension: {}", ext)),
    }
}

/// Check if a filename looks like a chapter file we can parse.
pub fn is_chapter_file_name(name: &str) -> bool {
    let lower = name.to_lowercase();
    lower == "chapters.txt"
        || lower == "chapter.txt"
        || lower == "chapters.json"
        || lower == "chapter.json"
        || lower.ends_with(".cue")
        || lower == "chapters.ini"
        || lower == "chapter.ini"
}

/// Parse "HH:MM:SS Title" or "MM:SS Title" or "M:SS Title" per line.
fn parse_txt(content: &str) -> Result<Vec<Chapter>, String> {
    let mut chapters = Vec::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        // Find where the timestamp ends (digits, colons, optional decimal)
        let mut i = 0;
        let bytes = line.as_bytes();
        while i < bytes.len() {
            let c = bytes[i] as char;
            if c.is_ascii_digit() || c == ':' || c == '.' {
                i += 1;
            } else if c == ' ' || c == '\t' {
                break;
            } else {
                return Err(format!("Invalid timestamp in line: {}", line));
            }
        }
        let time_str = line[..i].trim_end_matches(|c| c == ':' || c == '.');
        let title = line[i..].trim();
        if title.is_empty() {
            continue;
        }
        let start_ms = parse_timestamp_to_ms(time_str)?;
        // Duration: until next chapter or 0 (caller may adjust)
        let duration_ms = 0u64;
        chapters.push(Chapter::new(title.to_string(), start_ms, duration_ms));
    }
    // Fill durations: each chapter's duration = next start - this start
    for i in 0..chapters.len() {
        let end_ms = if i + 1 < chapters.len() {
            chapters[i + 1].start_time
        } else {
            chapters[i].start_time + chapters[i].duration
        };
        chapters[i].duration = end_ms.saturating_sub(chapters[i].start_time);
    }
    if chapters.is_empty() {
        return Err("No chapters found in file".to_string());
    }
    Ok(chapters)
}

fn parse_timestamp_to_ms(s: &str) -> Result<u64, String> {
    let parts: Vec<&str> = s.split(':').collect();
    let (h, m, sec) = match parts.len() {
        1 => {
            // seconds only
            let sec: f64 = parts[0].trim().parse().map_err(|_| format!("Invalid number: {}", parts[0]))?;
            (0u64, 0u64, sec)
        }
        2 => {
            let m: u64 = parts[0].trim().parse().map_err(|_| format!("Invalid minutes: {}", parts[0]))?;
            let sec: f64 = parts[1].trim().parse().map_err(|_| format!("Invalid seconds: {}", parts[1]))?;
            (0u64, m, sec)
        }
        3 => {
            let h: u64 = parts[0].trim().parse().map_err(|_| format!("Invalid hours: {}", parts[0]))?;
            let m: u64 = parts[1].trim().parse().map_err(|_| format!("Invalid minutes: {}", parts[1]))?;
            let sec: f64 = parts[2].trim().parse().map_err(|_| format!("Invalid seconds: {}", parts[2]))?;
            (h, m, sec)
        }
        _ => return Err(format!("Invalid timestamp: {}", s)),
    };
    let total_sec = (h * 3600) as f64 + (m * 60) as f64 + sec;
    Ok((total_sec * 1000.0).round() as u64)
}

/// Parse JSON array: [{ "title": "...", "start_ms": n }] or "start" in seconds.
fn parse_json(content: &str) -> Result<Vec<Chapter>, String> {
    let arr: Vec<serde_json::Value> = serde_json::from_str(content)
        .map_err(|e| format!("Invalid JSON: {}", e))?;
    let mut chapters = Vec::new();
    for (i, obj) in arr.iter().enumerate() {
        let title = obj
            .get("title")
            .or_else(|| obj.get("name"))
            .and_then(|t| t.as_str())
            .unwrap_or(&format!("Chapter {}", i + 1))
            .to_string();
        let start_ms = obj
            .get("start_ms")
            .and_then(|s| s.as_u64())
            .or_else(|| obj.get("start_time").and_then(|s| s.as_u64()))
            .or_else(|| {
                obj.get("start")
                    .and_then(|s| s.as_f64())
                    .map(|s| (s * 1000.0).round() as u64)
            })
            .unwrap_or(0);
        let duration_ms = obj
            .get("duration_ms")
            .and_then(|d| d.as_u64())
            .or_else(|| {
                obj.get("duration")
                    .and_then(|d| d.as_f64())
                    .map(|d| (d * 1000.0).round() as u64)
            })
            .unwrap_or(0);
        chapters.push(Chapter::new(title, start_ms, duration_ms));
    }
    if chapters.is_empty() {
        return Err("No chapters in JSON array".to_string());
    }
    // If durations missing, fill from next start
    for i in 0..chapters.len() {
        if chapters[i].duration == 0 && i + 1 < chapters.len() {
            chapters[i].duration = chapters[i + 1].start_time.saturating_sub(chapters[i].start_time);
        }
    }
    Ok(chapters)
}

/// CUE: TRACK nn TITLE "..." and INDEX 01 mm:ss:ff (75 frames per second).
fn parse_cue(content: &str) -> Result<Vec<Chapter>, String> {
    let mut tracks: Vec<(u64, String)> = Vec::new();
    let mut current_title = String::new();
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("TRACK ") {
            // TRACK 01 AUDIO ... then TITLE "Chapter 1" on next line(s)
            current_title.clear();
        } else if line.starts_with("TITLE ") {
            let rest = line["TITLE ".len()..].trim();
            current_title = rest.trim_matches('"').to_string();
        } else if line.starts_with("INDEX 01 ") {
            let rest = line["INDEX 01 ".len()..].trim();
            // mm:ss:ff (75 frames per second)
            let ms = parse_cue_index_to_ms(rest)?;
            let title = if current_title.is_empty() {
                format!("Chapter {}", tracks.len() + 1)
            } else {
                current_title.clone()
            };
            tracks.push((ms, title));
        }
    }
    if tracks.is_empty() {
        return Err("No INDEX 01 entries in CUE file".to_string());
    }
    let mut chapters = Vec::new();
    for (i, (start_ms, title)) in tracks.iter().enumerate() {
        let duration_ms = if i + 1 < tracks.len() {
            tracks[i + 1].0.saturating_sub(*start_ms)
        } else {
            0
        };
        chapters.push(Chapter::new(title.clone(), *start_ms, duration_ms));
    }
    Ok(chapters)
}

fn parse_cue_index_to_ms(s: &str) -> Result<u64, String> {
    let parts: Vec<&str> = s.split(':').collect();
    if parts.len() != 3 {
        return Err(format!("CUE INDEX must be mm:ss:ff, got: {}", s));
    }
    let m: u64 = parts[0].parse().map_err(|_| format!("Invalid minutes: {}", parts[0]))?;
    let sec: u64 = parts[1].parse().map_err(|_| format!("Invalid seconds: {}", parts[1]))?;
    let frames: u64 = parts[2].parse().map_err(|_| format!("Invalid frames: {}", parts[2]))?;
    // 75 frames per second
    let total_ms = (m * 60 + sec) * 1000 + (frames * 1000 / 75);
    Ok(total_ms)
}

/// INI-style: [Chapter N] or [CHAPTER] with start= or start_time= (seconds or HH:MM:SS).
fn parse_ini(content: &str) -> Result<Vec<Chapter>, String> {
    let mut chapters = Vec::new();
    let mut current_title = String::new();
    let mut current_start_ms: Option<u64> = None;

    fn flush(
        chapters: &mut Vec<Chapter>,
        title: &str,
        start_ms: Option<u64>,
    ) {
        if let Some(ms) = start_ms {
            let t = if title.is_empty() {
                format!("Chapter {}", chapters.len() + 1)
            } else {
                title.to_string()
            };
            let duration_ms = 0u64;
            chapters.push(Chapter::new(t, ms, duration_ms));
        }
    }

    for line in content.lines() {
        let line = line.trim();
        if line.starts_with('[') && line.contains(']') {
            flush(&mut chapters, &current_title, current_start_ms.take());
            let inside = line.trim_start_matches('[').split(']').next().unwrap_or("").trim();
            current_title = inside.to_string();
        } else if line.to_lowercase().starts_with("start=") {
            let v = line["start=".len()..].trim();
            current_start_ms = Some(parse_ini_start(v)?);
        } else if line.to_lowercase().starts_with("start_time=") {
            let v = line["start_time=".len()..].trim();
            current_start_ms = Some(parse_ini_start(v)?);
        }
    }
    flush(&mut chapters, &current_title, current_start_ms.take());

    if chapters.is_empty() {
        return Err("No chapters in INI file".to_string());
    }
    for i in 0..chapters.len() {
        if chapters[i].duration == 0 && i + 1 < chapters.len() {
            chapters[i].duration = chapters[i + 1].start_time.saturating_sub(chapters[i].start_time);
        }
    }
    Ok(chapters)
}

fn parse_ini_start(v: &str) -> Result<u64, String> {
    if v.contains(':') {
        parse_timestamp_to_ms(v)
    } else {
        let sec: f64 = v.parse().map_err(|_| format!("Invalid number: {}", v))?;
        Ok((sec * 1000.0).round() as u64)
    }
}
