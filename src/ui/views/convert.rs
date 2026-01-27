use crate::ui::{Message, Lectern};
use crate::ui::views::LecternView;
use crate::ui::colors;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space};
use iced::{Alignment, Element, Length};

pub fn view_convert(app: &Lectern) -> Element<Message> {
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        if app.selected_book.is_none() {
            return container(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(20.0)),
                    text("Convert to M4B")
                        .size(20)
                        .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                    Space::with_height(Length::Fixed(10.0)),
                    text("Please select a book first")
                        .size(14)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                    text("Go to the Metadata tab to select or search for a book")
                        .size(12)
                        .style(iced::theme::Text::Color(colors::TEXT_TERTIARY)),
                ]
                .spacing(20)
                .padding(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }
        
        // Determine output path display
        let output_path_display = if let Some(ref lib_path) = app.local_library_path {
            // Show template-based path
            let template = &app.media_management_template;
            let book = app.selected_book.as_ref().unwrap();
            let filename = format!("{}.m4b", book.title.replace("/", "-"));
            format!("{}/{}", lib_path, filename)
        } else if let Some(ref path) = app.output_path {
            path.clone()
        } else {
            "Not set - will prompt for location".to_string()
        };
        
        container(
            scrollable(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(20.0)),
                    text("Convert to M4B")
                        .size(24)
                        .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                    Space::with_height(Length::Fixed(20.0)),
                    // Output path section
                    container(
                        column![
                            text("Output Location")
                                .size(18)
                                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                            Space::with_height(Length::Fixed(10.0)),
                            if app.local_library_path.is_some() {
                                column![
                                    text("Using Local Library path with template")
                                        .size(12)
                                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                    Space::with_height(Length::Fixed(5.0)),
                                    text(output_path_display.as_str())
                                        .size(12)
                                        .style(iced::theme::Text::Color(colors::TEXT_TERTIARY)),
                                ]
                                .spacing(5)
                            } else {
                                column![
                                    row![
                                        text_input("Output file path", &output_path_display)
                                            .on_input(|_| Message::OutputPathSelected(None)) // Read-only when using library
                                            .width(Length::Fill),
                                        button("Browse...")
                                            .on_press(Message::BrowseOutputPath)
                                            .style(iced::theme::Button::Secondary)
                                            .padding([10, 15]),
                                    ]
                                    .spacing(10)
                                    .align_items(Alignment::Center),
                                ]
                            },
                        ]
                        .spacing(10),
                    )
                    .padding(20)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(20.0)),
                    // Conversion summary
                    container(
                        column![
                            text("Conversion Summary")
                                .size(18)
                                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                            Space::with_height(Length::Fixed(10.0)),
                            row![
                                text("Book:")
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                Space::with_width(Length::Fixed(10.0)),
                                text(app.selected_book.as_ref().map(|b| b.title.as_str()).unwrap_or(""))
                                    .size(12),
                            ]
                            .spacing(5),
                            row![
                                text("Cover:")
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                Space::with_width(Length::Fixed(10.0)),
                                text(if app.cover_image_path.is_some() { "Yes" } else { "No" })
                                    .size(12),
                            ]
                            .spacing(5),
                            row![
                                text("Chapters:")
                                    .size(12)
                                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                                Space::with_width(Length::Fixed(10.0)),
                                text(format!("{} chapters", app.chapters.len()))
                                    .size(12),
                            ]
                            .spacing(5),
                        ]
                        .spacing(8),
                    )
                    .padding(20)
                    .style(iced::theme::Container::Box),
                    Space::with_height(Length::Fixed(20.0)),
                    // Convert button
                    if app.is_converting {
                        column![
                            text("Converting...")
                                .size(16)
                                .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                            text("This may take a while")
                                .size(12)
                                .style(iced::theme::Text::Color(colors::TEXT_TERTIARY)),
                        ]
                        .spacing(5)
                        .align_items(Alignment::Center)
                    } else if let Some(ref error) = app.conversion_error {
                        column![
                            text(format!("Error: {}", error))
                                .size(14)
                                .style(iced::theme::Text::Color(colors::ERROR)),
                        ]
                        .spacing(5)
                    } else {
                        column![
                            button("Start Conversion")
                                .on_press(Message::StartConversion)
                                .style(iced::theme::Button::Primary)
                                .padding([15, 30]),
                        ]
                    },
                ]
                .spacing(20)
                .padding(20),
            )
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
