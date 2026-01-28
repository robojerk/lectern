use crate::ui::{Message, Lectern};
use crate::ui::colors;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view_settings(app: &Lectern) -> Element<'_, Message> {
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        container(
            column![
                tab_bar,
                scrollable(
                    column![
                        Space::with_height(Length::Fixed(20.0)),
                        text("Settings")
                            .size(24)
                            .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                        Space::with_height(Length::Fixed(20.0)),
                    // Local Library section
                    container(
                        column![
                            text("Local Library")
                                .size(18)
                                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("Path where converted audiobooks will be saved")
                                .size(12)
                                .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
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
                                    .style(iced::theme::Button::Secondary)
                                    .padding([10, 15]),
                            ]
                            .spacing(10)
                            .align_items(Alignment::Center),
                        ]
                        .spacing(10),
                    )
                    .padding(20)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(20.0)),
                    // Media Management Template section
                    container(
                        column![
                            text("Media Management Template")
                                .size(18)
                                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("How to organize saved files (if Local Library is set)")
                                .size(12)
                                .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                            Space::with_height(Length::Fixed(5.0)),
                            text("Available placeholders: {Author}, {Series}, {Title}, {SeriesNumber}, {Year}")
                                .size(11)
                                .style(iced::theme::Text::Color(colors::TEXT_TERTIARY)),
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
                    .padding(20)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(20.0)),
                    // Audiobookshelf section
                    container(
                        column![
                            text("Audiobookshelf (Optional)")
                                .size(18)
                                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                            Space::with_height(Length::Fixed(10.0)),
                            text("Automatically upload converted books to Audiobookshelf")
                                .size(12)
                                .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                            Space::with_height(Length::Fixed(15.0)),
                            column![
                                text("Host URL")
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                text_input("https://audiobookshelf.example.com", &app.audiobookshelf_host)
                                    .on_input(Message::AudiobookshelfHostChanged)
                                    .padding(12),
                            ]
                            .spacing(5),
                            column![
                                text("API Token")
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                text_input("Your API token", &app.audiobookshelf_token)
                                    .on_input(Message::AudiobookshelfTokenChanged)
                                    .padding(12),
                            ]
                            .spacing(5),
                            column![
                                text("Library ID")
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                text_input("Library ID", &app.audiobookshelf_library_id)
                                    .on_input(Message::AudiobookshelfLibraryIdChanged)
                                    .padding(12),
                            ]
                            .spacing(5),
                        ]
                        .spacing(15),
                    )
                    .padding(20)
                    .style(iced::theme::Container::Box),
                ]
                .spacing(20)
                .padding(20),
            ),
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
