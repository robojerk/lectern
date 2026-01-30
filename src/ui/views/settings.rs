use crate::ui::theme::ThemeId;
use crate::ui::{Message, Lectern};
use iced::widget::container::{Appearance as ContainerAppearance, StyleSheet as ContainerStyleSheet};
use iced::widget::{button, column, container, row, scrollable, text, text_input, toggler, Space, pick_list};
use iced::{Alignment, Background, Element, Length};

pub fn view_settings(app: &Lectern) -> Element<'_, Message> {
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(6.0)),
                scrollable(
                    column![
                        Space::with_height(Length::Fixed(6.0)),
                        text("Settings")
                            .size(24)
                            .style(iced::theme::Text::Color(app.palette().background.base.text)),
                        Space::with_height(Length::Fixed(10.0)),
                    // Appearance section
                    container(
                        column![
                            text("Appearance")
                                .size(18)
                                .style(iced::theme::Text::Color(app.palette().background.base.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("Theme and light/dark mode")
                                .size(12)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            row![
                                column![
                                    text("Theme")
                                        .size(12)
                                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                    pick_list(
                                        ThemeId::ALL,
                                        Some(app.theme_id),
                                        Message::ThemeIdChanged,
                                    )
                                    .width(Length::Fixed(140.0)),
                                ]
                                .spacing(5),
                                column![
                                    text("Mode")
                                        .size(12)
                                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                    row![
                                        text("Light")
                                            .size(14)
                                            .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                        toggler(
                                            None::<String>,
                                            app.dark_mode,
                                            Message::DarkModeToggled,
                                        )
                                        .spacing(10.0),
                                        text("Dark")
                                            .size(14)
                                            .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                    ]
                                    .spacing(10)
                                    .align_items(Alignment::Center),
                                ]
                                .spacing(5),
                                column![
                                    text("Accent")
                                        .size(12)
                                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                    row![
                                        toggler(
                                            Some("Use theme default".to_string()),
                                            app.accent_override.is_none(),
                                            Message::UseThemeDefaultAccentToggled,
                                        )
                                        .spacing(10.0),
                                    ]
                                    .align_items(Alignment::Center),
                                    {
                                        let custom_row: Element<Message> = if let Some(accent) = app.accent_override {
                                            row![
                                                text("Custom (#rrggbb)")
                                                    .size(12)
                                                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                                text_input("#rrggbb", &app.accent_hex_input)
                                                    .on_input(Message::AccentHexInputChanged)
                                                    .width(Length::Fixed(100.0))
                                                    .padding(8),
                                                container(
                                                    Space::with_width(Length::Fixed(24.0))
                                                        .height(Length::Fixed(24.0)),
                                                )
                                                .style(iced::theme::Container::Custom(Box::new(
                                                    AccentSwatchStyle(accent),
                                                )))
                                                .width(Length::Fixed(24.0))
                                                .height(Length::Fixed(24.0)),
                                            ]
                                            .spacing(10)
                                            .align_items(Alignment::Center)
                                            .into()
                                        } else {
                                            row![].into()
                                        };
                                        custom_row
                                    },
                                ]
                                .spacing(8),
                            ]
                            .spacing(30)
                            .align_items(Alignment::End),
                        ]
                        .spacing(10),
                    )
                    .padding(16)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(10.0)),
                    // Local Library section
                    container(
                        column![
                            text("Local Library")
                                .size(18)
                                .style(iced::theme::Text::Color(app.palette().background.base.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("Path where converted audiobooks will be saved")
                                .size(12)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            row![
                                text_input(
                                    "Local library path (optional)",
                                    app.local_library_path.as_deref().unwrap_or("")
                                )
                                .on_input(Message::LocalLibraryPathChanged)
                                .width(Length::Fill),
                                button("Browse...")
                                    .on_press(Message::BrowseLocalLibraryPath)
                                    .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                                    .padding([10, 15]),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                        ]
                        .spacing(10),
                    )
                    .padding(16)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(10.0)),
                    // Media Management Template section
                    container(
                        column![
                            text("Media Management Template")
                                .size(18)
                                .style(iced::theme::Text::Color(app.palette().background.base.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("How to organize saved files (if Local Library is set)")
                                .size(12)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            Space::with_height(Length::Fixed(5.0)),
                            text("Placeholders: {Author}, {Series}, {Title}, {SeriesNumber}, {Year}, {Genre}, {ASIN}, {Language}, {Tags}")
                                .size(11)
                                .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
                            text("SeriesNumber suffix (only when set): {SeriesNumber-} → \"4-\", {SeriesNumber.} → \"4.\", {SeriesNumber } → \"4 \"")
                                .size(11)
                                .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            text_input(
                                "Template (e.g., {Author}/{Title}.m4b)",
                                &app.media_management_template
                            )
                            .on_input(Message::MediaManagementTemplateChanged)
                            .padding(12),
                        ]
                        .spacing(10),
                    )
                    .padding(16)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(10.0)),
                    // Audiobookshelf section
                    container(
                        column![
                            text("Audiobookshelf (Optional)")
                                .size(18)
                                .style(iced::theme::Text::Color(app.palette().background.base.text)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("Automatically upload converted books to Audiobookshelf")
                                .size(12)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            Space::with_height(Length::Fixed(15.0)),
                            column![
                                text("Host URL")
                                    .size(12)
                                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                text_input("https://audiobookshelf.example.com", &app.audiobookshelf_host)
                                    .on_input(Message::AudiobookshelfHostChanged)
                                    .padding(12),
                            ]
                            .spacing(5),
                            column![
                                text("API Token")
                                    .size(12)
                                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                text_input("Your API token", &app.audiobookshelf_token)
                                    .on_input(Message::AudiobookshelfTokenChanged)
                                    .padding(12),
                            ]
                            .spacing(5),
                            column![
                                text("Library ID")
                                    .size(12)
                                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                text_input("Library ID", &app.audiobookshelf_library_id)
                                    .on_input(Message::AudiobookshelfLibraryIdChanged)
                                    .padding(12),
                            ]
                            .spacing(5),
                        ]
                        .spacing(15),
                    )
                    .padding(16)
                    .style(iced::theme::Container::Box),
                ]
                .spacing(12)
                .padding(16),
            ),
            ]
            .spacing(6),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

/// Container style that draws a solid color (for accent swatch).
struct AccentSwatchStyle(iced::Color);

impl ContainerStyleSheet for AccentSwatchStyle {
    type Style = iced::Theme;

    fn appearance(&self, _style: &Self::Style) -> ContainerAppearance {
        ContainerAppearance {
            background: Some(Background::Color(self.0.into())),
            border: iced::Border::with_radius(4.0),
            ..Default::default()
        }
    }
}
