pub mod search;
pub mod metadata;
pub mod cover;
pub mod chapters;
pub mod convert;
pub mod settings;

use crate::ui::{Message, Lectern};
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
        use crate::ui::theme::{SquarePrimary, SquareSecondary, SquareDisabled};
        let tid = self.theme_id;
        let no_book = self.metadata.selected_book.is_none();
        let tab_style = |selected: bool| {
            if selected {
                iced::theme::Button::custom(SquarePrimary(tid))
            } else {
                iced::theme::Button::custom(SquareSecondary(tid))
            }
        };
        let tab_row = if no_book {
            row![
                button("Metadata")
                    .style(tab_style(self.view_mode == ViewMode::Metadata))
                    .on_press(Message::SwitchToMetadata)
                    .padding([10, 20]),
                button("Cover")
                    .style(iced::theme::Button::custom(SquareDisabled(tid)))
                    .padding([10, 20]),
                button("Chapters")
                    .style(iced::theme::Button::custom(SquareDisabled(tid)))
                    .padding([10, 20]),
                button("Convert")
                    .style(iced::theme::Button::custom(SquareDisabled(tid)))
                    .padding([10, 20]),
                button("Settings")
                    .style(tab_style(self.view_mode == ViewMode::Settings))
                    .on_press(Message::SwitchToSettings)
                    .padding([10, 20]),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        } else {
            row![
                button("Metadata")
                    .style(tab_style(self.view_mode == ViewMode::Metadata))
                    .on_press(Message::SwitchToMetadata)
                    .padding([10, 20]),
                button("Cover")
                    .style(tab_style(self.view_mode == ViewMode::Cover))
                    .on_press(Message::SwitchToCover)
                    .padding([10, 20]),
                button("Chapters")
                    .style(tab_style(self.view_mode == ViewMode::Chapters))
                    .on_press(Message::SwitchToChapters)
                    .padding([10, 20]),
                button("Convert")
                    .style(tab_style(self.view_mode == ViewMode::Convert))
                    .on_press(Message::SwitchToConvert)
                    .padding([10, 20]),
                button("Settings")
                    .style(tab_style(self.view_mode == ViewMode::Settings))
                    .on_press(Message::SwitchToSettings)
                    .padding([10, 20]),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        };
        
        let mut final_row = row![tab_row];
        
        // Only show Search button on Metadata tab when a book is selected
        if self.view_mode == ViewMode::Metadata && self.metadata.selected_book.is_some() {
            final_row = final_row.push(Space::with_width(Length::Fill));
            final_row = final_row.push(
                button("Search Metadata")
                    .on_press(Message::SwitchToSearch)
                    .style(iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(self.theme_id)))
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
                        .style(iced::theme::Text::Color(self.palette().primary.base.color))
                        .font(iced::Font::with_name("sans-serif")),
                    text("Audiobook Management Tool")
                        .size(10)
                        .style(iced::theme::Text::Color(self.palette().secondary.base.text)),
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
                            self.palette().success.base.color
                        } else {
                            self.palette().secondary.base.text
                        })),
                        Space::with_width(Length::Fixed(8.0)),
                        text(if self.metadata.selected_book.is_some() {
                            format!("Editing: {}", self.metadata.editing_title)
                        } else {
                            "No book selected".to_string()
                        })
                        .size(13)
                        .style(iced::theme::Text::Color(self.palette().background.weak.text)),
                        if self.metadata.selected_book.is_some() {
                            Element::from(
                                button("×")
                                    .on_press(Message::CloseBook)
                                    .style(iced::theme::Button::Destructive)
                                    .padding([4, 10])
                            )
                        } else {
                            Space::with_width(Length::Fixed(0.0)).into()
                        },
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10)
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
