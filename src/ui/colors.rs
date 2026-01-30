//! Theme-independent color constants. Theme-driven colors use app.palette().
use iced::Color;

/// Warning / caution (e.g. "saved size" note when output is larger). No warning in iced Palette.
pub const WARNING: Color = Color::from_rgb(0.95, 0.75, 0.2);
