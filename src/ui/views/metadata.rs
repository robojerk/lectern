use crate::ui::{Message, Lectern};
use crate::ui::views::LecternView;
use crate::ui::colors;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Column, Space};
use iced::{Alignment, Element, Length};

pub fn view_metadata(app: &Lectern) -> Element<Message> {
        // File selection area (shown when no book is selected or at the top)
        let mut file_selection_col = Column::new();
        file_selection_col = file_selection_col.push(
            text("📁 Select Audiobook")
                .size(22)
                .style(iced::theme::Text::Color(colors::PRIMARY))
        );
        file_selection_col = file_selection_col.push(Space::with_height(Length::Fixed(8.0)));
        file_selection_col = file_selection_col.push(
            text("Drag and drop a folder or M4B file here, or click Browse")
                .size(14)
                .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
        );
        
        // Show Wayland-specific note if on Wayland
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            file_selection_col = file_selection_col.push(Space::with_height(Length::Fixed(5.0)));
            file_selection_col = file_selection_col.push(
                text("ℹ️ Note: Drag and drop from file managers is not yet supported on Wayland in Iced 0.12. Please use the Browse buttons below.")
                    .size(11)
                    .style(iced::theme::Text::Color(colors::TEXT_TERTIARY))
            );
        }
        file_selection_col = file_selection_col.push(Space::with_height(Length::Fixed(20.0)));
        file_selection_col = file_selection_col.push(
            row![
                button("Browse Files...")
                    .on_press(Message::BrowseFiles)
                    .style(iced::theme::Button::Primary)
                    .padding([12, 20]),
                button("Browse Folder...")
                    .on_press(Message::BrowseFolder)
                    .style(iced::theme::Button::Secondary)
                    .padding([12, 20]),
            ]
            .spacing(15)
            .align_items(Alignment::Center)
        );
        if let Some(ref path) = app.selected_file_path {
            file_selection_col = file_selection_col.push(
                container(
                    text(format!("Selected: {}", 
                        if path.len() > 60 {
                            format!("{}...", &path[..60])
                        } else {
                            path.clone()
                        }))
                        .size(12)
                        .style(iced::theme::Text::Color(colors::TEXT_TERTIARY))
                )
                .padding([10, 15])
                .style(iced::theme::Container::Box)
            );
        }
        file_selection_col = file_selection_col.push(Space::with_height(Length::Fixed(10.0)));
        if app.is_parsing_file {
            file_selection_col = file_selection_col.push(
                text("Parsing file...")
                    .size(14)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
            );
        } else if let Some(ref error) = app.file_parse_error {
            file_selection_col = file_selection_col.push(
                text(format!("Error: {}", error))
                    .size(14)
                    .style(iced::theme::Text::Color(colors::ERROR))
            );
        }
        
        let file_selection_area = container(
            file_selection_col
                .spacing(10)
                .align_items(Alignment::Center)
                .padding(40),
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill);
        
        if app.selected_book.is_none() {
            return container(
                column![
                    file_selection_area,
                ]
                .spacing(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into();
        }
        
        // Tab bar
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        // Metadata fields - styled with labels
        let fields = column![
            text("Book Metadata")
                .size(20)
                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
            Space::with_height(Length::Fixed(10.0)),
            column![
                text("Title")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text_input("Enter book title", &app.editing_title)
                    .on_input(Message::TitleChanged)
                    .padding(12),
            ]
            .spacing(5),
            column![
                text("Subtitle")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text_input("Enter subtitle (optional)", &app.editing_subtitle)
                    .on_input(Message::SubtitleChanged)
                    .padding(12),
            ]
            .spacing(5),
            column![
                text("Author(s)")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text_input("Enter author name(s)", &app.editing_author)
                    .on_input(Message::AuthorChanged)
                    .padding(12),
            ]
            .spacing(5),
            row![
                column![
                    text("Series")
                        .size(12)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                    text_input("Series name", &app.editing_series)
                        .on_input(Message::SeriesChanged)
                        .padding(12),
                ]
                .spacing(5)
                .width(Length::FillPortion(3)),
                column![
                    text("Series #")
                        .size(12)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                    text_input("Number", &app.editing_series_number)
                        .on_input(Message::SeriesNumberChanged)
                        .padding(12),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
            ]
            .spacing(15),
            column![
                text("Narrator(s)")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text_input("Enter narrator name(s)", &app.editing_narrator)
                    .on_input(Message::NarratorChanged)
                    .padding(12),
            ]
            .spacing(5),
            row![
                column![
                    text("ISBN")
                        .size(12)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                    text_input("ISBN", &app.editing_isbn)
                        .on_input(Message::IsbnChanged)
                        .padding(10),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
                column![
                    text("ASIN")
                        .size(12)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                    text_input("ASIN", &app.selected_book.as_ref().map(|b| b.asin.clone().unwrap_or_default()).unwrap_or_default())
                        .on_input(Message::AsinChanged)
                        .padding(10),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            row![
                text_input("Publisher", &app.editing_publisher)
                    .on_input(Message::PublisherChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                text_input("Publish Year", &app.editing_publish_year)
                    .on_input(Message::PublishYearChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            row![
                text_input("Genre", &app.editing_genre)
                    .on_input(Message::GenreChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
                text_input("Language", &app.editing_language)
                    .on_input(Message::LanguageChanged)
                    .padding(10)
                    .width(Length::FillPortion(1)),
            ]
            .spacing(10),
            text_input("Tags (comma-separated)", &app.editing_tags)
                .on_input(Message::TagsChanged)
                .padding(10),
            row![
                checkbox("Explicit", app.editing_explicit)
                    .on_toggle(Message::ExplicitToggled)
                    .text_size(14),
                Space::with_width(Length::Fixed(30.0)),
                checkbox("Abridged", app.editing_abridged)
                    .on_toggle(Message::AbridgedToggled)
                    .text_size(14),
            ]
            .spacing(20)
            .padding([10, 0]),
            column![
                text("Description")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text_input("Enter book description", &app.editing_description)
                    .on_input(Message::DescriptionChanged)
                    .padding(12)
                    .size(14),
            ]
            .spacing(5),
        ]
        .spacing(15);
        
        // Book is selected - hide file selection area, show tabs and fields
        container(
            column![
                tab_bar,
                scrollable(fields).height(Length::Fill),
            ]
            .spacing(15),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
