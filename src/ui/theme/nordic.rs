//! Nord theme – Polar Night / Snow Storm. See assets/css/nord_*.css.
use super::ThemeStyleConfig;
use iced::theme::Palette;
use iced::{Color, Shadow, Vector};

fn rgb(r: f32, g: f32, b: f32) -> Color {
    Color::from_rgb(r, g, b)
}

/// Button/tab styling: clean Nordic look, slightly rounded, very subtle shadow.
pub fn style_config() -> ThemeStyleConfig {
    ThemeStyleConfig {
        button_radius: 6.0,
        tab_radius: 0.0,
        shadow_offset: Vector::new(0.0, 1.0),
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: Vector::new(0.0, 1.0),
            blur_radius: 2.0,
        },
        border_width: 0.0,
        border_color: None,
        checkbox_radius: 4.0,
        checkbox_border_width: 1.0,
        checkbox_border_color: Color::from_rgb(0.45, 0.55, 0.65),
        checkbox_icon_color: None, // use theme primary (Frost)
    }
}

/// Nord dark: Polar Night background, Snow Storm text, Frost accent.
pub fn palette_dark() -> Palette {
    // Polar Night: nord0–nord3; Snow Storm text: nord4–nord6; Frost: nord8
    Palette {
        background: rgb(0.18, 0.20, 0.25),   // #2e3440 nord0
        text: rgb(0.85, 0.87, 0.91),         // #d8dee9 nord4
        primary: rgb(0.53, 0.75, 0.82),     // #88c0d0 nord8 Frost
        success: rgb(0.16, 0.63, 0.40),     // Nord green
        danger: rgb(0.90, 0.36, 0.38),       // Nord red
    }
}

/// Nord light: Snow Storm background, Polar Night text, Frost accent.
pub fn palette_light() -> Palette {
    Palette {
        background: rgb(0.93, 0.94, 0.96),   // #eceff4 nord6
        text: rgb(0.18, 0.20, 0.25),        // #2e3440 nord0
        primary: rgb(0.37, 0.51, 0.67),    // #5e81ac nord10
        success: rgb(0.16, 0.63, 0.40),
        danger: rgb(0.90, 0.36, 0.38),
    }
}
