//! Custom button and checkbox styles. Per-theme config from ThemeStyleConfig; pass ThemeId when building.
use crate::ui::theme::{style_config_for, ThemeId};
use iced::widget::button::{self, StyleSheet as ButtonStyleSheet};
use iced::widget::checkbox::{self, StyleSheet as CheckboxStyleSheet};
use iced::{Background, Border, Theme};

fn border_for_button(config: &crate::ui::theme::ThemeStyleConfig) -> Border {
    let mut border = Border::with_radius(config.button_radius);
    if config.border_width > 0.0 {
        border.width = config.border_width;
    }
    if let Some(c) = config.border_color {
        border.color = c;
    }
    border
}

/// Tab style with Primary colors. Uses theme tab radius and shadow.
#[derive(Debug, Clone, Copy)]
pub struct SquarePrimary(pub ThemeId);

impl ButtonStyleSheet for SquarePrimary {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let pair = palette.primary.strong;
        button::Appearance {
            background: Some(Background::Color(pair.color.into())),
            text_color: pair.text,
            border: Border::with_radius(config.tab_radius),
            shadow_offset: config.shadow_offset,
            shadow: config.shadow,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let active = self.active(theme);
        button::Appearance {
            background: Some(Background::Color(palette.primary.base.color.into())),
            ..active
        }
    }
}

/// Rounded primary button (theme roundness and shadow, used for main actions).
#[derive(Debug, Clone, Copy)]
pub struct RoundedPrimary(pub ThemeId);

impl ButtonStyleSheet for RoundedPrimary {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let pair = palette.primary.strong;
        button::Appearance {
            background: Some(Background::Color(pair.color.into())),
            text_color: pair.text,
            border: border_for_button(&config),
            shadow_offset: config.shadow_offset,
            shadow: config.shadow,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let active = self.active(theme);
        button::Appearance {
            background: Some(Background::Color(palette.primary.base.color.into())),
            ..active
        }
    }
}

/// Rounded secondary button (theme roundness and shadow).
#[derive(Debug, Clone, Copy)]
pub struct RoundedSecondary(pub ThemeId);

impl ButtonStyleSheet for RoundedSecondary {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let pair = palette.secondary.base;
        button::Appearance {
            background: Some(Background::Color(pair.color.into())),
            text_color: pair.text,
            border: border_for_button(&config),
            shadow_offset: config.shadow_offset,
            shadow: config.shadow,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let active = self.active(theme);
        button::Appearance {
            background: Some(Background::Color(palette.background.strong.color.into())),
            ..active
        }
    }
}

/// Rounded destructive button (theme roundness and shadow, red background).
#[derive(Debug, Clone, Copy)]
pub struct RoundedDestructive(pub ThemeId);

impl ButtonStyleSheet for RoundedDestructive {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let pair = palette.danger.base;
        button::Appearance {
            background: Some(Background::Color(pair.color.into())),
            text_color: pair.text,
            border: border_for_button(&config),
            shadow_offset: config.shadow_offset,
            shadow: config.shadow,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let active = self.active(theme);
        button::Appearance {
            background: Some(Background::Color(palette.danger.strong.color.into())),
            ..active
        }
    }
}

/// Tab style with Secondary colors. Uses theme tab radius and shadow. Used for unselected tabs.
#[derive(Debug, Clone, Copy)]
pub struct SquareSecondary(pub ThemeId);

impl ButtonStyleSheet for SquareSecondary {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let pair = palette.secondary.base;
        button::Appearance {
            background: Some(Background::Color(pair.color.into())),
            text_color: pair.text,
            border: Border::with_radius(config.tab_radius),
            shadow_offset: config.shadow_offset,
            shadow: config.shadow,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let active = self.active(theme);
        button::Appearance {
            background: Some(Background::Color(palette.background.strong.color.into())),
            ..active
        }
    }
}

/// Tab style for disabled tabs (no book selected). Muted background and text; no hover change.
#[derive(Debug, Clone, Copy)]
pub struct SquareDisabled(pub ThemeId);

impl ButtonStyleSheet for SquareDisabled {
    type Style = Theme;

    fn active(&self, theme: &Self::Style) -> button::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let weak = palette.background.weak;
        button::Appearance {
            background: Some(Background::Color(weak.color.into())),
            text_color: weak.text,
            border: Border::with_radius(config.tab_radius),
            shadow_offset: config.shadow_offset,
            shadow: config.shadow,
            ..button::Appearance::default()
        }
    }

    fn hovered(&self, theme: &Self::Style) -> button::Appearance {
        self.active(theme)
    }
}

// --- Checkbox (per-theme) ---

fn border_for_checkbox(config: &crate::ui::theme::ThemeStyleConfig) -> Border {
    let mut border = Border::with_radius(config.checkbox_radius);
    border.width = config.checkbox_border_width;
    border.color = config.checkbox_border_color;
    border
}

/// Checkbox style using theme checkbox config and palette (background, text color).
#[derive(Debug, Clone, Copy)]
pub struct ThemedCheckbox(pub ThemeId);

impl CheckboxStyleSheet for ThemedCheckbox {
    type Style = Theme;

    fn active(&self, theme: &Self::Style, _is_checked: bool) -> checkbox::Appearance {
        let palette = theme.extended_palette();
        let config = style_config_for(self.0);
        let icon_color = config
            .checkbox_icon_color
            .unwrap_or(palette.primary.base.color);
        checkbox::Appearance {
            background: Background::Color(palette.background.base.color.into()),
            icon_color,
            border: border_for_checkbox(&config),
            text_color: Some(palette.background.base.text),
        }
    }

    fn hovered(&self, theme: &Self::Style, is_checked: bool) -> checkbox::Appearance {
        let mut appearance = self.active(theme, is_checked);
        let palette = theme.extended_palette();
        appearance.background =
            Background::Color(palette.background.weak.color.into());
        appearance
    }
}
