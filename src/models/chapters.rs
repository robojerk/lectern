// src/models/chapters.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start_time: u64,  // milliseconds
    pub duration: u64,    // milliseconds
    pub is_locked: bool,
}

impl Chapter {
    pub fn new(title: String, start_time: u64, duration: u64) -> Self {
        Self {
            title,
            start_time,
            duration,
            is_locked: false,
        }
    }

    /// Validate chapters for gaps, overlaps, and duration issues
    pub fn validate_list(chapters: &[Chapter], total_duration_ms: Option<u64>) -> Vec<String> {
        let mut errors = Vec::new();
        
        if chapters.is_empty() {
            return errors;
        }
        
        for i in 0..chapters.len() {
            let current = &chapters[i];
            
            if let Some(total) = total_duration_ms {
                if current.start_time > total {
                    errors.push(format!("Chapter {} starts after file ends", i + 1));
                }
                if current.start_time + current.duration > total {
                    errors.push(format!("Chapter {} extends beyond file end", i + 1));
                }
            }
            
            if i > 0 {
                let prev = &chapters[i - 1];
                let expected_start = prev.start_time + prev.duration;
                if current.start_time > expected_start {
                    let gap_sec = (current.start_time - expected_start) as f64 / 1000.0;
                    if gap_sec > 1.0 {
                        errors.push(format!("Gap of {:.1}s between chapters {} and {}", gap_sec, i, i + 1));
                    }
                } else if current.start_time < expected_start {
                    errors.push(format!("Chapter {} overlaps with previous chapter", i + 1));
                }
            }
            
            if current.duration == 0 {
                errors.push(format!("Chapter {} has zero duration", i + 1));
            }
        }
        
        errors
    }

    /// Shift individual chapter with ripple effect
    pub fn shift_with_ripple(chapters: &mut [Chapter], index: usize, new_start_ms: u64) -> Result<(), String> {
        if index >= chapters.len() {
            return Err("Chapter index out of range".to_string());
        }
        
        let old_start = chapters[index].start_time;
        let offset = new_start_ms as i64 - old_start as i64;
        
        if new_start_ms > old_start && offset > 0 {
            // Moving forward
            for i in index..chapters.len() {
                if !chapters[i].is_locked {
                    chapters[i].start_time = (chapters[i].start_time as i64 + offset).max(0) as u64;
                }
            }
        } else if offset < 0 {
            // Moving backward
            if !chapters[index].is_locked {
                if index > 0 {
                    let prev_end = chapters[index - 1].start_time + chapters[index - 1].duration;
                    if new_start_ms < prev_end {
                        return Err(format!("Cannot shift chapter {}: would overlap with previous chapter", index + 1));
                    }
                }
                
                chapters[index].start_time = new_start_ms;
                
                let new_end = chapters[index].start_time + chapters[index].duration;
                for i in (index + 1)..chapters.len() {
                    if !chapters[i].is_locked {
                        let old_gap = chapters[i].start_time as i64 - (old_start as i64 + chapters[index].duration as i64);
                        chapters[i].start_time = (new_end as u64).saturating_add(old_gap.max(0) as u64);
                    }
                }
            }
        }
        
        Ok(())
    }
}