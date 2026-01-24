// src/models/chapter.rs
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
}