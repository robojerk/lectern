pub mod search;
pub mod metadata;
pub mod cover;
pub mod chapters;
pub mod convert;
pub mod settings;

use crate::ui::{Message, Lectern};
use crate::ui::colors;
use iced::widget::{button, container, row, text, Space};
use iced::{Alignment, Element, Length};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ViewMode {
    Search,
    Metadata,
    Cover,
    Chapters,
    Convert,
    Settings,
}


pub trait LecternView {
    fn view_tab_bar(&self) -> Element<'_, Message>;
    fn view_header(&self) -> Element<'_, Message>;
}

impl LecternView for Lectern {
    fn view_tab_bar(&self) -> Element<'_, Message> {
        let tab_row = row![
            button("Metadata")
                .style(if self.view_mode == ViewMode::Metadata {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToMetadata)
                .padding([10, 20]),
            button("Cover")
                .style(if self.view_mode == ViewMode::Cover {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToCover)
                .padding([10, 20]),
            button("Chapters")
                .style(if self.view_mode == ViewMode::Chapters {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToChapters)
                .padding([10, 20]),
            button("Convert")
                .style(if self.view_mode == ViewMode::Convert {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToConvert)
                .padding([10, 20]),
            button("Settings")
                .style(if self.view_mode == ViewMode::Settings {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToSettings)
                .padding([10, 20]),
        ]
        .spacing(10)
        .align_items(Alignment::Center);
        
        let mut final_row = row![tab_row];
        
        // Only show Search button on Metadata tab
        if self.view_mode == ViewMode::Metadata {
            final_row = final_row.push(Space::with_width(Length::Fill));
            final_row = final_row.push(
                button("Search Metadata")
                    .on_press(Message::SwitchToSearch)
                    .style(iced::theme::Button::Secondary)
                    .padding([10, 20])
            );
        }
        
        container(final_row.width(Length::Fill))
            .padding([0, 0, 10, 0])
            .into()
    }
    
    fn view_header(&self) -> Element<'_, Message> {
        container(
            row![
                iced::widget::column![
                    text("LECTERN")
                        .size(24)
                        .style(iced::theme::Text::Color(colors::PRIMARY))
                        .font(iced::Font::with_name("sans-serif")),
                    text("Audiobook Management Tool")
                        .size(10)
                        .style(iced::theme::Text::Color(colors::TEXT_TERTIARY)),
                ]
                .spacing(2),
                Space::with_width(Length::Fill),
                container(
                    row![
                        text(if self.metadata.selected_book.is_some() {
                            "●"
                        } else {
                            "○"
                        })
                        .size(10)
                        .style(iced::theme::Text::Color(if self.metadata.selected_book.is_some() {
                            colors::SUCCESS
                        } else {
                            colors::TEXT_TERTIARY
                        })),
                        Space::with_width(Length::Fixed(8.0)),
                        text(if self.metadata.selected_book.is_some() {
                            format!("Editing: {}", self.metadata.editing_title)
                        } else {
                            "No book selected".to_string()
                        })
                        .size(13)
                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
                    ]
                    .align_items(Alignment::Center)
                )
                .padding([8, 16])
                .style(iced::theme::Container::Box),
            ]
            .align_items(Alignment::Center)
        )
        .width(Length::Fill)
        .padding([10, 0, 20, 0])
        .into()
    }
}
