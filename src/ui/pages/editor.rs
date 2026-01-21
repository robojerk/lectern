use gtk4::prelude::*;
use gtk4::{
    Box, Button, Entry, Grid, Image, Label, Orientation, ScrolledWindow, 
    Align, ColumnView, ColumnViewColumn, SignalListItemFactory,
    SingleSelection, gio, CheckButton, EditableLabel, Notebook, ListBox
};
use crate::models::metadata::BookMetadata;
use crate::models::chapter::ChapterObject;

#[derive(Clone)]
pub struct EditorPage {
    pub tx: glib::Sender<crate::app_event::AppEvent>,
    pub container: Box,
    pub notebook: Notebook,
    pub back_btn: Button,
    pub folder_path_label: Label,
    pub title_entry: Entry,
    pub subtitle_entry: Entry,
    pub author_entry: Entry,
    pub series_entry: Entry,
    pub series_num_entry: Entry,
    pub narrator_entry: Entry,
    pub publisher_entry: Entry,
    pub language_entry: Entry,
    pub release_date_entry: Entry,
    pub asin_entry: Entry,
    pub genres_entry: Entry,
    pub disk_num_entry: Entry,
    pub chapter_num_entry: Entry,
    pub quality_entry: Entry,
    pub abridged_check: CheckButton,
    pub description_view: gtk4::TextView,
    pub cover_image: Image,
    pub change_cover_btn: Button,
    pub restore_cover_btn: Button,
    pub search_btn: Button,
    pub convert_btn: Button,
    pub chapter_store: gio::ListStore,
    pub global_shift_entry: Entry,
    pub apply_shift_btn: Button,
    pub fetch_chapters_btn: Button,
    pub audible_locale_combo: gtk4::DropDown,
    pub fetch_audnexus_btn: Button,
    pub titles_from_files_btn: Button,
    pub asin_lookup_btn: Button,
    pub title_search_btn: Button,
    pub match_search_entry: Entry,
    pub match_search_btn: Button,
    pub match_results_list: ListBox,
    pub progress_bar: gtk4::ProgressBar,
    pub log_view: gtk4::TextView,
}

