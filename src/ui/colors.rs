use iced::Color;

// Primary colors - a more sophisticated purple
pub const PRIMARY: Color = Color::from_rgb(0.55, 0.45, 0.95);
pub const PRIMARY_DARK: Color = Color::from_rgb(0.45, 0.35, 0.85);
pub const PRIMARY_LIGHT: Color = Color::from_rgb(0.65, 0.55, 1.0);

// Background layers - sleek slate/charcoal
pub const BG_PRIMARY: Color = Color::from_rgb(0.12, 0.12, 0.14); // #1e1e24
pub const BG_SECONDARY: Color = Color::from_rgb(0.16, 0.16, 0.18); // #29292e
pub const BG_TERTIARY: Color = Color::from_rgb(0.20, 0.20, 0.24); // #33333d

// Text colors
pub const TEXT_PRIMARY: Color = Color::from_rgb(0.95, 0.95, 0.98);
pub const TEXT_SECONDARY: Color = Color::from_rgb(0.75, 0.75, 0.80);
pub const TEXT_TERTIARY: Color = Color::from_rgb(0.55, 0.55, 0.60);

// Accent colors
pub const SUCCESS: Color = Color::from_rgb(0.3, 0.85, 0.5);
pub const ERROR: Color = Color::from_rgb(0.95, 0.35, 0.35);
pub const WARNING: Color = Color::from_rgb(0.95, 0.75, 0.2);

// Border colors
pub const BORDER: Color = Color::from_rgb(0.25, 0.25, 0.30);
pub const BORDER_FOCUS: Color = PRIMARY;
