//! Centralized icon loading: light/dark chapter icons. Caches live on Lectern; do not call from view().
use iced::widget::image::Handle;
use std::collections::HashMap;

const ICON_SUFFIX_LIGHT: &str = "000000";
const ICON_SUFFIX_DARK: &str = "E3E3E3";

/// Icon logical names and their filename stem (before _24dp_XXXXX_...).
const CHAPTER_ICON_STEMS: &[(&str, &str)] = &[
    ("progress_activity", "progress_activity_24dp"),
    ("lock", "lock_24dp"),
    ("lock_open", "lock_open_right_24dp"),
    ("delete", "delete_24dp"),
    ("insert", "add_row_below_24dp"),
    ("play", "play_circle_24dp"),
    ("stop", "stop_circle_24dp"),
    ("remove", "remove_24dp"),
    ("add", "add_24dp"),
    ("error", "error_24dp"),
];

fn load_set(suffix: &str) -> HashMap<String, Handle> {
    let mut icons = HashMap::new();
    for (name, stem) in CHAPTER_ICON_STEMS {
        let path = format!("assets/png/{}_{}_FILL0_wght400_GRAD0_opsz24.png", stem, suffix);
        if let Ok(bytes) = std::fs::read(&path) {
            if let Ok(img) = ::image::load_from_memory(&bytes) {
                let rgba = img.to_rgba8();
                let (width, height) = rgba.dimensions();
                let pixels: Vec<u8> = rgba.into_raw();
                icons.insert((*name).to_string(), Handle::from_pixels(width, height, pixels));
            } else {
                eprintln!("[WARNING] Failed to decode icon {}", path);
            }
        } else {
            eprintln!("[WARNING] Icon file not found: {}", path);
        }
    }
    icons
}

/// Load both light and dark chapter icon sets. Call once at startup; store on Lectern.
pub fn load_chapter_icons_both() -> (HashMap<String, Handle>, HashMap<String, Handle>) {
    (load_set(ICON_SUFFIX_LIGHT), load_set(ICON_SUFFIX_DARK))
}
