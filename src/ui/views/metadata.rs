use crate::ui::{Message, Lectern};
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, text_editor, Column, Space};
use iced::{Alignment, Element, Length};

pub fn view_metadata(app: &Lectern) -> Element<'_, Message> {
        // File selection area (shown when no book is selected or at the top)
        let mut file_selection_col = Column::new().spacing(15).align_items(Alignment::Center);
        file_selection_col = file_selection_col.push(
            text("Select Audiobook")
                .size(24)
                .style(iced::theme::Text::Color(app.palette().primary.base.color))
        );
        file_selection_col = file_selection_col.push(
            text("Drag and drop a folder or M4B file here, or click Browse")
                .size(14)
                .style(iced::theme::Text::Color(app.palette().background.weak.text))
        );
        
        // Show Wayland-specific note if on Wayland
        if std::env::var("WAYLAND_DISPLAY").is_ok() {
            file_selection_col = file_selection_col.push(
                text("ℹ Note: Drag and drop is not yet supported on Wayland. Use Browse buttons.")
                    .size(12)
                    .style(iced::theme::Text::Color(app.palette().secondary.base.text))
            );
        }
        
        file_selection_col = file_selection_col.push(
            row![
                button("Browse Files...")
                    .on_press(Message::BrowseFiles)
                    .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                    .padding([12, 24]),
                button("Browse Folder...")
                    .on_press(Message::BrowseFolder)
                    .style(iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(app.theme_id)))
                    .padding([12, 24]),
            ]
            .spacing(20)
            .align_items(Alignment::Center)
        );
        
        if let Some(ref path) = app.file.selected_file_path {
            file_selection_col = file_selection_col.push(
                container(
                    text(format!("Selected: {}", 
                        if path.len() > 80 {
                            format!("...{}", &path[path.len()-80..])
                        } else {
                            path.clone()
                        }))
                        .size(12)
                        .style(iced::theme::Text::Color(app.palette().secondary.base.text))
                )
                .padding([10, 20])
                .style(iced::theme::Container::Box)
            );
        }
        
        if app.file.is_parsing_file {
            file_selection_col = file_selection_col.push(
                text("Parsing file...")
                    .size(14)
                    .style(iced::theme::Text::Color(app.palette().primary.base.color))
            );
        } else if let Some(ref error) = app.file.file_parse_error {
            file_selection_col = file_selection_col.push(
                text(format!("Error: {}", error))
                    .size(14)
                    .style(iced::theme::Text::Color(app.palette().danger.base.color))
            );
        }
        
        let file_selection_area = container(
            file_selection_col.padding(40)
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill);
        
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        if app.metadata.selected_book.is_none() {
            return container(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(20.0)),
                    container(
                        column![
                            file_selection_area,
                        ]
                        .max_width(800)
                        .spacing(20)
                        .align_items(Alignment::Center),
                    )
                    .width(Length::Fill)
                    .center_x(),
                ]
                .spacing(20)
                .width(Length::Fill)
                .height(Length::Fill),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }
        
        // Tab bar (already built above when book is selected)
        
        // Helper for building labeled fields
        let label_color = app.palette().background.weak.text;
        fn labeled_field<'a>(label: &str, label_color: iced::Color, input: impl Into<Element<'a, Message>>) -> Element<'a, Message> {
            column![
                text(label)
                    .size(13)
                    .style(iced::theme::Text::Color(label_color)),
                container(input.into()).width(Length::Fill)
            ]
            .spacing(5)
            .width(Length::Fill)
            .into()
        }

        // Metadata grid
        let fields = column![
            // Row 1: Title
            labeled_field("Book Title", label_color, 
                text_input("Enter book title", &app.metadata.editing_title)
                    .on_input(Message::TitleChanged)
                    .padding(12)
                    .width(Length::Fill)
            ),
            
            // Row 2: Subtitle
            labeled_field("Subtitle", label_color, 
                text_input("Enter subtitle (optional)", &app.metadata.editing_subtitle)
                    .on_input(Message::SubtitleChanged)
                    .padding(12)
                    .width(Length::Fill)
            ),
            
            // Row 3: Author & Narrator
            row![
                labeled_field("Author(s)", label_color, 
                    text_input("Enter author name(s)", &app.metadata.editing_author)
                        .on_input(Message::AuthorChanged)
                        .padding(12)
                ),
                labeled_field("Narrator(s)", label_color, 
                    text_input("Enter narrator name(s)", &app.metadata.editing_narrator)
                        .on_input(Message::NarratorChanged)
                        .padding(12)
                ),
            ].spacing(20),
            
            // Row 4: Series
            row![
                column![
                    text("Series")
                        .size(13)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    text_input("Series name", &app.metadata.editing_series)
                        .on_input(Message::SeriesChanged)
                        .padding(12),
                ]
                .spacing(5)
                .width(Length::FillPortion(3)),
                column![
                    text("Series #")
                        .size(13)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    text_input("Number", &app.metadata.editing_series_number)
                        .on_input(Message::SeriesNumberChanged)
                        .padding(12),
                ]
                .spacing(5)
                .width(Length::FillPortion(1)),
            ].spacing(20),
            
            // Row 5: ISBN & ASIN
            row![
                labeled_field("ISBN", label_color,
                    text_input("ISBN", &app.metadata.editing_isbn)
                        .on_input(Message::IsbnChanged)
                        .padding(12)
                ),
                labeled_field("ASIN", label_color, 
                    text_input("ASIN", &app.metadata.editing_asin)
                        .on_input(Message::AsinChanged)
                        .padding(12)
                ),
            ].spacing(20),
            
            // Row 6: Publisher & Year
            row![
                labeled_field("Publisher", label_color,
                    text_input("Publisher", &app.metadata.editing_publisher)
                        .on_input(Message::PublisherChanged)
                        .padding(12)
                ),
                labeled_field("Publish Year", label_color, 
                    text_input("Year", &app.metadata.editing_publish_year)
                        .on_input(Message::PublishYearChanged)
                        .padding(12)
                ),
            ].spacing(20),
            
            // Row 7: Genre & Language
            row![
                labeled_field("Genre", label_color, 
                    text_input("Genre", &app.metadata.editing_genre)
                        .on_input(Message::GenreChanged)
                        .padding(12)
                ),
                labeled_field("Language", label_color,
                    text_input("Language", &app.metadata.editing_language)
                        .on_input(Message::LanguageChanged)
                        .padding(12)
                ),
            ].spacing(20),
            
            // Row 8: Tags
            labeled_field("Tags (comma-separated)", label_color, 
                text_input("Tags", &app.metadata.editing_tags)
                    .on_input(Message::TagsChanged)
                    .padding(12)
            ),
            
            // Row 9: Checkboxes
            row![
                checkbox("Explicit Content", app.metadata.editing_explicit)
                    .on_toggle(Message::ExplicitToggled)
                    .style(iced::theme::Checkbox::Custom(Box::new(crate::ui::theme::ThemedCheckbox(app.theme_id))))
                    .text_size(15),
                Space::with_width(Length::Fixed(40.0)),
                checkbox("Abridged Version", app.metadata.editing_abridged)
                    .on_toggle(Message::AbridgedToggled)
                    .style(iced::theme::Checkbox::Custom(Box::new(crate::ui::theme::ThemedCheckbox(app.theme_id))))
                    .text_size(15),
            ]
            .padding([10, 0]),
            
            // Row 10: Description with visible vertical scrollbar when content overflows
            column![
                text("Description")
                    .size(13)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                scrollable(
                    container(
                        text_editor(&app.metadata.editing_description_content)
                            .on_action(Message::DescriptionAction)
                            .padding(15)
                            .height(Length::Shrink),
                    )
                    .width(Length::Fill)
                )
                .height(Length::Fixed(200.0)),
            ]
            .spacing(8),
        ]
        .spacing(20)
        .max_width(1000);
        
        container(
            column![
                tab_bar,
                scrollable(
                    container(fields)
                        .width(Length::Fill)
                        .padding([5, 20, 20, 5])
                )
                .width(Length::Fill)
                .height(Length::Fill),
            ]
            .spacing(15),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
