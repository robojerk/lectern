use gstreamer as gst;
use gst::prelude::*;
use std::sync::{Arc, Mutex};

pub struct LecternPlayer {
    playbin: gst::Element,
}

impl LecternPlayer {
    pub fn new() -> Self {
        gst::init().expect("Failed to initialize GStreamer");
        // playbin3 is the successor to playbin, better at handling modern containers
        let playbin = gst::ElementFactory::make("playbin3")
            .build()
            .expect("Failed to create playbin3. Is gst-plugins-base installed?");
        
        Self { playbin }
    }

    pub fn play_at(&self, path: &str, local_offset_ms: u64) {
        println!("🎵 Playing {} at {}ms", path, local_offset_ms);
        self.playbin.set_state(gst::State::Null).ok();
        
        // Ensure path is absolute and properly URI-encoded for GStreamer
        let uri = if path.starts_with("/") {
             format!("file://{}", path)
        } else {
             path.to_string()
        };
        
        self.playbin.set_property("uri", &uri);
        
        // We must reach at least the 'Paused' state before we can seek
        self.playbin.set_state(gst::State::Paused).ok();
        
        // Wait for state change to complete
        let _ = self.playbin.state(gst::ClockTime::from_seconds(2));

        let seek_pos = gst::ClockTime::from_mseconds(local_offset_ms);
        self.playbin.seek_simple(
            gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, 
            seek_pos
        ).ok();

        self.playbin.set_state(gst::State::Playing).ok();
    }

    pub fn stop(&self) {
        self.playbin.set_state(gst::State::Null).ok();
    }
}
