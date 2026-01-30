use crate::ui::{Message, Lectern};
use crate::ui::state::MetadataProvider;
use crate::models::BookMetadata;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Space, image, pick_list};
use iced::{Alignment, Element, Length};

pub fn view_search(app: &Lectern) -> Element<'_, Message> {
        // Tab bar for navigation
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        // Search bar with Provider Dropdown
        let search_bar = row![
            pick_list(
                &MetadataProvider::ALL[..],
                Some(app.metadata.metadata_provider),
                Message::MetadataProviderChanged
            )
            .padding(10)
            .width(Length::Fixed(150.0)),
            
            text_input("Search Title or ASIN...", &app.search.query)
                .on_input(Message::SearchQueryChanged)
                .on_submit(Message::PerformSearch)
                .width(Length::Fill)
                .padding(12),
            
            text_input("Author (optional)...", &app.search.author)
                .on_input(Message::SearchAuthorChanged)
                .on_submit(Message::PerformSearch)
                .width(Length::Fill)
                .padding(12),
            
            button(
                if app.search.is_searching {
                    "Searching..."
                } else {
                    "Search"
                }
            )
            .on_press(Message::PerformSearch)
            .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
            .padding([12, 20]),
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        // Search results
        let results_content: Element<Message> = if app.search.is_searching {
            container(
                column![
                    text("Searching...").size(24),
                    text("Querying providers for metadata").size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else if let Some(ref error) = app.search.error {
            container(
                column![
                    text("Search Failed").size(20).style(iced::theme::Text::Color(app.palette().danger.base.color)),
                    text(format!("Error: {}", error)).size(16),
                    text("Check console for details").size(12)
                        .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else if app.search.results.is_empty() && !app.search.query.is_empty() {
            container(
                column![
                    text("No results found").size(20),
                    text("Try a different search term or ASIN").size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                ]
                .spacing(10)
                .align_items(Alignment::Center),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else if app.search.results.is_empty() {
            container(
                column![
                    text("Ready to Search").size(20).style(iced::theme::Text::Color(app.palette().primary.base.color)),
                    text("Enter a book title, author, or ASIN to begin").size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                ]
                .spacing(10)
                .align_items(Alignment::Center)
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
        } else {
            // Pagination: show only current page
            let start_idx = app.search.current_page * app.search.results_per_page;
            let end_idx = (start_idx + app.search.results_per_page).min(app.search.results.len());
            let total_pages = (app.search.results.len() + app.search.results_per_page - 1) / app.search.results_per_page;
            let current_page_results = &app.search.results[start_idx..end_idx];
            
            let mut results_column = Column::new().spacing(12);
            
            // Pagination info and controls
            if app.search.results.len() > app.search.results_per_page {
                results_column = results_column.push(
                    row![
                        text(format!("Showing {} - {} of {} results", 
                            start_idx + 1,
                            end_idx,
                            app.search.results.len()))
                            .size(14)
                            .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                        Space::with_width(Length::Fill),
                        button("Previous")
                            .on_press(Message::PreviousPage)
                            .style(if app.search.current_page > 0 {
                                iced::theme::Button::Primary
                            } else {
                                iced::theme::Button::Secondary
                            })
                            .padding([8, 15]),
                        text(format!("Page {} / {}", app.search.current_page + 1, total_pages.max(1)))
                            .size(14),
                        button("Next")
                            .on_press(Message::NextPage)
                            .style(if app.search.current_page < total_pages.saturating_sub(1) {
                                iced::theme::Button::Primary
                            } else {
                                iced::theme::Button::Secondary
                            })
                            .padding([8, 15]),
                    ]
                    .spacing(15)
                    .align_items(Alignment::Center)
                );
                results_column = results_column.push(Space::with_height(Length::Fixed(5.0)));
            }
            
            // Display results for current page
            for (page_idx, book) in current_page_results.iter().enumerate() {
                let global_idx = start_idx + page_idx;
                results_column = results_column.push(view_search_result(global_idx, book, app));
            }
            
            scrollable(
                results_column
                    .padding(10),
            )
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
        };
        
        container(
            column![
                tab_bar,
                search_bar,
                results_content,
            ]
            .spacing(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

pub fn view_search_result<'a>(index: usize, book: &BookMetadata, app: &'a Lectern) -> Element<'a, Message> {
        // Try to load cover image from cached handles
        let cover_display: Element<Message> = if let Some(ref cover_url) = book.cover_url {
            if let Some(handle) = app.search.result_covers.get(cover_url) {
                container(
                    image(handle.clone())
                        .width(Length::Fixed(80.0))
                        .height(Length::Fixed(120.0))
                )
                .width(80)
                .height(120)
                .into()
            } else {
                // Not cached yet or not a URL - show placeholder
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
                        .style(iced::theme::Text::Color(app.palette().background.base.text)),
                    Space::with_height(Length::Fixed(5.0)),
                    text(&book.author)
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_height(Length::Fixed(8.0)),
                    if let Some(ref narrator) = book.narrator {
                        text(format!("Narrated by: {}", narrator))
                            .size(12)
                            .style(iced::theme::Text::Color(app.palette().secondary.base.text))
                    } else {
                        text("").size(12)
                    },
                    if let Some(ref year) = book.publish_year {
                        text(format!("Published: {}", year))
                            .size(12)
                            .style(iced::theme::Text::Color(app.palette().secondary.base.text))
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
