//! Persist theme_id, dark_mode, and accent_override. Load on startup, save on change.
//! Uses XDG config on Linux: ~/.config/lectern/theme_settings.json
use crate::config;
use crate::ui::theme::ThemeId;
use iced::Color;
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ThemeSettingsFile {
    theme_id: String,
    dark_mode: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    accent_hex: Option<String>,
}

fn config_path() -> Option<std::path::PathBuf> {
    config::config_file("theme_settings.json")
}

fn color_to_hex(c: Color) -> String {
    let [r, g, b, _] = c.into_rgba8();
    format!("#{:02x}{:02x}{:02x}", r, g, b)
}

fn parse_hex(s: &str) -> Option<Color> {
    let s = s.trim().trim_start_matches('#');
    if s.len() != 6 {
        return None;
    }
    let r = u8::from_str_radix(&s[0..2], 16).ok()?;
    let g = u8::from_str_radix(&s[2..4], 16).ok()?;
    let b = u8::from_str_radix(&s[4..6], 16).ok()?;
    Some(Color::from_rgb(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
    ))
}

fn theme_id_to_string(t: ThemeId) -> String {
    t.name().to_string()
}

fn string_to_theme_id(s: &str) -> ThemeId {
    match s {
        "Nordic" => ThemeId::Nordic,
        "Breeze" => ThemeId::Breeze,
        "Candy" => ThemeId::Candy,
        "Windows 11" => ThemeId::Windows11,
        _ => ThemeId::default(),
    }
}

/// Load theme_id, dark_mode, accent_override from config file. Returns None on missing/error.
pub fn load() -> Option<(ThemeId, bool, Option<Color>)> {
    let path = config_path()?;
    let data = fs::read_to_string(&path).ok()?;
    let file: ThemeSettingsFile = serde_json::from_str(&data).ok()?;
    let accent = file.accent_hex.as_ref().and_then(|h| parse_hex(h));
    Some((
        string_to_theme_id(&file.theme_id),
        file.dark_mode,
        accent,
    ))
}

/// Save theme_id, dark_mode, accent_override to config file. Creates dir if needed (XDG on Linux).
pub fn save(theme_id: ThemeId, dark_mode: bool, accent_override: Option<Color>) {
    let path = match config_path() {
        Some(p) => p,
        None => return,
    };
    let file = ThemeSettingsFile {
        theme_id: theme_id_to_string(theme_id),
        dark_mode,
        accent_hex: accent_override.map(color_to_hex),
    };
    if let Ok(json) = serde_json::to_string_pretty(&file) {
        let _ = fs::write(&path, json);
    }
}

/// Parse hex string to Color for accent. Use in handler when user types in hex field.
pub fn parse_accent_hex(s: &str) -> Option<Color> {
    parse_hex(s)
}

/// Format Color as #rrggbb for the hex input field.
pub fn color_to_hex_export(c: Color) -> String {
    color_to_hex(c)
}
