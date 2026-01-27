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
    fn view_tab_bar(&self) -> Element<Message>;
    fn view_header(&self) -> Element<Message>;
}

impl LecternView for Lectern {
    fn view_tab_bar(&self) -> Element<Message> {
        let mut tab_row = row![
            button("📁 Metadata")
                .style(if self.view_mode == ViewMode::Metadata {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToMetadata)
                .padding([8, 15]),
            button("🖼️ Cover")
                .style(if self.view_mode == ViewMode::Cover {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToCover)
                .padding([8, 15]),
            button("📑 Chapters")
                .style(if self.view_mode == ViewMode::Chapters {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToChapters)
                .padding([8, 15]),
            button("🔄 Convert")
                .style(if self.view_mode == ViewMode::Convert {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToConvert)
                .padding([8, 15]),
            button("⚙️ Settings")
                .style(if self.view_mode == ViewMode::Settings {
                    iced::theme::Button::Primary
                } else {
                    iced::theme::Button::Secondary
                })
                .on_press(Message::SwitchToSettings)
                .padding([8, 15]),
        ]
        .spacing(8)
        .align_items(Alignment::Center);
        
        // Only show Search button on Metadata tab
        if self.view_mode == ViewMode::Metadata {
            tab_row = tab_row.push(Space::with_width(Length::Fill));
            tab_row = tab_row.push(
                button("🔍 Search")
                    .on_press(Message::SwitchToSearch)
                    .style(iced::theme::Button::Secondary)
                    .padding([8, 15])
            );
        }
        
        tab_row.into()
    }
    
    fn view_header(&self) -> Element<Message> {
        container(
            row![
                text("🎵 Lectern")
                    .size(26)
                    .style(iced::theme::Text::Color(colors::PRIMARY))
                    .font(iced::Font::MONOSPACE),
                Space::with_width(Length::Fill),
                container(
                    text(if self.selected_book.is_some() {
                        "Editing book"
                    } else {
                        "No book selected"
                    })
                    .size(14)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
                )
                .padding([6, 12])
                .style(iced::theme::Container::Box),
            ]
            .align_items(Alignment::Center)
            .spacing(20),
        )
        .padding([20, 25])
        .style(iced::theme::Container::Box)
        .into()
    }
}
