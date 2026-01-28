use crate::ui::{Lectern, Message};
use crate::ui::views::ViewMode;
use iced::Command;

pub fn handle_navigation(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::SwitchToSearch => {
            app.view_mode = ViewMode::Search;
            Some(Command::none())
        }
        Message::SwitchToMetadata => {
            app.view_mode = ViewMode::Metadata;
            Some(Command::none())
        }
        Message::SwitchToCover => {
            app.view_mode = ViewMode::Cover;
            Some(Command::none())
        }
        Message::SwitchToChapters => {
            app.view_mode = ViewMode::Chapters;
            Some(Command::none())
        }
        Message::SwitchToConvert => {
            app.view_mode = ViewMode::Convert;
            Some(Command::none())
        }
        Message::SwitchToSettings => {
            app.view_mode = ViewMode::Settings;
            Some(Command::none())
        }
        _ => None,
    }
}
