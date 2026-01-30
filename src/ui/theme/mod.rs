//! Theme registry: ThemeId, palette_for, build_theme. Per-theme palettes and style configs live in nordic, breeze, candy, windows11.
mod breeze;
mod candy;
mod nordic;
mod styles;
mod windows11;

pub use styles::*;

use iced::{Color, Shadow, Vector};

/// Per-theme styling for buttons, tabs, and checkboxes.
#[derive(Debug, Clone, Copy)]
pub struct ThemeStyleConfig {
    pub button_radius: f32,
    pub tab_radius: f32,
    pub shadow_offset: Vector,
    pub shadow: Shadow,
    /// Border width for buttons; 0 to use default.
    pub border_width: f32,
    /// Border color for buttons; None to use default.
    pub border_color: Option<Color>,
    // Checkbox
    pub checkbox_radius: f32,
    pub checkbox_border_width: f32,
    pub checkbox_border_color: Color,
    /// Check mark icon color; None to use theme primary.
    pub checkbox_icon_color: Option<Color>,
}

/// Returns the style config (button/tab radius, shadow) for the given theme.
pub fn style_config_for(theme_id: ThemeId) -> ThemeStyleConfig {
    match theme_id {
        ThemeId::Nordic => nordic::style_config(),
        ThemeId::Breeze => breeze::style_config(),
        ThemeId::Candy => candy::style_config(),
        ThemeId::Windows11 => windows11::style_config(),
    }
}

use iced::theme::palette::Extended;
use iced::theme::{Palette, Theme};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ThemeId {
    Nordic,
    Breeze,
    Candy,
    #[default]
    Windows11,
}

impl ThemeId {
    pub const ALL: &'static [ThemeId] = &[ThemeId::Nordic, ThemeId::Breeze, ThemeId::Candy, ThemeId::Windows11];

    pub fn name(self) -> &'static str {
        match self {
            ThemeId::Nordic => "Nordic",
            ThemeId::Breeze => "Breeze",
            ThemeId::Candy => "Candy",
            ThemeId::Windows11 => "Windows 11",
        }
    }
}

impl std::fmt::Display for ThemeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}

/// Returns the base palette for the given theme and light/dark mode. No accent override.
pub fn palette_for(theme_id: ThemeId, dark: bool) -> Palette {
    match theme_id {
        ThemeId::Nordic => {
            if dark {
                nordic::palette_dark()
            } else {
                nordic::palette_light()
            }
        }
        ThemeId::Breeze => {
            if dark {
                breeze::palette_dark()
            } else {
                breeze::palette_light()
            }
        }
        ThemeId::Candy => {
            if dark {
                candy::palette_dark()
            } else {
                candy::palette_light()
            }
        }
        ThemeId::Windows11 => {
            if dark {
                windows11::palette_dark()
            } else {
                windows11::palette_light()
            }
        }
    }
}

/// Builds the Iced Theme and Extended palette for the given state.
/// When accent_override is Some, only palette.primary is replaced; success/danger stay unchanged.
/// Extended::generate derives primary text contrast from the (possibly overridden) primary.
pub fn build_theme(theme_id: ThemeId, dark: bool, accent_override: Option<Color>) -> (Theme, Extended) {
    let mut palette = palette_for(theme_id, dark);
    if let Some(accent) = accent_override {
        palette.primary = accent;
    }
    let extended = Extended::generate(palette);
    let name = format!("{} {}", theme_id.name(), if dark { "Dark" } else { "Light" });
    let theme = Theme::custom(name, palette);
    (theme, extended)
}
