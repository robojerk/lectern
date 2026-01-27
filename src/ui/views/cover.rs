use crate::ui::{Message, Lectern};
use crate::ui::views::LecternView;
use crate::ui::colors;
use crate::ui::cover_search::CoverResult;
use iced::widget::{button, column, container, row, scrollable, text, text_input, Column, Space, image};
use iced::widget::image::Handle;
use iced::{Alignment, Element, Length};
use std::path::Path;

pub fn view_cover(app: &Lectern) -> Element<Message> {
        // Tab bar - always visible when book is selected
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        if app.selected_book.is_none() {
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
        
        let cover_display: Element<Message> = if let Some(ref cover_path) = app.cover_image_path {
            // Try to load and display the image
            let is_url = cover_path.starts_with("http://") || cover_path.starts_with("https://");
            let image_handle = if is_url {
                // For URLs, use downloaded image data if available
                if let Some(ref img_data) = app.cover_image_data {
                    match ::image::load_from_memory(img_data) {
                        Ok(img) => {
                            let rgba = img.to_rgba8();
                            let (width, height) = rgba.dimensions();
                            let pixels: Vec<u8> = rgba.into_raw();
                            println!("[DEBUG] Successfully loaded downloaded image: {}x{}", width, height);
                            Some(Handle::from_pixels(width, height, pixels))
                        },
                        Err(e) => {
                            println!("[DEBUG] Failed to load downloaded image from memory: {}", e);
                            None
                        },
                    }
                } else {
                    // Image not downloaded yet - check if URL matches cached URL
                    // If URL doesn't match cached URL, we need to download
                    let needs_download = app.cover_image_url_cached.as_ref() != Some(cover_path);
                    if needs_download && !app.is_downloading_cover {
                        // Trigger download via message
                        // Note: This will be handled by returning a Command from the view
                        // For now, we'll let the user manually trigger or it will be handled
                        // when the cover is selected/changed
                        None
                    } else if app.is_downloading_cover {
                        None // Still downloading
                    } else {
                        // Should have cached data but don't - this shouldn't happen
                        // but handle gracefully
                        None
                    }
                }
            } else {
                // For local files, try to load the image
                let path = Path::new(cover_path);
                if path.exists() {
                    // Try reading the file
                    if let Ok(img_data) = std::fs::read(path) {
                        // Try loading with image crate - supports JPEG, PNG, etc.
                        match ::image::load_from_memory(&img_data) {
                            Ok(img) => {
                                // Convert to RGBA
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let pixels: Vec<u8> = rgba.into_raw();
                                println!("[DEBUG] Successfully loaded image: {}x{}", width, height);
                                Some(Handle::from_pixels(width, height, pixels))
                            },
                            Err(e) => {
                                println!("[DEBUG] Failed to load image from memory: {}", e);
                                None
                            },
                        }
                    } else {
                        println!("[DEBUG] Failed to read file: {}", cover_path);
                        None
                    }
                } else {
                    println!("[DEBUG] File does not exist: {}", cover_path);
                    None
                }
            };
            
            if let Some(handle) = image_handle {
                // Display actual image
                container(
                    column![
                        text("Cover Image")
                            .size(16),
                        image(handle)
                            .width(Length::Fixed(300.0))
                            .height(Length::Fixed(400.0)),
                        text(if is_url { "URL" } else { "Local file" })
                            .size(12)
                            .style(iced::theme::Text::Color(iced::Color::from_rgb(0.6, 0.6, 0.6))),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center),
                )
                .width(Length::Fill)
                .into()
            } else {
                // Fallback: show placeholder with URL info
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
                            if app.is_downloading_cover {
                                "Downloading image..."
                            } else if app.cover_image_data.is_none() {
                                "URL provided (click to download)"
                            } else {
                                "URL (download failed)"
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
            }
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
                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
            Space::with_height(Length::Fixed(10.0)),
            row![
                button("Search Cover")
                    .on_press(Message::SearchCover)
                    .style(iced::theme::Button::Primary)
                    .padding([12, 20]),
                if app.is_searching_cover {
                    text("Searching...")
                        .size(14)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
                } else {
                    text("")
                },
            ]
            .spacing(15)
            .align_items(Alignment::Center),
            if let Some(ref error) = app.cover_search_error {
                text(format!("Error: {}", error))
                    .size(14)
                    .style(iced::theme::Text::Color(colors::ERROR))
            } else {
                text("").size(14)
            },
            ]
            .spacing(10);
        
        // Cover search results - build the column directly (no nested scrollable)
        let mut cover_results_column = Column::new();
        if !app.cover_search_results.is_empty() {
            cover_results_column = cover_results_column.push(
                text(format!("Found {} cover results:", app.cover_search_results.len()))
                    .size(16)
                    .style(iced::theme::Text::Color(colors::TEXT_PRIMARY))
            );
            cover_results_column = cover_results_column.push(Space::with_height(Length::Fixed(10.0)));
            
            for (index, cover) in app.cover_search_results.iter().enumerate() {
                cover_results_column = cover_results_column.push(
                    container(
                        row![
                            container(
                                text("🖼️")
                                    .size(24)
                                    .horizontal_alignment(iced::alignment::Horizontal::Center)
                            )
                            .width(Length::Fixed(80.0))
                            .height(Length::Fixed(120.0))
                            .style(iced::theme::Container::Box)
                            .center_x()
                            .center_y(),
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
            scrollable(
                column![
                    tab_bar,
                    Space::with_height(Length::Fixed(15.0)),
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
                                .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                            Space::with_height(Length::Fixed(15.0)),
                            button("Browse Image File...")
                                .on_press(Message::BrowseCoverImage)
                                .style(iced::theme::Button::Primary)
                                .padding([12, 20]),
                            Space::with_height(Length::Fixed(15.0)),
                            text("Or enter URL:")
                                .size(14)
                                .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                            text_input("Cover Image URL", 
                                app.cover_image_path.as_deref().unwrap_or(""))
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
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