impl EditorPage {
    pub fn new(tx: glib::Sender<crate::app_event::AppEvent>) -> Self {
        let container = Box::new(Orientation::Vertical, 12);
        container.set_margin_start(20);
        container.set_margin_end(20);
        container.set_margin_top(20);
        container.set_margin_bottom(20);

        // --- Toolbar ---
        let toolbar = Box::new(Orientation::Horizontal, 12);
        
        let back_btn = Button::builder()
            .label("⬅ Back")
            .build();
        toolbar.append(&back_btn);

        let title_box = Box::new(Orientation::Vertical, 2);
        let page_title = Label::builder()
            .label("Edit Audiobook")
            .css_classes(["title"])
            .halign(Align::Start)
            .build();
        
        let folder_path_label = Label::builder()
            .label("")
            .css_classes(["dim-label", "caption"])
            .halign(Align::Start)
            .build();
            
        title_box.append(&page_title);
        title_box.append(&folder_path_label);
        
        toolbar.append(&title_box);
        title_box.set_hexpand(true);
        
        container.append(&toolbar);

        // --- Create Notebook (Tabs) ---
        let notebook = Notebook::new();
        notebook.set_vexpand(true);

        // ========== TAB 1: METADATA ==========
        let metadata_box = Box::new(Orientation::Vertical, 12);
        metadata_box.set_margin_start(20);
        metadata_box.set_margin_end(20);
        metadata_box.set_margin_top(20);
        metadata_box.set_margin_bottom(20);

        let grid = Grid::builder()
            .row_spacing(10)
            .column_spacing(15)
            .build();

        let title_entry = Entry::builder().placeholder_text("Title").hexpand(true).build();
        let subtitle_entry = Entry::builder().placeholder_text("Subtitle").hexpand(true).build();
        let author_entry = Entry::builder().placeholder_text("Author").hexpand(true).build();
        let series_entry = Entry::builder().placeholder_text("Series").hexpand(true).build();
        let series_num_entry = Entry::builder().placeholder_text("#").width_chars(5).build();
        let narrator_entry = Entry::builder().placeholder_text("Narrator").hexpand(true).build();
        let publisher_entry = Entry::builder().placeholder_text("Publisher").hexpand(true).build();
        let language_entry = Entry::builder().placeholder_text("Language").hexpand(true).build();
        let release_date_entry = Entry::builder().placeholder_text("Release Date (YYYY-MM-DD)").hexpand(true).build();
        let asin_entry = Entry::builder().placeholder_text("ASIN").hexpand(true).build();
        let genres_entry = Entry::builder().placeholder_text("Genres (comma separated)").hexpand(true).build();
        let disk_num_entry = Entry::builder().placeholder_text("Disk #").width_chars(8).build();
        let chapter_num_entry = Entry::builder().placeholder_text("Ch #").width_chars(8).build();
        let quality_entry = Entry::builder().placeholder_text("Quality").width_chars(10).build();
        let abridged_check = CheckButton::with_label("Abridged");
        
        // Description (Multi-line)
        let description_view = gtk4::TextView::new();
        description_view.set_wrap_mode(gtk4::WrapMode::Word);
        description_view.set_vexpand(true);
        let description_scroll = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .min_content_height(120)
            .child(&description_view)
            .build();

        grid.attach(&Label::new(Some("Title:")), 0, 0, 1, 1);
        let title_box = Box::new(Orientation::Horizontal, 8);
        title_box.append(&title_entry);
        let title_search_btn = Button::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Search Audible for this title")
            .build();
        title_box.append(&title_search_btn);
        grid.attach(&title_box, 1, 0, 1, 1);

        grid.attach(&Label::new(Some("Subtitle:")), 0, 1, 1, 1);
        grid.attach(&subtitle_entry, 1, 1, 1, 1);
        grid.attach(&Label::new(Some("Author:")), 0, 2, 1, 1);
        grid.attach(&author_entry, 1, 2, 1, 1);
        
        grid.attach(&Label::new(Some("Series:")), 0, 3, 1, 1);
        let series_box = Box::new(Orientation::Horizontal, 8);
        series_box.append(&series_entry);
        series_box.append(&Label::new(Some("№:")));
        series_box.append(&series_num_entry);
        grid.attach(&series_box, 1, 3, 1, 1);

        grid.attach(&Label::new(Some("Narrator:")), 0, 4, 1, 1);
        grid.attach(&narrator_entry, 1, 4, 1, 1);
        
        grid.attach(&Label::new(Some("Publisher:")), 0, 5, 1, 1);
        grid.attach(&publisher_entry, 1, 5, 1, 1);
        
        grid.attach(&Label::new(Some("Lang/Date:")), 0, 6, 1, 1);
        let lang_date_box = Box::new(Orientation::Horizontal, 8);
        lang_date_box.append(&language_entry);
        lang_date_box.append(&release_date_entry);
        lang_date_box.append(&quality_entry);
        grid.attach(&lang_date_box, 1, 6, 1, 1);
        
        grid.attach(&Label::new(Some("ASIN:")), 0, 7, 1, 1);
        let asin_box = Box::new(Orientation::Horizontal, 8);
        asin_box.append(&asin_entry);
        let asin_lookup_btn = Button::builder()
            .icon_name("system-search-symbolic")
            .tooltip_text("Lookup metadata by ASIN")
            .build();
        asin_box.append(&asin_lookup_btn);
        grid.attach(&asin_box, 1, 7, 1, 1);

        grid.attach(&Label::new(Some("Genre:")), 0, 8, 1, 1);
        grid.attach(&genres_entry, 1, 8, 1, 1);

        grid.attach(&Label::new(Some("Disk/Ch:")), 0, 9, 1, 1);
        let disk_ch_box = Box::new(Orientation::Horizontal, 8);
        disk_ch_box.append(&disk_num_entry);
        disk_ch_box.append(&chapter_num_entry);
        disk_ch_box.append(&abridged_check);
        grid.attach(&disk_ch_box, 1, 9, 1, 1);
        
        grid.attach(&Label::new(Some("Description:")), 0, 10, 1, 1);
        grid.attach(&description_scroll, 1, 10, 1, 1);

        let search_btn = Button::builder()
            .label("🔍 Search Metadata")
            .halign(Align::End)
            .css_classes(["suggested-action"])
            .build();
        grid.attach(&search_btn, 1, 11, 1, 1);

        metadata_box.append(&grid);
        
        let metadata_label = Label::new(Some("Metadata"));
        notebook.append_page(&metadata_box, Some(&metadata_label));

        // ========== TAB 2: COVER ==========
        let cover_box = Box::new(Orientation::Vertical, 12);
        cover_box.set_margin_start(20);
        cover_box.set_margin_end(20);
        cover_box.set_margin_top(20);
        cover_box.set_margin_bottom(20);
        cover_box.set_halign(Align::Center);
        cover_box.set_valign(Align::Start);

        let cover_image = Image::builder()
            .icon_name("folder-music-symbolic")
            .pixel_size(200)
            .css_classes(["cover-image"])
            .halign(Align::Center)
            .valign(Align::Center)
            .build();

        cover_box.append(&cover_image);
        
        let cover_btns = Box::new(Orientation::Horizontal, 12);
        cover_btns.set_halign(Align::Center);
        cover_btns.set_margin_top(12);

        let change_cover_btn = Button::with_label("Change Cover");
        let restore_cover_btn = Button::with_label("Restore Original");
        
        cover_btns.append(&change_cover_btn);
        cover_btns.append(&restore_cover_btn);
        cover_box.append(&cover_btns);

        let cover_label = Label::new(Some("Cover"));
        notebook.append_page(&cover_box, Some(&cover_label));

        // ========== TAB 3: CHAPTERS ==========
        let chapters_box = Box::new(Orientation::Vertical, 12);
        chapters_box.set_margin_start(20);
        chapters_box.set_margin_end(20);
        chapters_box.set_margin_top(20);
        chapters_box.set_margin_bottom(20);

        // Global Shift Controls
        let shift_header = Box::new(Orientation::Horizontal, 12);
        let shift_label = Label::new(Some("Global Time Shift (ms):"));
        let global_shift_entry = Entry::builder()
            .placeholder_text("+/- ms")
            .width_chars(10)
            .build();
        let apply_shift_btn = Button::with_label("Apply Shift");
        
        shift_header.append(&shift_label);
        shift_header.append(&global_shift_entry);
        shift_header.append(&apply_shift_btn);
        shift_header.set_halign(Align::Start);
        
        chapters_box.append(&shift_header);

        // Chapter Lookup & Utility Actions
        let action_header = Box::new(Orientation::Horizontal, 12);
        action_header.set_halign(Align::Start);
        
        let titles_from_files_btn = Button::with_label("Filename -> Titles");
        
        let fetch_label = Label::new(Some("Fetch from Audible:"));
        let locales = ["us", "ca", "uk", "au", "de", "fr", "it", "es", "jp", "in"];
        let audible_locale_combo = gtk4::DropDown::from_strings(&locales);
        let fetch_chapters_btn = Button::with_label("Fetch Chapters");
        fetch_chapters_btn.add_css_class("suggested-action");

        let fetch_audnexus_btn = Button::with_label("Fetch from Audnexus");
        fetch_audnexus_btn.add_css_class("suggested-action");

        action_header.append(&titles_from_files_btn);
        action_header.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
        action_header.append(&fetch_audnexus_btn);
        action_header.append(&gtk4::Separator::new(gtk4::Orientation::Vertical));
        action_header.append(&fetch_label);
        action_header.append(&audible_locale_combo);
        action_header.append(&fetch_chapters_btn);

        chapters_box.append(&action_header);
        
        // Column View
        let chapter_store = gio::ListStore::new::<ChapterObject>();
        let selection_model = SingleSelection::new(Some(chapter_store.clone()));
        let chapter_view = ColumnView::new(Some(selection_model));
        chapter_view.set_show_row_separators(true);
        chapter_view.set_show_column_separators(true);

        // 0. Play Column
        let factory_play = SignalListItemFactory::new();
        let tx_play = tx.clone();
        factory_play.connect_setup(move |_, list_item| {
            let btn = Button::builder()
                .icon_name("media-playback-start-symbolic")
                .has_frame(false)
                .build();
            list_item.set_child(Some(&btn));
        });
        factory_play.connect_bind(move |_, list_item| {
            let btn = list_item.child().and_downcast::<Button>().unwrap();
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            let tx_inner = tx_play.clone();
            
            btn.connect_clicked(move |_| {
                let start_time = item.start_time();
                let _ = tx_inner.send(crate::app_event::AppEvent::PlayRequested(start_time));
            });
        });
        let col_play = ColumnViewColumn::new(Some(""), Some(factory_play));
        col_play.set_fixed_width(40);
        chapter_view.append_column(&col_play);

        // 1. Lock Column
        let factory_lock = SignalListItemFactory::new();
        factory_lock.connect_setup(move |_, list_item| {
            let chk = CheckButton::new();
            list_item.set_child(Some(&chk));
        });
        factory_lock.connect_bind(move |_, list_item| {
            let chk = list_item.child().and_downcast::<CheckButton>().unwrap();
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            
            chk.set_active(item.is_locked());
            
            // UI -> Model
            chk.connect_toggled(move |btn| {
                item.set_locked(btn.is_active());
            });
            
            // Model -> UI (for external updates)
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            item.connect_notify_local(Some("is-locked"), move |obj, _| {
                chk.set_active(obj.is_locked());
            });
        });
        let col_lock = ColumnViewColumn::new(Some("🔒"), Some(factory_lock));
        chapter_view.append_column(&col_lock);

        // 2. Title Column (Editable)
        let factory_title = SignalListItemFactory::new();
        factory_title.connect_setup(move |_, list_item| {
            let label = EditableLabel::new("");
            list_item.set_child(Some(&label));
        });
        factory_title.connect_bind(move |_, list_item| {
            let label = list_item.child().and_downcast::<EditableLabel>().unwrap();
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            
            label.set_text(&item.title());
            
            label.connect_notify_local(Some("editing"), move |lbl, _| {
                if !lbl.is_editing() {
                    item.set_title(lbl.text().to_string());
                }
            });

            // Model -> UI
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            item.connect_notify_local(Some("title"), move |obj, _| {
                label.set_text(&obj.title());
            });
        });
        let col_title = ColumnViewColumn::new(Some("Chapter Title"), Some(factory_title));
        col_title.set_expand(true);
        chapter_view.append_column(&col_title);

        // 3. Start Time Column (Editable)
        let factory_start = SignalListItemFactory::new();
        factory_start.connect_setup(move |_, list_item| {
            let label = EditableLabel::new("");
            label.set_width_chars(12);
            list_item.set_child(Some(&label));
        });
        factory_start.connect_bind(move |_, list_item| {
            let label = list_item.child().and_downcast::<EditableLabel>().unwrap();
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            
            label.set_text(&format_ms(item.start_time()));
            
            let tx_start = tx.clone();
            let store_clone = chapter_store.clone();
            label.connect_notify_local(Some("editing"), move |lbl, _| {
                if !lbl.is_editing() {
                    let text = lbl.text();
                    if let Ok(ms) = parse_ms(&text) {
                        let pos = list_item.position();
                        let mut valid = true;
                        
                        // Rule: Must be >= previous start time
                        if pos > 0 {
                            if let Some(prev) = store_clone.item(pos - 1).and_downcast::<ChapterObject>() {
                                if ms < prev.start_time() {
                                    valid = false;
                                }
                            }
                        }
                        
                        if valid {
                            item.set_start_time(ms);
                        } else {
                            let _ = tx_start.send(crate::app_event::AppEvent::Error(
                                "Invalid start time: must be greater than or equal to previous chapter start time".to_string()
                            ));
                            item.notify("start-time"); // Revert
                        }
                    } else {
                        // Revert on bad input
                        item.notify("start-time"); 
                    }
                }
            });

            // Model -> UI
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            item.connect_notify_local(Some("start-time"), move |obj, _| {
                label.set_text(&format_ms(obj.start_time()));
            });
        });
        let col_start = ColumnViewColumn::new(Some("Start"), Some(factory_start));
        chapter_view.append_column(&col_start);

        // 4. Duration Column
        let factory_dur = SignalListItemFactory::new();
        factory_dur.connect_setup(move |_, list_item| {
            let label = Label::new(None);
            list_item.set_child(Some(&label));
        });
        factory_dur.connect_bind(move |_, list_item| {
            let label = list_item.child().and_downcast::<Label>().unwrap();
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            label.set_text(&format_ms(item.duration()));

             // Model -> UI
            let item = list_item.item().and_downcast::<ChapterObject>().unwrap();
            item.connect_notify_local(Some("duration"), move |obj, _| {
                label.set_text(&format_ms(obj.duration()));
            });
        });
        let col_dur = ColumnViewColumn::new(Some("Duration"), Some(factory_dur));
        chapter_view.append_column(&col_dur);

        // Scroll Container
        let chapters_scroll = ScrolledWindow::builder()
            .min_content_height(300)
            .vexpand(true)
            .child(&chapter_view)
            .build();
            
        chapters_box.append(&chapters_scroll);

        let chapters_label = Label::new(Some("Chapters"));
        notebook.append_page(&chapters_box, Some(&chapters_label));

        // ========== TAB 4: MATCH ==========
        let match_box = Box::new(Orientation::Vertical, 12);
        match_box.set_margin_start(20);
        match_box.set_margin_end(20);
        match_box.set_margin_top(20);
        match_box.set_margin_bottom(20);

        let match_controls = Box::new(Orientation::Horizontal, 8);
        let match_search_entry = Entry::builder()
            .placeholder_text("Search for book title or ASIN...")
            .hexpand(true)
            .build();
        let match_search_btn = Button::with_label("🔍 Search");
        match_search_btn.add_css_class("suggested-action");
        
        match_controls.append(&match_search_entry);
        match_controls.append(&match_search_btn);
        
        let match_results_list = ListBox::new();
        match_results_list.set_selection_mode(gtk4::SelectionMode::Single);
        
        let match_scroll = ScrolledWindow::builder()
            .vexpand(true)
            .min_content_height(300)
            .child(&match_results_list)
            .build();
            
        match_box.append(&match_controls);
        match_box.append(&match_scroll);

        let match_label = Label::new(Some("Match"));
        notebook.append_page(&match_box, Some(&match_label));

        // ========== TAB 5: CONVERT ==========
        let convert_box = Box::new(Orientation::Vertical, 12);
        convert_box.set_margin_start(20);
        convert_box.set_margin_end(20);
        convert_box.set_margin_top(20);
        convert_box.set_margin_bottom(20);
        convert_box.set_valign(Align::Start);

        let convert_info = Label::builder()
            .label("Ready to convert your audiobook to M4B format.\n\nThis will:\n• Merge all audio files\n• Apply metadata and cover art\n• Add chapter markers\n• Upload to Audiobookshelf (if configured)")
            .wrap(true)
            .justify(gtk4::Justification::Left)
            .build();
        convert_box.append(&convert_info);

        let convert_btn = Button::builder()
            .label("🚀 Convert & Upload")
            .css_classes(["suggested-action"])
            .halign(Align::Center)
            .margin_top(20)
            .build();
        convert_box.append(&convert_btn);

        let progress_bar = gtk4::ProgressBar::builder()
            .margin_top(20)
            .show_text(true)
            .build();
        convert_box.append(&progress_bar);

        let log_view = gtk4::TextView::builder()
            .editable(false)
            .cursor_visible(false)
            .vexpand(true)
            .margin_top(12)
            .css_classes(["log-view"])
            .build();
        
        let log_scroll = ScrolledWindow::builder()
            .child(&log_view)
            .min_content_height(200)
            .build();
        convert_box.append(&log_scroll);

        let convert_label = Label::new(Some("Convert"));
        notebook.append_page(&convert_box, Some(&convert_label));

        // Add notebook to container
        container.append(&notebook);

        Self {
            container,
            notebook,
            back_btn,
            folder_path_label,
            title_entry,
            subtitle_entry,
            author_entry,
            series_entry,
            series_num_entry,
            narrator_entry,
            publisher_entry,
            language_entry,
            release_date_entry,
            asin_entry,
            genres_entry,
            disk_num_entry,
            chapter_num_entry,
            quality_entry,
            abridged_check,
            description_view,
            cover_image,
            change_cover_btn,
            restore_cover_btn,
            search_btn,
            convert_btn,
            chapter_store,
            global_shift_entry,
            apply_shift_btn,
            fetch_chapters_btn,
            fetch_audnexus_btn,
            audible_locale_combo,
            titles_from_files_btn,
            asin_lookup_btn,
            title_search_btn,
            match_search_entry,
            match_search_btn,
            match_results_list,
            progress_bar,
            log_view,
            tx,
        }
    }

