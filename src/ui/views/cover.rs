use crate::ui::{Message, Lectern};
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Space, image};
use iced::{Alignment, Element, Length};

pub fn view_cover(app: &Lectern) -> Element<'_, Message> {
        // Tab bar - always visible when book is selected
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        if app.metadata.selected_book.is_none() {
            return container(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(20.0)),
                    text("No book selected")
                        .size(18)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    text("Select a book to manage its cover art")
                        .size(14)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.5, 0.5, 0.5))),
                ]
                .spacing(20)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into();
        }
        
        let cover_display: Element<Message> = if let Some(ref handle) = app.cover.cover_image_handle {
            // Display actual image from cached handle
            container(
                column![
                    text("Cover Image")
                        .size(16),
                    image(handle.clone())
                        .width(Length::Fixed(300.0))
                        .height(Length::Fixed(400.0)),
                    text(if app.cover.cover_image_path.as_ref().map(|p| p.starts_with("http")).unwrap_or(false) { "URL" } else { "Local file" })
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .into()
        } else if let Some(ref cover_path) = app.cover.cover_image_path {
            // Path provided but no handle - check if it's a URL or local file
            let is_url = cover_path.starts_with("http://") || cover_path.starts_with("https://");
            
            let display_text = if is_url {
                format!("🖼️\nCover Image\n\nURL:\n{}", 
                    if cover_path.len() > 50 {
                        format!("{}...", &cover_path[..50])
                    } else {
                        cover_path.clone()
                    })
            } else {
                format!("🖼️\nCover Image\n\nFile:\n{}", 
                    if cover_path.len() > 50 {
                        format!("{}...", &cover_path[..50])
                    } else {
                        cover_path.clone()
                    })
            };
            
            container(
                column![
                    text("Cover Image")
                        .size(16),
                    container(
                        text(&display_text)
                            .size(12)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(400.0))
                    .style(iced::theme::Container::Box)
                    .center_x()
                    .center_y(),
                    text(if is_url {
                        if app.cover.is_downloading_cover {
                            "Downloading image..."
                        } else {
                            "URL provided (click to download)"
                        }
                    } else {
                        "Local file (could not load image)"
                    })
                        .size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .into()
        } else {
            container(
                column![
                    text("No cover image")
                        .size(16)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    container(
                        text("📖\nNo Cover")
                            .size(14)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .width(Length::Fixed(300.0))
                    .height(Length::Fixed(400.0))
                    .style(iced::theme::Container::Box)
                    .center_x()
                    .center_y(),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .into()
        };
        
        // Cover search section
        let cover_search_section = column![
            text("Search for Cover Art")
                .size(18)
                .style(iced::theme::Text::Color(app.palette().background.base.text)),
            Space::with_height(Length::Fixed(10.0)),
            row![
                button("Search Cover")
                    .on_press(Message::SearchCover)
                    .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                    .padding([12, 20]),
                if app.cover.is_searching_cover {
                    text("Searching...")
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text))
                } else {
                    text("")
                },
            ]
            .spacing(15)
            .align_items(Alignment::Center),
            if let Some(ref error) = app.cover.cover_search_error {
                text(format!("Error: {}", error))
                    .size(14)
                    .style(iced::theme::Text::Color(app.palette().danger.base.color))
            } else {
                text("").size(14)
            },
            ]
            .spacing(10);
        
        // Cover search results - build the column directly (no nested scrollable)
        let mut cover_results_column = Column::new();
        if !app.cover.cover_search_results.is_empty() {
            cover_results_column = cover_results_column.push(
                text(format!("Found {} cover results:", app.cover.cover_search_results.len()))
                    .size(16)
                    .style(iced::theme::Text::Color(app.palette().background.base.text))
            );
            cover_results_column = cover_results_column.push(Space::with_height(Length::Fixed(10.0)));
            
            for (index, cover) in app.cover.cover_search_results.iter().enumerate() {
                let thumb: Element<Message> = if let Some(handle) = app.cover.cover_search_result_handles.get(&cover.url) {
                    container(
                        image(handle.clone())
                            .width(Length::Fixed(80.0))
                            .height(Length::Fixed(120.0)),
                    )
                    .width(Length::Fixed(80.0))
                    .height(Length::Fixed(120.0))
                    .style(iced::theme::Container::Box)
                    .center_x()
                    .center_y()
                    .into()
                } else {
                    container(
                        text("🖼️")
                            .size(24)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .width(Length::Fixed(80.0))
                    .height(Length::Fixed(120.0))
                    .style(iced::theme::Container::Box)
                    .center_x()
                    .center_y()
                    .into()
                };
                cover_results_column = cover_results_column.push(
                    container(
                        row![
                            thumb,
                            column![
                                text(&cover.source)
                                    .size(14)
                                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.7, 0.7, 0.7))),
                                text(format!("{}x{}", cover.width, cover.height))
                                    .size(12)
                                    .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                            ]
                            .spacing(5)
                            .width(Length::Fill),
                            button("Use This")
                                .on_press(Message::SelectCover(index))
                                .style(iced::theme::Button::Primary)
                                .padding([10, 20]),
                        ]
                        .spacing(15)
                        .align_items(Alignment::Center)
                        .padding(10),
                    )
                    .style(iced::theme::Container::Box)
                    .width(Length::Fill)
                );
            }
        }
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(15.0)),
                scrollable(
                    column![
                        // Cover display and options side by side
                        row![
                            // Cover preview - fixed width
                            container(cover_display)
                                .width(Length::Fixed(350.0))
                                .height(Length::Shrink),
                            Space::with_width(Length::Fixed(20.0)),
                            // Options column - takes remaining space
                            column![
                                text("Cover Options")
                                    .size(20)
                                    .style(iced::theme::Text::Color(app.palette().background.base.text)),
                                Space::with_height(Length::Fixed(15.0)),
                                button("Browse Image File...")
                                    .on_press(Message::BrowseCoverImage)
                                    .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                                    .padding([12, 20]),
                                Space::with_height(Length::Fixed(15.0)),
                                text("Or enter URL:")
                                    .size(14)
                                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                text_input("Cover Image URL", 
                                    app.cover.cover_image_path.as_deref().unwrap_or(""))
                                    .on_input(Message::CoverUrlChanged)
                                    .padding(12),
                                Space::with_height(Length::Fixed(20.0)),
                                cover_search_section,
                            ]
                            .spacing(10)
                            .width(Length::Fill),
                        ]
                        .spacing(20)
                        .align_items(Alignment::Start),
                        Space::with_height(Length::Fixed(20.0)),
                        // Search results - part of main scrollable
                        cover_results_column.spacing(10),
                    ]
                    .spacing(20)
                    .padding(20),
                )
            ]
            .spacing(10),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
