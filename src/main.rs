mod services;
mod models;
mod ui;

use ui::Lectern;
use iced::{Application, Settings, window};

pub fn main() -> iced::Result {
    Lectern::run(Settings {
        window: window::Settings {
            size: iced::Size::new(900.0, 700.0),
            ..Default::default()
        },
        ..Default::default()
    })
}
