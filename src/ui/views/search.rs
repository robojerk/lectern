use crate::ui::{Message, Lectern};
use crate::ui::views::LecternView;
use crate::ui::colors;
use crate::services::BookMetadata;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Space, image};
use iced::widget::image::Handle;
use iced::{Alignment, Element, Length};

pub fn view_search(app: &Lectern) -> Element<Message> {
        // Tab bar for navigation
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        // Provider selection
        let provider_section = container(
            column![
                text("Search Provider:")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                Space::with_height(Length::Fixed(5.0)),
                // First row of providers
                row![
                    button("Auto")
                        .style(if app.metadata_provider == "auto" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("auto".to_string()))
                        .padding([6, 12]),
                    button("Audible.com")
                        .style(if app.metadata_provider == "audible_com" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("audible_com".to_string()))
                        .padding([6, 12]),
                    button("Audible.ca")
                        .style(if app.metadata_provider == "audible_ca" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("audible_ca".to_string()))
                        .padding([6, 12]),
                    button("Audnexus")
                        .style(if app.metadata_provider == "audnexus" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("audnexus".to_string()))
                        .padding([6, 12]),
                ]
                .spacing(8)
                .align_items(Alignment::Center),
                Space::with_height(Length::Fixed(5.0)),
                // Second row of providers
                row![
                    button("Google Books")
                        .style(if app.metadata_provider == "google_books" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("google_books".to_string()))
                        .padding([6, 12]),
                    button("iTunes")
                        .style(if app.metadata_provider == "itunes" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("itunes".to_string()))
                        .padding([6, 12]),
                    button("Open Library")
                        .style(if app.metadata_provider == "open_library" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("open_library".to_string()))
                        .padding([6, 12]),
                    button("FantLab.ru")
                        .style(if app.metadata_provider == "fantlab" {
                            iced::theme::Button::Primary
                        } else {
                            iced::theme::Button::Secondary
                        })
                        .on_press(Message::MetadataProviderChanged("fantlab".to_string()))
                        .padding([6, 12]),
                ]
                .spacing(8)
                .align_items(Alignment::Center),
            ]
            .spacing(5),
        )
        .padding(10)
        .style(iced::theme::Container::Box);
        
        // Search bar
        let search_bar = column![
            row![
                button("← Back")
                    .on_press(Message::SwitchToMetadata)
                    .style(iced::theme::Button::Secondary)
                    .padding([10, 15]),
                text_input("Search Title or ASIN...", &app.search_query)
                    .on_input(Message::SearchQueryChanged)
                    .on_submit(Message::PerformSearch)
                    .width(Length::Fill)
                    .padding(12),
                text_input("Author (optional)...", &app.search_author)
                    .on_input(Message::SearchAuthorChanged)
                    .on_submit(Message::PerformSearch)
                    .width(Length::Fill)
                    .padding(12),
                button(
                    if app.is_searching {
                        "Searching..."
                    } else {
                        "Search"
                    }
                )
                .on_press(Message::PerformSearch)
                .style(iced::theme::Button::Primary)
                .padding([12, 20]),
            ]
            .spacing(10)
            .align_items(Alignment::Center),
        ]
        .spacing(10);
        
        // Search results
        let results_content: Element<Message> = if app.is_searching {
            container(text("Searching...").size(18))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Box)
                .into()
        } else if let Some(ref error) = app.search_error {
            container(
                column![
                    text(format!("Error: {}", error)).size(16),
                    text("Check console for details").size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box)
            .into()
        } else if app.search_results.is_empty() && !app.search_query.is_empty() {
            container(
                column![
                    text("No results found").size(16),
                    text("Try a different search term or ASIN").size(12)
                        .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .style(iced::theme::Container::Box)
            .into()
        } else if app.search_results.is_empty() {
            container(text("Enter a book title, author, or ASIN to search").size(16))
                .width(Length::Fill)
                .height(Length::Fill)
                .center_x()
                .center_y()
                .style(iced::theme::Container::Box)
                .into()
        } else {
            // Pagination: show only current page
            let start_idx = app.search_current_page * app.search_results_per_page;
            let end_idx = (start_idx + app.search_results_per_page).min(app.search_results.len());
            let total_pages = (app.search_results.len() + app.search_results_per_page - 1) / app.search_results_per_page;
            let current_page_results = &app.search_results[start_idx..end_idx];
            
            let mut results_column = Column::new();
            
            // Pagination info and controls
            if app.search_results.len() > app.search_results_per_page {
                results_column = results_column.push(
                    row![
                        text(format!("Page {} of {} ({} total results)", 
                            app.search_current_page + 1, 
                            total_pages.max(1),
                            app.search_results.len()))
                            .size(14)
                            .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                        Space::with_width(Length::Fill),
                        button("← Previous")
                            .on_press(Message::PreviousPage)
                            .style(if app.search_current_page > 0 {
                                iced::theme::Button::Primary
                            } else {
                                iced::theme::Button::Secondary
                            })
                            .padding([8, 15]),
                        button("Next →")
                            .on_press(Message::NextPage)
                            .style(if app.search_current_page < total_pages.saturating_sub(1) {
                                iced::theme::Button::Primary
                            } else {
                                iced::theme::Button::Secondary
                            })
                            .padding([8, 15]),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                );
                results_column = results_column.push(Space::with_height(Length::Fixed(10.0)));
            }
            
            // Display results for current page
            for (page_idx, book) in current_page_results.iter().enumerate() {
                let global_idx = start_idx + page_idx;
                results_column = results_column.push(view_search_result(global_idx, book, app));
            }
            
            scrollable(
                results_column
                    .spacing(10)
                    .padding(10),
            )
            .into()
        };
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(15.0)),
                provider_section,
                Space::with_height(Length::Fixed(10.0)),
                search_bar,
                results_content,
            ]
            .spacing(15),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn view_search_result<'a>(index: usize, book: &BookMetadata, app: &'a Lectern) -> Element<'a, Message> {
        // Try to load cover image
        let cover_display: Element<Message> = if let Some(ref cover_url) = book.cover_url {
            if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
                // Check if we have it cached
                if let Some(img_data) = app.search_result_covers.get(cover_url) {
                    // Try to load the image
                    match ::image::load_from_memory(img_data) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let (width, height) = rgba.dimensions();
                            let pixels: Vec<u8> = rgba.into_raw();
                            let handle = Handle::from_pixels(width, height, pixels);
                            container(
                                image(handle)
                                    .width(Length::Fixed(80.0))
                                    .height(Length::Fixed(120.0))
                            )
                            .width(80)
                            .height(120)
                            .into()
                        },
                        Err(_) => {
                            // Failed to load - show placeholder
                            container(
                                text("📖")
                                    .size(40)
                                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                            )
                            .width(80)
                            .height(120)
                            .style(iced::theme::Container::Box)
                            .center_x()
                            .center_y()
                            .into()
                        }
                    }
                } else {
                    // Not cached yet - show placeholder
                    container(
                        text("📖")
                            .size(40)
                            .horizontal_alignment(iced::alignment::Horizontal::Center)
                    )
                    .width(80)
                    .height(120)
                    .style(iced::theme::Container::Box)
                    .center_x()
                    .center_y()
                    .into()
                }
            } else {
                // Not a URL - show placeholder
                container(
                    text("📖")
                        .size(40)
                        .horizontal_alignment(iced::alignment::Horizontal::Center)
                )
                .width(80)
                .height(120)
                .style(iced::theme::Container::Box)
                .center_x()
                .center_y()
                .into()
            }
        } else {
            // No cover URL - show placeholder
            container(
                text("📖")
                    .size(40)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
            )
            .width(80)
            .height(120)
            .style(iced::theme::Container::Box)
            .center_x()
            .center_y()
            .into()
        };
        
        container(
            row![
                // Cover image or placeholder
                cover_display,
                
                // Book info
                column![
                    text(&book.title)
                        .size(18)
                        .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                    Space::with_height(Length::Fixed(5.0)),
                    text(&book.author)
                        .size(14)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                    Space::with_height(Length::Fixed(8.0)),
                    if let Some(ref narrator) = book.narrator {
                        text(format!("Narrated by: {}", narrator))
                            .size(12)
                            .style(iced::theme::Text::Color(colors::TEXT_TERTIARY))
                    } else {
                        text("").size(12)
                    },
                    if let Some(ref year) = book.publish_year {
                        text(format!("Published: {}", year))
                            .size(12)
                            .style(iced::theme::Text::Color(colors::TEXT_TERTIARY))
                    } else {
                        text("").size(12)
                    },
                ]
                .spacing(3)
                .width(Length::Fill)
                .align_items(Alignment::Start),
                
                // "Use This" button
                button("Use This")
                    .on_press(Message::SelectBook(index))
                    .style(iced::theme::Button::Primary)
                    .padding([12, 20]),
            ]
            .spacing(20)
            .align_items(Alignment::Center)
            .padding(20),
        )
        .style(iced::theme::Container::Box)
        .width(Length::Fill)
        .into()
}