    pub fn append_log(&self, msg: &str) {
        let buffer = self.log_view.buffer();
        let mut iter = buffer.end_iter();
        buffer.insert(&mut iter, &format!("{}\n", msg));
        
        // Auto-scroll to end
        let mark = buffer.create_mark(None, &buffer.end_iter(), false);
        self.log_view.scroll_to_mark(&mark, 0.0, true, 0.0, 1.0);
    }

    pub fn set_progress(&self, fraction: f64, text: Option<&str>) {
        self.progress_bar.set_fraction(fraction);
        if let Some(t) = text {
            self.progress_bar.set_text(Some(t));
        }
    }

    pub fn set_metadata(&self, metadata: &BookMetadata) {
        self.title_entry.set_text(&metadata.title);
        self.subtitle_entry.set_text(metadata.subtitle.as_deref().unwrap_or(""));
        self.author_entry.set_text(&metadata.authors.join(", "));
        self.series_entry.set_text(metadata.series_name.as_deref().unwrap_or(""));
        self.series_num_entry.set_text(metadata.series_number.as_deref().unwrap_or(""));
        self.publisher_entry.set_text(metadata.publisher.as_deref().unwrap_or(""));
        self.language_entry.set_text(metadata.language.as_deref().unwrap_or(""));
        self.release_date_entry.set_text(metadata.release_date.as_deref().unwrap_or(""));
        self.asin_entry.set_text(metadata.asin.as_deref().unwrap_or(""));
        self.disk_num_entry.set_text(metadata.disk_number.as_deref().unwrap_or(""));
        self.chapter_num_entry.set_text(metadata.chapter_number.as_deref().unwrap_or(""));
        self.quality_entry.set_text(metadata.quality.as_deref().unwrap_or(""));
        
        if let Some(genres) = &metadata.genres {
            self.genres_entry.set_text(&genres.join(", "));
        } else {
            self.genres_entry.set_text("");
        }

        self.abridged_check.set_active(metadata.is_abridged);

        if let Some(narrators) = &metadata.narrator_names {
            self.narrator_entry.set_text(&narrators.join(", "));
        } else {
            self.narrator_entry.set_text("");
        }

        if let Some(desc) = &metadata.description {
            self.description_view.buffer().set_text(desc);
        } else {
            self.description_view.buffer().set_text("");
        }
    }
    
