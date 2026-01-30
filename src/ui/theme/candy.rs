//! Candy theme – soft pastels, candy accents. See assets/css/candy_*.css.
use super::ThemeStyleConfig;
use iced::theme::Palette;
use iced::{Color, Shadow, Vector};

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::from_rgb(r, g, b)
}

/// Button/tab styling: softer, more rounded, gentle shadow.
pub fn style_config() -> ThemeStyleConfig {
    ThemeStyleConfig {
        button_radius: 10.0,
        tab_radius: 6.0,
        shadow_offset: Vector::new(0.0, 2.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.12),
            offset: Vector::new(0.0, 2.0),
            blur_radius: 4.0,
        },
        border_width: 0.0,
        border_color: None,
        checkbox_radius: 6.0,
        checkbox_border_width: 1.0,
        checkbox_border_color: Color::from_rgb(0.85, 0.75, 0.95),
        checkbox_icon_color: None, // use theme primary (candy pink)
    }
}

/// Candy light: lavender-white base, hot pink accent (from candy_dark.css – light variant).
pub fn palette_light() -> Palette {
    Palette {
        background: rgb(0.98, 0.96, 0.99),   // #f9f5fd
        text: rgb(0.18, 0.11, 0.28),         // #2d1b47
        primary: rgb(0.94, 0.38, 0.57),     // #f06292
        success: rgb(0.08, 0.72, 0.65),      // #14b8a6
        danger: rgb(0.90, 0.36, 0.38),
    }
}

/// Candy dark: deep purple-gray base, same candy accent.
pub fn palette_dark() -> Palette {
    Palette {
        background: rgb(0.14, 0.10, 0.18),
        text: rgb(0.93, 0.94, 0.96),
        primary: rgb(0.94, 0.38, 0.57),
        success: rgb(0.08, 0.72, 0.65),
        danger: rgb(0.90, 0.36, 0.38),
    }
}
