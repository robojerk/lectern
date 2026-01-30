use crate::ui::{Message, Lectern};
use crate::ui::colors;
use crate::utils::time::format_time;
use iced::widget::{button, checkbox, column, container, row, scrollable, text, text_input, tooltip, Column, Space, Image};
use iced::widget::tooltip::Position;
use iced::{Alignment, Element, Length, Point};
use iced::widget::canvas::{Canvas, Frame, Path, Program, Stroke};
use iced::Color;

/// Canvas program that draws a rotating arc (loading spinner).
#[derive(Debug)]
struct SpinnerProgram {
    rotation_degrees: f32,
    color: Color,
}

impl Program<Message> for SpinnerProgram {
    type State = ();

    fn draw(
        &self,
        _state: &(),
        renderer: &iced::Renderer,
        _theme: &iced::Theme,
        bounds: iced::Rectangle,
        _cursor: iced::mouse::Cursor,
    ) -> Vec<iced_renderer::Geometry> {
        let mut frame = Frame::new(renderer, bounds.size());
        let center = frame.center();
        let r = (bounds.size().width.min(bounds.size().height) * 0.4) / 2.0;
        let start_rad = self.rotation_degrees.to_radians();
        let end_rad = start_rad + 3.0 * std::f32::consts::PI / 2.0;
        const N: usize = 32;
        let path = Path::new(|p| {
            let start = Point::new(center.x + r * start_rad.cos(), center.y + r * start_rad.sin());
            p.move_to(start);
            for i in 1..=N {
                let angle = start_rad + (i as f32 / N as f32) * (end_rad - start_rad);
                let pt = Point::new(center.x + r * angle.cos(), center.y + r * angle.sin());
                p.line_to(pt);
            }
        });
        frame.stroke(&path, Stroke::default().with_color(self.color).with_width(2.5));
        vec![frame.into_geometry()]
    }
}

/// Fixed row height for virtual list; used to compute visible range from scroll offset.
const CHAPTER_ROW_HEIGHT: f32 = 56.0;
/// Number of rows to render above/below viewport for smooth scrolling.
const VISIBLE_BUFFER: usize = 4;