    pub fn get_metadata(&self) -> BookMetadata {
        let authors = self.author_entry.text().to_string()
            .split(',')
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
            
        let narrators = self.narrator_entry.text().to_string();
        let narrator_vec = if narrators.is_empty() {
            None
        } else {
            Some(narrators.split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect())
        };

        let buffer = self.description_view.buffer();
        let (start, end) = buffer.bounds();
        let description = buffer.text(&start, &end, false).to_string();

        BookMetadata {
            title: self.title_entry.text().to_string(),
            subtitle: Some(self.subtitle_entry.text().to_string()).filter(|s| !s.is_empty()),
            authors,
            narrator_names: narrator_vec,
            series_name: Some(self.series_entry.text().to_string()).filter(|s| !s.is_empty()),
            series_number: Some(self.series_num_entry.text().to_string()).filter(|s| !s.is_empty()),
            disk_number: Some(self.disk_num_entry.text().to_string()).filter(|s| !s.is_empty()),
            chapter_number: Some(self.chapter_num_entry.text().to_string()).filter(|s| !s.is_empty()),
            publisher: Some(self.publisher_entry.text().to_string()).filter(|s| !s.is_empty()),
            language: Some(self.language_entry.text().to_string()).filter(|s| !s.is_empty()),
            release_date: Some(self.release_date_entry.text().to_string()).filter(|s| !s.is_empty()),
            asin: Some(self.asin_entry.text().to_string()).filter(|s| !s.is_empty()),
            genres: Some(self.genres_entry.text().to_string().split(',').map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()).filter(|v: &Vec<String>| !v.is_empty()),
            quality: Some(self.quality_entry.text().to_string()).filter(|s| !s.is_empty()),
            is_abridged: self.abridged_check.is_active(),
            description: Some(description).filter(|s| !s.is_empty()),
            ..Default::default()
        }
    }
    
