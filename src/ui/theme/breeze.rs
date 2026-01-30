//! Breeze theme – KDE Plasma. See assets/css/breeze_*.css.
use super::ThemeStyleConfig;
use iced::theme::Palette;
use iced::{Color, Shadow, Vector};

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::from_rgb(r, g, b)
}

/// Button/tab styling to match KDE Plasma Breeze: radius 4, light shadow, gray border.
pub fn style_config() -> ThemeStyleConfig {
    ThemeStyleConfig {
        button_radius: 4.0,
        tab_radius: 0.0,
        shadow_offset: Vector::new(0.0, 1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 1.0,
        },
        border_width: 1.0,
        border_color: Some(Color::from_rgb(0.8, 0.8, 0.8)),
        checkbox_radius: 2.0,
        checkbox_border_width: 1.0,
        checkbox_border_color: Color::from_rgb(0.7, 0.7, 0.7),
        checkbox_icon_color: Some(Color::from_rgb(0.24, 0.52, 0.78)), // Breeze accent blue
    }
}

/// Breeze dark: deep gray background, Plasma blue accent.
pub fn palette_dark() -> Palette {
    Palette {
        background: rgb(0.14, 0.15, 0.16),   // #232629
        text: rgb(0.94, 0.94, 0.95),          // #eff0f1
        primary: rgb(0.24, 0.68, 0.91),       // #3daee9
        success: rgb(0.29, 0.78, 0.49),
        danger: rgb(0.90, 0.36, 0.38),
    }
}

/// Breeze light: light gray background, Plasma blue accent.
pub fn palette_light() -> Palette {
    Palette {
        background: rgb(0.98, 0.98, 0.98),
        text: rgb(0.13, 0.13, 0.13),
        primary: rgb(0.24, 0.68, 0.91),       // #3daee9
        success: rgb(0.29, 0.78, 0.49),
        danger: rgb(0.90, 0.36, 0.38),
    }
}
