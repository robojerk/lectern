use gtk4::prelude::*;
use gtk4::{Dialog, Entry, Grid, Label, PasswordEntry, ResponseType, FileChooserAction, FileChooserDialog, Button};
use crate::models::settings::AppConfig;

pub fn show_config_dialog(parent: &impl IsA<gtk4::Window>, current_config: Option<&AppConfig>) -> Option<AppConfig> {
    let dialog = Dialog::with_buttons(
        Some("Settings"),
        Some(parent),
        gtk4::DialogFlags::MODAL,
        &[("Cancel", ResponseType::Cancel), ("Save", ResponseType::Ok)],
    );

    let content = dialog.content_area();
    let grid = Grid::builder()
        .row_spacing(10)
        .column_spacing(10)
        .margin_start(20)
        .margin_end(20)
        .margin_top(20)
        .margin_bottom(20)
        .build();

    // --- Media Management Section ---
    let mm_label = Label::new(Some("Media Management"));
    mm_label.add_css_class("title-4");
    grid.attach(&mm_label, 0, 0, 2, 1);

    grid.attach(&Label::new(Some("Local Library:")), 0, 1, 1, 1);
    let library_box = gtk4::Box::new(gtk4::Orientation::Horizontal, 6);
    let library_entry = Entry::builder().hexpand(true).build();
    let library_btn = Button::with_label("Browse...");
    library_box.append(&library_entry);
    library_box.append(&library_btn);
    grid.attach(&library_box, 1, 1, 1, 1);

    grid.attach(&Label::new(Some("Path Template:")), 0, 2, 1, 1);
    let template_entry = Entry::builder()
        .placeholder_text("{Author}/{Title}.m4b")
        .hexpand(true)
        .build();
    grid.attach(&template_entry, 1, 2, 1, 1);

    let template_hint = Label::new(Some("Tokens: {Author}, {Title}, {Series}, {Year}, {SeriesNumber}, {DiskNumber:00}, {ChapterNumber:00}, {Quality}"));
    template_hint.add_css_class("dim-label");
    grid.attach(&template_hint, 1, 3, 1, 1);

    // --- Audiobookshelf Section ---
    let separator = gtk4::Separator::new(gtk4::Orientation::Horizontal);
    separator.set_margin_top(10);
    separator.set_margin_bottom(10);
    grid.attach(&separator, 0, 4, 2, 1);

    let abs_label = Label::new(Some("Audiobookshelf (Optional)"));
    abs_label.add_css_class("title-4");
    grid.attach(&abs_label, 0, 5, 2, 1);

    let host_entry = Entry::builder()
        .placeholder_text("https://audiobookshelf.example.com")
        .hexpand(true)
        .build();
    let token_entry = PasswordEntry::builder()
        .hexpand(true)
        .build();
    let library_id_entry = Entry::builder()
        .placeholder_text("Library ID")
        .hexpand(true)
        .build();

    grid.attach(&Label::new(Some("Server Host:")), 0, 6, 1, 1);
    grid.attach(&host_entry, 1, 6, 1, 1);
    grid.attach(&Label::new(Some("API Token:")), 0, 7, 1, 1);
    grid.attach(&token_entry, 1, 7, 1, 1);
    grid.attach(&Label::new(Some("Library ID:")), 0, 8, 1, 1);
    grid.attach(&library_id_entry, 1, 8, 1, 1);

    // Populate data
    if let Some(config) = current_config {
        host_entry.set_text(&config.abs_host);
        token_entry.set_text(&config.abs_token);
        library_id_entry.set_text(&config.abs_library_id);
        library_entry.set_text(config.local_library.as_deref().unwrap_or(""));
        template_entry.set_text(&config.path_template);
    } else {
        template_entry.set_text("{Author}/{Title}.m4b");
    }

    // Connect browse button
    let dialog_weak = dialog.downgrade();
    let library_entry_clone = library_entry.clone();
    library_btn.connect_clicked(move |_| {
        let Some(dialog) = dialog_weak.upgrade() else { return };
        let browse_dialog = FileChooserDialog::new(
            Some("Select Local Library Folder"),
            Some(&dialog),
            FileChooserAction::SelectFolder,
            &[("Cancel", ResponseType::Cancel), ("Select", ResponseType::Accept)],
        );

        let library_entry_inner = library_entry_clone.clone();
        browse_dialog.connect_response(move |d, resp| {
            if resp == ResponseType::Accept {
                if let Some(file) = d.file() {
                    if let Some(path) = file.path() {
                        library_entry_inner.set_text(&path.to_string_lossy());
                    }
                }
            }
            d.close();
        });
        browse_dialog.show();
    });

    content.append(&grid);
    dialog.show();
    
    // Run event loop until response
    let response = std::rc::Rc::new(std::cell::Cell::new(None));
    let response_clone = response.clone();
    
    dialog.connect_response(move |dialog, resp| {
        response_clone.set(Some(resp));
        dialog.close();
    });
    
    while response.get().is_none() {
        glib::MainContext::default().iteration(true);
    }

    if response.get() == Some(ResponseType::Ok) {
        let local_lib = library_entry.text().to_string();
        let config = AppConfig {
            abs_host: host_entry.text().to_string(),
            abs_token: token_entry.text().to_string(),
            abs_library_id: library_id_entry.text().to_string(),
            local_library: if local_lib.is_empty() { None } else { Some(local_lib) },
            path_template: template_entry.text().to_string(),
        };
        return Some(config);
    }

    None
}