    pub fn load_cover_image(&self, image_bytes: &[u8]) {
        use gtk4::gdk_pixbuf::Pixbuf;
        use gtk4::glib::Bytes;
        
        let bytes = Bytes::from(image_bytes);
        if let Ok(pixbuf) = Pixbuf::from_stream_at_scale(
            &gio::MemoryInputStream::from_bytes(&bytes),
            150,
            150,
            true,
            gio::Cancellable::NONE,
        ) {
            self.cover_image.set_from_pixbuf(Some(&pixbuf));
        }
    }

    pub fn clear_search_results(&self) {
        while let Some(child) = self.match_results_list.first_child() {
            self.match_results_list.remove(&child);
        }
    }

    pub fn add_search_result(&self, book: &BookMetadata) {
        let container = Box::new(Orientation::Vertical, 0);
        container.add_css_class("search-row");

        let grid = Grid::builder()
            .column_spacing(12)
            .row_spacing(4)
            .build();

        // Title Row
        let title_text = if book.title.is_empty() { "Unknown Title" } else { &book.title };
        let title_label = Label::builder()
            .label(&format!("📚 {}", title_text))
            .halign(Align::Start)
            .css_classes(["search-row-title"])
            .build();
        grid.attach(&title_label, 0, 0, 2, 1);

        // Author Row
        let author_text = if book.authors.is_empty() { 
            "Unknown Author".to_string() 
        } else { 
            book.authors.join(", ") 
        };
        let author_label = Label::builder()
            .label(&format!("👤 {}", author_text))
            .halign(Align::Start)
            .css_classes(["dim-label"])
            .build();
        grid.attach(&author_label, 0, 1, 1, 1);

        // Details Row (Series, Narrator, Duration)
        let mut details = Vec::new();
        if let Some(series) = &book.series_name {
            if !series.is_empty() {
                details.push(format!("🔗 {}", series));
            }
        }
        if let Some(narrators) = &book.narrator_names {
            if !narrators.is_empty() {
                details.push(format!("🎙️ {}", narrators.join(", ")));
            }
        }
        if let Some(duration) = book.duration_minutes {
            let hours = duration / 60;
            let mins = duration % 60;
            details.push(format!("⏱️ {}h {}m", hours, mins));
        }

        if !details.is_empty() {
            let details_label = Label::builder()
                .label(&details.join("  •  "))
                .halign(Align::Start)
                .css_classes(["dim-label", "caption"])
                .build();
            grid.attach(&details_label, 0, 2, 2, 1);
        }

        container.append(&grid);
        self.match_results_list.append(&container);
    }
}

