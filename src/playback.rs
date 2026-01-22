use std::process::{Command, Child};


pub struct AudioPlayer {
    process: Option<Child>,
}

impl Default for AudioPlayer {
    fn default() -> Self {
        Self { process: None }
    }
}

impl AudioPlayer {
    pub fn new() -> Option<Self> {
        Some(Self::default())
    }

    pub fn play_file(&mut self, path: &str) {
        self.stop();
        
        let path = path.to_string();
        
        // Try ffplay first (part of ffmpeg which is required)
        let child = Command::new("ffplay")
            .arg("-nodisp")
            .arg("-autoexit")
            .arg("-ss")
            .arg("0")
            .arg(&path)
            .spawn()
            .or_else(|_| {
                // Fallback to mpv
                Command::new("mpv")
                    .arg("--no-video")
                    .arg(&path)
                    .spawn()
            });

        if let Ok(child) = child {
            self.process = Some(child);
        } else {
            eprintln!("Failed to start playback. Ensure ffplay or mpv is installed.");
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut child) = self.process.take() {
            let _ = child.kill(); // Kill the process
            let _ = child.wait(); // Clean up zombie
        }
    }
}