pub fn view_chapters(app: &Lectern) -> Element<'_, Message> {
        use crate::ui::views::LecternView;
        let tab_bar = app.view_tab_bar();
        
        // ASIN input for chapter lookup - now collapsible
        let asin_input_section: Element<Message> = if app.chapters.show_asin_input {
            container(
                column![
                    text("Audiobook Lookup (Audnexus)")
                        .size(18)
                        .style(iced::theme::Text::Color(app.palette().background.base.text)),
                    Space::with_height(Length::Fixed(10.0)),
                    row![
                        column![
                            text("ASIN or ISBN")
                                .size(12)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            text_input("ASIN or ISBN", &app.chapters.asin_input)
                                .on_input(Message::ChapterAsinChanged)
                                .padding(12),
                        ]
                        .spacing(5)
                        .width(Length::FillPortion(2)),
                        
                        column![
                            text("Region")
                                .size(12)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            iced::widget::pick_list(
                                &crate::ui::state::ChapterRegion::ALL[..],
                                Some(app.chapters.selected_region),
                                Message::ChapterRegionChanged
                            )
                            .padding(12)
                            .width(Length::Fill),
                        ]
                        .spacing(5)
                        .width(Length::FillPortion(1)),
                        
                        button("Search")
                            .on_press(Message::ChapterLookup)
                            .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                            .padding([12, 20]),
                    ]
                    .spacing(15)
                    .align_items(Alignment::End),
                    
                    checkbox("Remove Audible intro and outro from chapters", app.chapters.remove_audible_intro_outro)
                        .on_toggle(Message::ChapterRemoveAudibleToggled)
                        .style(iced::theme::Checkbox::Custom(Box::new(crate::ui::theme::ThemedCheckbox(app.theme_id))))
                        .text_size(14),

                    row![
                        text(format!("Current book ASIN/ISBN: {}", 
                            app.metadata.selected_book.as_ref()
                                .and_then(|b| b.asin.as_ref().or(b.isbn.as_ref()))
                                .map(|s| s.as_str())
                                .unwrap_or("None")))
                            .size(11)
                            .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
                        Space::with_width(Length::Fill),
                        button("Close")
                            .on_press(Message::ChapterToggleAsinInput)
                            .style(iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(app.theme_id)))
                            .padding([8, 16]),
                    ]
                    .align_items(Alignment::Center)
                    .spacing(10),
                ]
                .spacing(15),
            )
            .padding(20)
            .style(iced::theme::Container::Box)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };
        
        // Top controls (Extract/Map/Validate removed; done on open or via Lookup)
        let controls_row = row![
            button("Lookup")
                .on_press(Message::ChapterToggleAsinInput)
                .style(if app.chapters.show_asin_input {
                    iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id))
                } else {
                    iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(app.theme_id))
                })
                .padding([10, 15]),
            button("Remove All")
                .on_press(Message::ChapterRemoveAll)
                .style(iced::theme::Button::custom(crate::ui::theme::RoundedDestructive(app.theme_id)))
                .padding([10, 15]),
        ];
        
        // Shift all: single field (seconds, e.g. "-5" or "2.5") + Apply button
        let shift_controls: Element<Message> = if !app.chapters.chapters.is_empty() {
            row![
                text("Shift all (seconds):")
                    .size(12)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                text_input("e.g. -5 or 2.5", &app.chapters.shift_all_input)
                    .on_input(Message::ChapterShiftAmountChanged)
                    .width(Length::Fixed(100.0))
                    .padding(8),
                button("Shift")
                    .on_press(Message::ChapterShiftAllApply)
                    .style(iced::theme::Button::Secondary)
                    .padding([8, 16]),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };

        // Duration: show "Duration found (Audible)" and "Your audiobook duration" when available
        let lookup_duration_str = app.chapters.lookup_duration_ms
            .map(|ms| format_time(ms, true));
        let book_duration_str = app.chapters.book_duration_ms
            .map(|ms| format_time(ms, true));
        let duration_differ = match (app.chapters.lookup_duration_ms, app.chapters.book_duration_ms) {
            (Some(a), Some(b)) => a != b,
            _ => false,
        };
        let duration_text = match (&lookup_duration_str, &book_duration_str) {
            (Some(ref aud), Some(ref yours)) => format!("Audible: {}  |  Yours: {}", aud, yours),
            (Some(ref aud), None) => format!("Duration found (Audible): {}", aud),
            (None, Some(ref yours)) => format!("Your audiobook duration: {}", yours),
            (None, None) => String::new(),
        };
        let top_controls = controls_row
            .push(Space::with_width(Length::Fill))
            .push(if !duration_text.is_empty() {
                Element::from(
                    text(&duration_text)
                        .size(12)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                )
            } else {
                Space::with_width(Length::Fixed(0.0)).into()
            })
            .push(checkbox("Show seconds", app.chapters.show_seconds)
                .on_toggle(Message::ChaptersShowSecondsToggled)
                .style(iced::theme::Checkbox::Custom(Box::new(crate::ui::theme::ThemedCheckbox(app.theme_id))))
                .text_size(14))
            .spacing(12)
            .align_items(Alignment::Center);

        // Warning when book end time differs from Audible
        let duration_warning: Element<Message> = if duration_differ {
            let (aud_ms, book_ms) = (app.chapters.lookup_duration_ms.unwrap(), app.chapters.book_duration_ms.unwrap());
            let msg = if book_ms < aud_ms {
                format!("Your audiobook duration ({}) is shorter than Audible ({}).", format_time(book_ms, true), format_time(aud_ms, true))
            } else {
                format!("Your audiobook duration ({}) is longer than Audible ({}).", format_time(book_ms, true), format_time(aud_ms, true))
            };
            container(
                row![
                    text("⚠ ")
                        .size(14)
                        .style(iced::theme::Text::Color(colors::WARNING)),
                    text(msg)
                        .size(14)
                        .style(iced::theme::Text::Color(colors::WARNING)),
                ]
                .spacing(6)
                .align_items(Alignment::Center),
            )
            .padding(10)
            .style(iced::theme::Container::Box)
            .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };

        // Chapter list header - styled as a table header (error column for duration violation icon)
        let header = container(
            row![
                text("#")
                    .width(Length::Fixed(50.0))
                    .size(12)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                Space::with_width(Length::Fixed(24.0)),
                text("START")
                    .width(Length::Fixed(230.0))
                    .size(12)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                text("TITLE")
                    .width(Length::Fill)
                    .size(12)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                tooltip(
                    checkbox("", app.chapters.global_locked)
                        .on_toggle(|_| Message::ChaptersGlobalLockToggled)
                        .style(iced::theme::Checkbox::Custom(Box::new(crate::ui::theme::ThemedCheckbox(app.theme_id))))
                        .width(Length::Fixed(30.0)),
                    text("Lock/unlock all chapters"),
                    Position::Bottom,
                ),
                text("Actions")
                    .width(Length::Fixed(320.0))
                    .size(12)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
            ]
            .spacing(10)
            .align_items(Alignment::Center)
        )
        .padding(12)
        .style(iced::theme::Container::Box);
        
        // Chapter list
        let chapter_list_content: Element<Message> = if app.chapters.chapters.is_empty() {
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
            let icons = if app.palette().is_dark { &app.chapter_icons_dark } else { &app.chapter_icons_light };
            let total = app.chapters.chapters.len();
            // Virtual list: only render rows in viewport + buffer to avoid cosmic-text overflow.
            let (start, end) = if let Some((offset_y, viewport_height, _)) = app.chapters.chapter_list_viewport {
                let start_row = (offset_y / CHAPTER_ROW_HEIGHT).floor() as usize;
                let end_row = ((offset_y + viewport_height) / CHAPTER_ROW_HEIGHT).ceil() as usize;
                let start = start_row.saturating_sub(VISIBLE_BUFFER).min(total);
                let end = (end_row + VISIBLE_BUFFER).min(total).max(start);
                (start, end)
            } else {
                (0, (30 + VISIBLE_BUFFER * 2).min(total))
            };
            chapter_list = chapter_list.push(Space::with_height(Length::Fixed(start as f32 * CHAPTER_ROW_HEIGHT)));
            for index in start..end {
                let chapter = &app.chapters.chapters[index];
                let time_str = format_time(chapter.start_time, app.chapters.show_seconds);
                let chapter_index = index;
                let is_locked = chapter.is_locked;
                
                // Get the current editing value for this chapter (if any)
                let current_editing_value = app.chapters.chapter_time_editing.get(&chapter_index)
                    .cloned()
                    .unwrap_or_else(|| time_str.clone());
                
                // Check if playback is active for this chapter - if so, make time clickable to set from timer
                let is_currently_playing_for_time = app.chapter_playback_state.as_ref()
                    .map(|s| s.chapter_index == chapter_index && s.is_playing)
                    .unwrap_or(false);
                
                // Time input - when locked read-only; during playback show time + separate "Set" button to avoid accidental overwrite
                let time_input: Element<Message> = if is_locked {
                    text(&time_str)
                        .width(Length::Fixed(110.0))
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text))
                        .into()
                } else if is_currently_playing_for_time {
                    row![
                        text(&time_str)
                            .width(Length::Fixed(90.0))
                            .size(14)
                            .style(iced::theme::Text::Color(app.palette().background.base.text)),
                        button("Set")
                            .on_press(Message::ChapterSetTimeFromPlayback(chapter_index))
                            .style(iced::theme::Button::Text)
                            .padding([4, 8]),
                    ]
                    .spacing(6)
                    .align_items(Alignment::Center)
                    .into()
                } else {
                    text_input("HH:MM:SS", &current_editing_value)
                        .width(Length::Fixed(110.0))
                        .on_input(move |s| Message::ChapterTimeChanged(chapter_index, s))
                        .into()
                };

                // Title: when locked read-only text; otherwise editable
                let title_input: Element<Message> = if is_locked {
                    text(&chapter.title)
                        .width(Length::Fill)
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text))
                        .into()
                } else {
                    text_input("Chapter title", &chapter.title)
                        .width(Length::Fill)
                        .on_input(move |s| Message::ChapterTitleChanged(chapter_index, s))
                        .into()
                };
                
                // Check if this chapter is currently playing
                let is_currently_playing = app.chapter_playback_state.as_ref()
                    .map(|s| s.chapter_index == chapter_index && s.is_playing)
                    .unwrap_or(false);
                
                                // Get icon handles
                                let lock_icon = icons.get(if is_locked { "lock" } else { "lock_open" });
                                let delete_icon = icons.get("delete");
                                let insert_icon = icons.get("insert");
                                let play_icon = icons.get("play");
                                let stop_icon = icons.get("stop");
                                let remove_icon = icons.get("remove");
                                let add_icon = icons.get("add");
                                let error_icon = icons.get("error");
                                let chapter_exceeds_duration = app.chapters.book_duration_ms
                                    .map(|d| chapter.start_time >= d)
                                    .unwrap_or(false);
                
                // -1s / +1s buttons: when locked show inert icon (no on_press); otherwise editable
                let minus_btn: Element<Message> = if is_locked {
                    let content: Element<Message> = if let Some(h) = remove_icon {
                        Image::new(h.clone()).width(Length::Fixed(20.0)).height(Length::Fixed(20.0)).into()
                    } else {
                        text("-").size(14).into()
                    };
                    container(content)
                        .width(Length::Fixed(28.0))
                        .padding(5)
                        .center_x()
                        .center_y()
                        .into()
                } else if let Some(h) = remove_icon {
                    button(Image::new(h.clone()).width(Length::Fixed(20.0)).height(Length::Fixed(20.0)))
                        .on_press(Message::ChapterTimeAdjusted(chapter_index, -1))
                        .style(iced::theme::Button::Text)
                        .width(Length::Fixed(28.0))
                        .padding(5)
                        .into()
                } else {
                    button("-")
                        .on_press(Message::ChapterTimeAdjusted(chapter_index, -1))
                        .style(iced::theme::Button::Text)
                        .width(Length::Fixed(28.0))
                        .padding(5)
                        .into()
                };
                let plus_btn: Element<Message> = if is_locked {
                    let content: Element<Message> = if let Some(h) = add_icon {
                        Image::new(h.clone()).width(Length::Fixed(20.0)).height(Length::Fixed(20.0)).into()
                    } else {
                        text("+").size(14).into()
                    };
                    container(content)
                        .width(Length::Fixed(28.0))
                        .padding(5)
                        .center_x()
                        .center_y()
                        .into()
                } else if let Some(h) = add_icon {
                    button(Image::new(h.clone()).width(Length::Fixed(20.0)).height(Length::Fixed(20.0)))
                        .on_press(Message::ChapterTimeAdjusted(chapter_index, 1))
                        .style(iced::theme::Button::Text)
                        .width(Length::Fixed(28.0))
                        .padding(5)
                        .into()
                } else {
                    button("+")
                        .on_press(Message::ChapterTimeAdjusted(chapter_index, 1))
                        .style(iced::theme::Button::Text)
                        .width(Length::Fixed(28.0))
                        .padding(5)
                        .into()
                };

                // Delete button: when locked show inert icon (no on_press); otherwise remove chapter
                let delete_btn: Element<Message> = if is_locked {
                    let content: Element<Message> = if let Some(icon_handle) = delete_icon {
                        Image::new(icon_handle.clone())
                            .width(Length::Fixed(20.0))
                            .height(Length::Fixed(20.0))
                            .into()
                    } else {
                        text("🗑").size(14).into()
                    };
                    tooltip(
                        container(content)
                            .width(Length::Fixed(40.0))
                            .padding(5)
                            .center_x()
                            .center_y(),
                        text("locked – unlock to remove"),
                        Position::Bottom,
                    )
                    .into()
                } else {
                    let btn: Element<Message> = if let Some(icon_handle) = delete_icon {
                        button(
                            Image::new(icon_handle.clone())
                                .width(Length::Fixed(20.0))
                                .height(Length::Fixed(20.0))
                        )
                        .on_press(Message::ChapterDelete(chapter_index))
                        .width(Length::Fixed(40.0))
                        .padding(5)
                        .style(iced::theme::Button::Text)
                        .into()
                    } else {
                        button("🗑")
                            .on_press(Message::ChapterDelete(chapter_index))
                            .width(Length::Fixed(40.0))
                            .padding(5)
                            .style(iced::theme::Button::Text)
                            .into()
                    };
                    tooltip(btn, text("remove chapter"), Position::Bottom).into()
                };

                // Error icon when chapter start is after book end
                let error_indicator: Element<Message> = if chapter_exceeds_duration {
                    if let Some(h) = error_icon {
                        Element::from(
                            Image::new(h.clone())
                                .width(Length::Fixed(20.0))
                                .height(Length::Fixed(20.0)),
                        )
                    } else {
                        text("!").size(12).style(iced::theme::Text::Color(app.palette().danger.base.color)).into()
                    }
                } else {
                    Space::with_width(Length::Fixed(8.0)).into()
                };

                chapter_list = chapter_list.push(
                    row![
                        text(format!("#{}", index + 1))
                            .width(Length::Fixed(50.0))
                            .size(14),
                        error_indicator,
                        row![
                            tooltip(minus_btn, text("-1 second"), Position::Bottom),
                            time_input,
                            tooltip(plus_btn, text("+1 second"), Position::Bottom),
                        ]
                        .spacing(4)
                        .width(Length::Fixed(210.0)),
                        title_input,
                        row![
                            // Lock/Unlock with tooltip
                            tooltip(
                                Element::from(if let Some(icon_handle) = lock_icon {
                                    button(
                                        Image::new(icon_handle.clone())
                                            .width(Length::Fixed(20.0))
                                            .height(Length::Fixed(20.0))
                                    )
                                    .on_press(Message::ChapterLockToggled(chapter_index))
                                    .width(Length::Fixed(40.0))
                                    .padding(5)
                                    .style(iced::theme::Button::Text)
                                } else {
                                    button(if is_locked { "🔒" } else { "🔓" })
                                        .on_press(Message::ChapterLockToggled(chapter_index))
                                        .width(Length::Fixed(40.0))
                                        .padding(5)
                                        .style(iced::theme::Button::Text)
                                }),
                                text(if is_locked { "Unlock chapter (Shift+click for range)" } else { "Lock chapter (Shift+click for range)" }),
                                Position::Bottom,
                            ),
                            delete_btn,
                            tooltip(
                                Element::from(if let Some(icon_handle) = insert_icon {
                                    button(
                                        Image::new(icon_handle.clone())
                                            .width(Length::Fixed(20.0))
                                            .height(Length::Fixed(20.0))
                                    )
                                    .on_press(Message::ChapterInsertBelow(chapter_index))
                                    .width(Length::Fixed(40.0))
                                    .padding(5)
                                    .style(iced::theme::Button::Text)
                                } else {
                                    button("➕")
                                        .on_press(Message::ChapterInsertBelow(chapter_index))
                                        .width(Length::Fixed(40.0))
                                        .padding(5)
                                        .style(iced::theme::Button::Text)
                                }),
                                text("add chapter below"),
                                Position::Bottom,
                            ),
                            // Play/Stop button with timer (no colored square)
                            tooltip(
                            row![
                                if is_currently_playing {
                                    if let Some(icon_handle) = stop_icon {
                                        button(
                                            Image::new(icon_handle.clone())
                                                .width(Length::Fixed(20.0))
                                                .height(Length::Fixed(20.0))
                                        )
                                        .on_press(Message::ChapterStopPlayback)
                                        .width(Length::Fixed(40.0))
                                        .padding(5)
                                        .style(iced::theme::Button::Text)
                                    } else {
                                        button("⏹")
                                            .on_press(Message::ChapterStopPlayback)
                                            .width(Length::Fixed(40.0))
                                            .padding(5)
                                            .style(iced::theme::Button::Text)
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
                                    .style(iced::theme::Button::Text)
                                } else {
                                    button("▶")
                                        .on_press(Message::ChapterPlay(chapter_index))
                                        .width(Length::Fixed(40.0))
                                        .padding(5)
                                        .style(iced::theme::Button::Text)
                                },
                                // Timer display - only visible when this chapter is playing
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
                                        Element::from(
                                            text(timer_text)
                                                .size(12)
                                                .style(iced::theme::Text::Color(app.palette().background.weak.text))
                                                .width(Length::Fixed(50.0)),
                                        )
                                    } else {
                                        Element::from(Space::with_width(Length::Fixed(50.0)))
                                    }
                                } else {
                                    Element::from(Space::with_width(Length::Fixed(50.0)))
                                },
                            ]
                            .spacing(5)
                            .align_items(Alignment::Center),
                            text("playback button"),
                            Position::Bottom,
                            ),
                        ]
                        .spacing(8)
                        .width(Length::Fixed(370.0)),
                    ]
                    .spacing(10)
                    .padding(10)
                );
            }
            chapter_list = chapter_list.push(Space::with_height(Length::Fixed((total - end) as f32 * CHAPTER_ROW_HEIGHT)));
            let total_height = total as f32 * CHAPTER_ROW_HEIGHT;
            let scroll_content = container(chapter_list.spacing(5))
                .width(Length::Fill)
                .height(Length::Fixed(total_height));
            scrollable(scroll_content)
                .id(scrollable::Id::new("chapter_list"))
                .on_scroll(move |v| {
                    let abs = v.absolute_offset();
                    Message::ChapterListViewportChanged {
                        offset_y: abs.y,
                        viewport_height: v.bounds().height,
                        content_height: v.content_bounds().height,
                    }
                })
                .height(Length::Fill)
                .into()
        };
        
        // Loading indicator: spinning arc (canvas) when mapping or looking up
        let loading_icon: Element<Message> = {
            let color = app.palette().primary.base.color;
            Canvas::new(SpinnerProgram {
                rotation_degrees: app.chapters.loading_spinner_rotation,
                color,
            })
            .width(Length::Fixed(24.0))
            .height(Length::Fixed(24.0))
            .into()
        };
        // Status messages and playback timer
        let status: Element<Message> = if app.chapters.is_mapping_from_files {
            row![
                loading_icon,
                Space::with_width(Length::Fixed(8.0)),
                text("Mapping chapters from files...").size(14),
            ]
            .align_items(Alignment::Center)
            .into()
        } else if app.chapters.is_looking_up_chapters {
            row![
                loading_icon,
                Space::with_width(Length::Fixed(8.0)),
                text("Looking up chapters...").size(14),
            ]
            .align_items(Alignment::Center)
            .into()
        } else if let Some(ref error) = app.chapters.lookup_error {
            text(format!("Error: {}", error)).size(14).into()
        } else {
            let mut status_text = format!("{} chapters", app.chapters.chapters.len());
            
            // Add playback timer if playing
            if let Some(ref state) = app.chapter_playback_state {
                if state.is_playing {
                    let elapsed_sec = state.elapsed_ms as f64 / 1000.0;
                    let chapter = &app.chapters.chapters[state.chapter_index];
                    let max_duration = chapter.duration.min(30000) as f64 / 1000.0;
                    status_text = format!("▶ Playing Chapter {}: {:.1}s / {:.1}s", 
                        state.chapter_index + 1, elapsed_sec, max_duration);
                }
            }
            
            text(status_text).size(14).into()
        };
        
        // When lookup result is pending, show full-window lookup results (Apply / Map titles / Cancel).
        // Otherwise show normal chapter tab (controls, list, status).
        let main_content: Element<Message> = if let Some(ref lookup) = app.chapters.lookup_result {
            let mut list = Column::new().spacing(6);
            for (i, ch) in lookup.iter().enumerate() {
                list = list.push(
                    container(
                        row![
                            text(format!("{}", i + 1))
                                .width(Length::Fixed(44.0))
                                .size(14)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            text(format_time(ch.start_time, true))
                                .width(Length::Fixed(110.0))
                                .size(14)
                                .style(iced::theme::Text::Color(app.palette().background.base.text)),
                            text(&ch.title)
                                .size(14)
                                .style(iced::theme::Text::Color(app.palette().background.base.text)),
                        ]
                        .spacing(12)
                        .align_items(Alignment::Center),
                    )
                    .padding(8)
                    .style(iced::theme::Container::Box)
                    .width(Length::Fill)
                );
            }
            column![
                text("Chapters from Audible")
                    .size(20)
                    .style(iced::theme::Text::Color(app.palette().background.base.text)),
                text(format!("{} chapters with timestamps — choose an action below.", lookup.len()))
                    .size(14)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                scrollable(list)
                    .height(Length::Fill),
                row![
                    button("Apply Chapters")
                        .on_press(Message::ChapterLookupApply)
                        .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                        .padding([12, 20]),
                    button("Map Chapter Titles")
                        .on_press(Message::MapChapterTitlesOnly)
                        .style(iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(app.theme_id)))
                        .padding([12, 20]),
                    button("Cancel")
                        .on_press(Message::ChapterLookupCancel)
                        .style(iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(app.theme_id)))
                        .padding([12, 20]),
                ]
                .spacing(12)
                .align_items(Alignment::Center),
            ]
            .spacing(16)
            .padding(20)
            .into()
        } else {
            column![
                Space::with_height(Length::Fixed(6.0)),
                asin_input_section,
                Space::with_height(Length::Fixed(6.0)),
                top_controls,
                duration_warning,
                shift_controls,
                Space::with_height(Length::Fixed(6.0)),
                header,
                chapter_list_content,
                Space::with_height(Length::Fixed(6.0)),
                status,
            ]
            .spacing(6)
            .into()
        };

        container(
            column![
                tab_bar,
                main_content,
            ]
            .spacing(6)
            .height(Length::Fill),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}
