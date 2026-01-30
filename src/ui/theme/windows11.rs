//! Windows 11–style theme – Fluent-inspired light/dark.
use super::ThemeStyleConfig;
use iced::theme::Palette;
use iced::{Color, Shadow, Vector};

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::from_rgb(r, g, b)
}

/// Button/tab styling: Fluent-like, small radius, minimal shadow.
pub fn style_config() -> ThemeStyleConfig {
    ThemeStyleConfig {
        button_radius: 4.0,
        tab_radius: 0.0,
        shadow_offset: Vector::new(0.0, 1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.1),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 2.0,
        },
        border_width: 0.0,
        border_color: None,
        checkbox_radius: 4.0,
        checkbox_border_width: 1.0,
        checkbox_border_color: Color::from_rgb(0.7, 0.7, 0.7),
        checkbox_icon_color: None, // use theme primary (Fluent blue)
    }
}

/// Windows 11 dark: dark gray background, accent blue.
pub fn palette_dark() -> Palette {
    Palette {
        background: rgb(0.11, 0.11, 0.12),
        text: rgb(0.95, 0.95, 0.95),
        primary: rgb(0.00, 0.47, 0.84),      // Fluent blue
        success: rgb(0.16, 0.63, 0.40),
        danger: rgb(0.90, 0.36, 0.38),
    }
}

/// Windows 11 light: white/gray background, accent blue.
pub fn palette_light() -> Palette {
    Palette {
        background: rgb(0.96, 0.96, 0.96),
        text: rgb(0.11, 0.11, 0.12),
        primary: rgb(0.00, 0.47, 0.84),
        success: rgb(0.16, 0.63, 0.40),
        danger: rgb(0.90, 0.36, 0.38),
    }
}
