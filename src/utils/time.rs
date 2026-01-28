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

// Helper function to parse time string (HH:MM:SS or HH:MM:SS.mmm) to milliseconds
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
        let ms_str = sec_parts.get(1).unwrap_or(&"0");
        let ms: u64 = ms_str.parse().unwrap_or(0);
        
        // Handle different precision (e.g., .5 -> 500ms, .50 -> 500ms, .500 -> 500ms)
        let ms_normalized = match ms_str.len() {
            1 => ms * 100,
            2 => ms * 10,
            3 => ms,
            _ => ms, // Just take it as is if more than 3
        };
        (secs, ms_normalized)
    } else {
        (seconds_part.parse().map_err(|_| "Invalid seconds")?, 0)
    };
    
    Ok((hours * 3600 + minutes * 60 + seconds) * 1000 + milliseconds)
}
