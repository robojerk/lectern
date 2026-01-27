use crate::ui::{Message, Lectern};
use crate::ui::views::LecternView;
use crate::ui::colors;
use crate::ui::helpers::format_time;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, Column, Space, image, Image};
use iced::{Alignment, Element, Length, Font};
use iced::widget::image::Handle;
use std::collections::HashMap;

// Load chapter action icons from PNG files
fn load_chapter_icons() -> HashMap<String, Handle> {
    let mut icons = HashMap::new();
    
    // Load each icon file
    let icon_files = vec![
        ("lock", "assets/png/lock_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png"),
        ("lock_open", "assets/png/lock_open_right_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png"),
        ("delete", "assets/png/delete_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png"),
        ("insert", "assets/png/add_row_below_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png"),
        ("play", "assets/png/play_circle_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png"),
        ("pause", "assets/png/pause_circle_24dp_E3E3E3_FILL0_wght400_GRAD0_opsz24.png"),
    ];
    
    for (name, path) in icon_files {
        if let Ok(bytes) = std::fs::read(path) {
            // Load image using image crate and convert to Handle
            match ::image::load_from_memory(&bytes) {
                Ok(img) => {
                    let rgba = img.to_rgba8();
                    let (width, height) = rgba.dimensions();
                    let pixels: Vec<u8> = rgba.into_raw();
                    icons.insert(name.to_string(), Handle::from_pixels(width, height, pixels));
                },
                Err(e) => {
                    eprintln!("[WARNING] Failed to load icon {}: {}", path, e);
                }
            }
        } else {
            // Fallback: create empty handle if file not found
            eprintln!("[WARNING] Icon file not found: {}", path);
        }
    }
    
    icons
}