// Helper functions for time formatting
fn format_ms(ms: u64) -> String {
    let seconds = ms / 1000;
    let hours = seconds / 3600;
    let mins = (seconds % 3600) / 60;
    let secs = seconds % 60;
    let ms_part = ms % 1000;
    
    // format: HH:MM:SS.mmm
    format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, ms_part)
}

fn parse_ms(text: &str) -> Result<u64, String> {
    // very basic parser for now: allow raw ms or HH:MM:SS
    // Try raw number first
    if let Ok(val) = text.parse::<u64>() {
        return Ok(val);
    }
    
    // Try split by :
    let parts: Vec<&str> = text.split(':').collect();
    if parts.len() == 3 {
        let h = parts[0].parse::<u64>().map_err(|_| "bad hour")?;
        let m = parts[1].parse::<u64>().map_err(|_| "bad min")?;
        // s might have decimal
        let s_parts: Vec<&str> = parts[2].split('.').collect();
        let s = s_parts[0].parse::<u64>().map_err(|_| "bad sec")?;
        let ms = if s_parts.len() > 1 {
            // pad/truncate to 3 digits
            let ms_str = format!("{:0<3}", s_parts[1]);
            ms_str[0..3].parse::<u64>().unwrap_or(0)
        } else {
            0
        };
        
        return Ok( (h * 3600 * 1000) + (m * 60 * 1000) + (s * 1000) + ms );
    }
    
    Err("Invalid format".to_string())
}
