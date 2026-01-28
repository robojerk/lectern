use crate::ui::{Lectern, Message};
use iced::Command;

pub fn handle_settings(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::LocalLibraryPathChanged(path) => {
            app.local_library_path = if path.trim().is_empty() {
                None
            } else {
                Some(path)
            };
            Some(Command::none())
        }
        Message::BrowseLocalLibraryPath => {
            Some(Command::perform(async move {
                let (tx, rx) = futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let dialog = rfd::FileDialog::new();
                    let result = dialog.pick_folder()
                        .map(|p| p.to_string_lossy().to_string());
                    let _ = tx.send(result);
                });
                rx.await.unwrap_or(None)
            }, |path| {
                Message::LocalLibraryPathSelected(path)
            }))
        }
        Message::LocalLibraryPathSelected(Some(path)) => {
            app.local_library_path = Some(path);
            Some(Command::none())
        }
        Message::LocalLibraryPathSelected(None) => {
            // User cancelled
            Some(Command::none())
        }
        Message::MediaManagementTemplateChanged(template) => {
            app.media_management_template = template;
            Some(Command::none())
        }
        Message::AudiobookshelfHostChanged(host) => {
            app.audiobookshelf_host = host;
            Some(Command::none())
        }
        Message::AudiobookshelfTokenChanged(token) => {
            app.audiobookshelf_token = token;
            Some(Command::none())
        }
        Message::AudiobookshelfLibraryIdChanged(library_id) => {
            app.audiobookshelf_library_id = library_id;
            Some(Command::none())
        }
        Message::SwitchToSettings => {
            app.view_mode = crate::ui::views::ViewMode::Settings;
            Some(Command::none())
        }
        _ => None,
    }
}