pub fn view_chapters(app: &Lectern) -> Element<Message> {
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        // ASIN input for chapter lookup
        let asin_input_section = container(
            column![
                text("Enter ASIN to fetch chapters:")
                    .size(16)
                    .style(iced::theme::Text::Color(colors::TEXT_PRIMARY)),
                Space::with_height(Length::Fixed(10.0)),
                row![
                    column![
                        text("ASIN")
                            .size(12)
                            .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                        text_input("ASIN (e.g., B002V02KPU)", &app.chapter_asin_input)
                            .on_input(Message::ChapterAsinChanged)
                            .padding(12),
                    ]
                    .spacing(5)
                    .width(Length::FillPortion(2)),
                    container(
                        text(format!("Current book ASIN: {}", 
                            app.selected_book.as_ref()
                                .and_then(|b| b.asin.as_ref())
                                .map(|a| a.as_str())
                                .unwrap_or("None")))
                            .size(12)
                            .style(iced::theme::Text::Color(colors::TEXT_TERTIARY))
                    )
                    .padding(12)
                    .style(iced::theme::Container::Box)
                    .width(Length::FillPortion(1)),
                ]
                .spacing(15)
                .align_items(Alignment::End),
            ]
            .spacing(10),
        )
        .padding(15)
        .style(iced::theme::Container::Box);
        
        // Top controls
        let mut controls_row = row![
            button("Extract from File")
                .on_press(Message::ChapterExtractFromFile)
                .style(iced::theme::Button::Primary)
                .padding([10, 15]),
            button("Lookup")
                .on_press(Message::ChapterLookup)
                .style(iced::theme::Button::Primary)
                .padding([10, 15]),
            button("Validate")
                .on_press(Message::ChapterValidate)
                .style(iced::theme::Button::Secondary)
                .padding([10, 15]),
            button("Remove All")
                .on_press(Message::ChapterRemoveAll)
                .style(iced::theme::Button::Destructive)
                .padding([10, 15]),
        ];
        
        // Add map files button if audio files are available
        if !app.audio_file_paths.is_empty() {
            let count = app.audio_file_paths.len();
            let btn_label = if count == 1 {
                "Map from 1 File"
            } else {
                "Map from Files"
            };
            controls_row = controls_row.push(
                button(btn_label)
                    .on_press(Message::MapChaptersFromFiles)
                    .style(iced::theme::Button::Primary)
                    .padding([10, 15])
            );
        }
        
        // Add shift controls
        let shift_controls: Element<Message> = if !app.chapters.is_empty() {
            row![
                text("Shift all by:")
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                button("-1s")
                    .on_press(Message::ChapterShiftAll(-1000))
                    .style(iced::theme::Button::Secondary)
                    .padding([5, 10]),
                button("-5s")
                    .on_press(Message::ChapterShiftAll(-5000))
                    .style(iced::theme::Button::Secondary)
                    .padding([5, 10]),
                button("+5s")
                    .on_press(Message::ChapterShiftAll(5000))
                    .style(iced::theme::Button::Secondary)
                    .padding([5, 10]),
                button("+1s")
                    .on_press(Message::ChapterShiftAll(1000))
                    .style(iced::theme::Button::Secondary)
                    .padding([5, 10]),
            ]
            .spacing(8)
            .align_items(Alignment::Center)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };
        
        let top_controls = controls_row
            .push(Space::with_width(Length::Fill))
            .push(checkbox("Show seconds", app.chapters_show_seconds)
                .on_toggle(Message::ChaptersShowSecondsToggled)
                .text_size(14))
            .spacing(12)
            .align_items(Alignment::Center);
        
        // Chapter list header - styled as a table header
        let header = container(
            row![
                text("#")
                    .width(Length::Fixed(50.0))
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text("START")
                    .width(Length::Fixed(150.0))
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                text("TITLE")
                    .width(Length::Fill)
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
                checkbox("", app.chapters_global_locked)
                    .on_toggle(|_| Message::ChaptersGlobalLockToggled)
                    .width(Length::Fixed(30.0)),
                text("Actions")
                    .width(Length::Fixed(320.0))
                    .size(12)
                    .style(iced::theme::Text::Color(colors::TEXT_SECONDARY)),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .padding(12)
        .style(iced::theme::Container::Box);
        
        // Chapter list
        let chapter_list_content: Element<Message> = if app.chapters.is_empty() {
            column![
                Space::with_height(Length::Fixed(50.0)),
                text("No chapters yet. Use 'Lookup' to fetch chapters from a provider, or add them manually.")
                    .size(14)
                    .horizontal_alignment(iced::alignment::Horizontal::Center),
            ]
            .width(Length::Fill)
            .into()
        } else {
            let mut chapter_list = Column::new();
            let icons = load_chapter_icons(); // Load icons once for all chapters
            for (index, chapter) in app.chapters.iter().enumerate() {
                let time_str = format_time(chapter.start_time, app.chapters_show_seconds);
                let chapter_index = index;
                let is_locked = chapter.is_locked;
                
                let time_input = text_input("HH:MM:SS", &time_str)
                    .width(Length::Fixed(120.0))
                    .on_input(move |s| Message::ChapterTimeChanged(chapter_index, s));
                
                let title_input = text_input("Chapter title", &chapter.title)
                    .width(Length::Fill)
                    .on_input(move |s| Message::ChapterTitleChanged(chapter_index, s));
                
                // Check if this chapter is currently playing
                let is_currently_playing = app.chapter_playback_state.as_ref()
                    .map(|s| s.chapter_index == chapter_index && s.is_playing)
                    .unwrap_or(false);
                
                                // Get icon handles
                                let lock_icon = icons.get(if is_locked { "lock" } else { "lock_open" });
                                let delete_icon = icons.get("delete");
                                let insert_icon = icons.get("insert");
                                let play_icon = icons.get("play");
                                let pause_icon = icons.get("pause");
                
                chapter_list = chapter_list.push(
                    row![
                        text(format!("#{}", index + 1))
                            .width(Length::Fixed(50.0))
                            .size(14),
                        row![
                            button("-")
                                .on_press(Message::ChapterTimeAdjusted(chapter_index, -1))
                                .style(iced::theme::Button::Secondary)
                                .width(Length::Fixed(35.0))
                                .padding(5),
                            time_input,
                            button("+")
                                .on_press(Message::ChapterTimeAdjusted(chapter_index, 1))
                                .style(iced::theme::Button::Secondary)
                                .width(Length::Fixed(35.0))
                                .padding(5),
                        ]
                        .spacing(8)
                        .width(Length::Fixed(150.0)),
                        title_input,
                        checkbox("", is_locked)
                            .on_toggle(move |_| Message::ChapterLockToggled(chapter_index))
                            .width(Length::Fixed(30.0)),
                        row![
                            // Lock/Unlock button - using PNG icon
                            if let Some(icon_handle) = lock_icon {
                                button(
                                    Image::new(icon_handle.clone())
                                        .width(Length::Fixed(20.0))
                                        .height(Length::Fixed(20.0))
                                )
                                .on_press(Message::ChapterLockToggled(chapter_index))
                                .width(Length::Fixed(40.0))
                                .padding(5)
                                .style(if is_locked {
                                    iced::theme::Button::Primary
                                } else {
                                    iced::theme::Button::Secondary
                                })
                            } else {
                                button(if is_locked { "🔒" } else { "🔓" })
                                    .on_press(Message::ChapterLockToggled(chapter_index))
                                    .width(Length::Fixed(40.0))
                                    .padding(5)
                                    .style(if is_locked {
                                        iced::theme::Button::Primary
                                    } else {
                                        iced::theme::Button::Secondary
                                    })
                            },
                            // Delete button
                            if let Some(icon_handle) = delete_icon {
                                button(
                                    Image::new(icon_handle.clone())
                                        .width(Length::Fixed(20.0))
                                        .height(Length::Fixed(20.0))
                                )
                                .on_press(Message::ChapterDelete(chapter_index))
                                .width(Length::Fixed(40.0))
                                .padding(5)
                                .style(iced::theme::Button::Destructive)
                            } else {
                                button("🗑")
                                    .on_press(Message::ChapterDelete(chapter_index))
                                    .width(Length::Fixed(40.0))
                                    .padding(5)
                                    .style(iced::theme::Button::Destructive)
                            },
                            // Insert below button
                            if let Some(icon_handle) = insert_icon {
                                button(
                                    Image::new(icon_handle.clone())
                                        .width(Length::Fixed(20.0))
                                        .height(Length::Fixed(20.0))
                                )
                                .on_press(Message::ChapterInsertBelow(chapter_index))
                                .width(Length::Fixed(40.0))
                                .padding(5)
                                .style(iced::theme::Button::Secondary)
                            } else {
                                button("➕")
                                    .on_press(Message::ChapterInsertBelow(chapter_index))
                                    .width(Length::Fixed(40.0))
                                    .padding(5)
                                    .style(iced::theme::Button::Secondary)
                            },
                            // Play/Pause button with timer - shows current state
                            row![
                                if is_currently_playing {
                                    // Show pause icon when playing
                                    if let Some(icon_handle) = pause_icon {
                                        button(
                                            Image::new(icon_handle.clone())
                                                .width(Length::Fixed(20.0))
                                                .height(Length::Fixed(20.0))
                                        )
                                        .on_press(Message::ChapterStopPlayback)
                                        .width(Length::Fixed(40.0))
                                        .padding(5)
                                        .style(iced::theme::Button::Primary)
                                    } else {
                                        button("⏸")
                                            .on_press(Message::ChapterStopPlayback)
                                            .width(Length::Fixed(40.0))
                                            .padding(5)
                                            .style(iced::theme::Button::Primary)
                                    }
                                } else if let Some(icon_handle) = play_icon {
                                    button(
                                        Image::new(icon_handle.clone())
                                            .width(Length::Fixed(20.0))
                                            .height(Length::Fixed(20.0))
                                    )
                                    .on_press(Message::ChapterPlay(chapter_index))
                                    .width(Length::Fixed(40.0))
                                    .padding(5)
                                    .style(iced::theme::Button::Primary)
                                } else {
                                    button("▶")
                                        .on_press(Message::ChapterPlay(chapter_index))
                                        .width(Length::Fixed(40.0))
                                        .padding(5)
                                        .style(iced::theme::Button::Primary)
                                },
                                // Timer display - shows elapsed time when playing
                                if is_currently_playing {
                                    if let Some(ref state) = app.chapter_playback_state {
                                        let elapsed_sec = state.elapsed_ms / 1000;
                                        let timer_text = if elapsed_sec < 60 {
                                            format!("{}s", elapsed_sec)
                                        } else {
                                            let minutes = elapsed_sec / 60;
                                            let seconds = elapsed_sec % 60;
                                            format!("{}m {}s", minutes, seconds)
                                        };
                                        text(timer_text)
                                            .size(12)
                                            .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
                                            .width(Length::Fixed(50.0))
                                    } else {
                                        text("0s")
                                            .size(12)
                                            .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
                                            .width(Length::Fixed(50.0))
                                    }
                                } else {
                                    text("0s")
                                        .size(12)
                                        .style(iced::theme::Text::Color(colors::TEXT_SECONDARY))
                                        .width(Length::Fixed(50.0))
                                },
                            ]
                            .spacing(5)
                            .align_items(Alignment::Center),
                        ]
                        .spacing(8)
                        .width(Length::Fixed(370.0)),
                    ]
                    .spacing(10)
                    .padding(10)
                );
            }
            scrollable(chapter_list.spacing(5)).into()
        };
        
        // Status messages and playback timer
        let status = if app.is_looking_up_chapters {
            text("Looking up chapters...").size(14)
        } else if let Some(ref error) = app.chapter_lookup_error {
            text(format!("Error: {}", error)).size(14)
        } else {
            let mut status_text = format!("{} chapters", app.chapters.len());
            
            // Add playback timer if playing
            if let Some(ref state) = app.chapter_playback_state {
                if state.is_playing {
                    let elapsed_sec = state.elapsed_ms as f64 / 1000.0;
                    let chapter = &app.chapters[state.chapter_index];
                    let max_duration = chapter.duration.min(30000) as f64 / 1000.0;
                    status_text = format!("▶ Playing Chapter {}: {:.1}s / {:.1}s", 
                        state.chapter_index + 1, elapsed_sec, max_duration);
                }
            }
            
            text(status_text).size(14)
        };
        
        container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(15.0)),
                asin_input_section,
                Space::with_height(Length::Fixed(10.0)),
                top_controls,
                shift_controls,
                Space::with_height(Length::Fixed(10.0)),
                header,
                chapter_list_content,
                Space::with_height(Length::Fixed(10.0)),
                status,
            ]
            .spacing(10)
            .padding(20),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
