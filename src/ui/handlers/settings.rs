use crate::ui::theme::build_theme;
use crate::ui::theme::palette_for;
use crate::ui::theme_settings;
use crate::ui::{Lectern, Message};
use iced::Command;

fn refresh_palette_cache(app: &mut Lectern) {
    app.cached_palette = Some(
        build_theme(app.theme_id, app.dark_mode, app.accent_override).1,
    );
}

fn save_theme_settings(app: &Lectern) {
    theme_settings::save(app.theme_id, app.dark_mode, app.accent_override);
}

pub fn handle_settings(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::ThemeIdChanged(theme_id) => {
            app.theme_id = theme_id;
            refresh_palette_cache(app);
            save_theme_settings(app);
            Some(Command::none())
        }
        Message::DarkModeToggled(dark) => {
            app.dark_mode = dark;
            refresh_palette_cache(app);
            save_theme_settings(app);
            Some(Command::none())
        }
        Message::AccentColorChanged(accent) => {
            app.accent_override = accent;
            app.accent_hex_input = accent
                .map(theme_settings::color_to_hex_export)
                .unwrap_or_default();
            refresh_palette_cache(app);
            save_theme_settings(app);
            Some(Command::none())
        }
        Message::AccentHexInputChanged(s) => {
            app.accent_hex_input = s.clone();
            if let Some(c) = theme_settings::parse_accent_hex(&s) {
                app.accent_override = Some(c);
                refresh_palette_cache(app);
                save_theme_settings(app);
            }
            Some(Command::none())
        }
        Message::UseThemeDefaultAccentToggled(use_default) => {
            if use_default {
                app.accent_override = None;
                app.accent_hex_input.clear();
            } else if app.accent_override.is_none() {
                let palette = palette_for(app.theme_id, app.dark_mode);
                app.accent_override = Some(palette.primary);
                app.accent_hex_input = theme_settings::color_to_hex_export(palette.primary);
            }
            refresh_palette_cache(app);
            save_theme_settings(app);
            Some(Command::none())
        }
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
