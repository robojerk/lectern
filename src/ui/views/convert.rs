use crate::ui::{Message, Lectern};
use crate::ui::colors; // WARNING only
use iced::widget::{button, column, container, row, scrollable, text, text_input, Space, pick_list, checkbox};
use iced::{Alignment, Element, Length};

pub fn view_convert(app: &Lectern) -> Element<'_, Message> {
    use crate::ui::views::LecternView;
    let tab_bar = app.view_tab_bar();

    if app.metadata.selected_book.is_none() {
        return container(
            column![
                tab_bar,
                Space::with_height(Length::Fixed(20.0)),
                text("Convert to M4B")
                    .size(24)
                    .style(iced::theme::Text::Color(app.palette().background.base.text)),
                Space::with_height(Length::Fixed(40.0)),
                text("Please select a book first")
                    .size(18)
                    .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                text("Go to the Metadata tab to select or search for a book")
                    .size(14)
                    .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
            ]
            .spacing(10)
            .padding(20)
            .align_items(Alignment::Center),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .into();
    }

    // Determine output path display
    let output_path_display = if let Some(ref lib_path) = app.local_library_path {
        crate::ui::helpers::apply_media_template(
            &app.media_management_template,
            lib_path,
            &app.metadata.editing_title,
            &app.metadata.editing_author,
            &app.metadata.editing_series,
            &app.metadata.editing_series_number,
            &app.metadata.editing_publish_year,
            &app.metadata.editing_genre,
            &app.metadata.editing_asin,
            &app.metadata.editing_language,
            &app.metadata.editing_tags,
        )
    } else if let Some(ref path) = app.output_path {
        path.clone()
    } else {
        "Not set - will prompt for location".to_string()
    };

    // Conversion result with size comparison
    let conversion_result: Element<'_, Message> = if app.source_size > 0 && app.output_size > 0 {
        let ratio = (app.output_size as f64 / app.source_size as f64) * 100.0;
        let saved = if app.output_size < app.source_size {
            format!("({:.1}% smaller)", 100.0 - ratio)
        } else {
            format!("({:.1}% larger)", ratio - 100.0)
        };

        container(
            column![
                text("Conversion Result")
                    .size(18)
                    .style(iced::theme::Text::Color(app.palette().success.base.color)),
                Space::with_height(Length::Fixed(10.0)),
                row![
                    text("Source Size:")
                        .size(12)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_width(Length::Fixed(10.0)),
                    text(crate::utils::format::format_size(app.source_size)).size(12),
                ],
                row![
                    text("Output Size:")
                        .size(12)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_width(Length::Fixed(10.0)),
                    text(crate::utils::format::format_size(app.output_size)).size(12),
                    Space::with_width(Length::Fixed(10.0)),
                    text(saved).size(12).style(iced::theme::Text::Color(if app.output_size < app.source_size { app.palette().success.base.color } else { colors::WARNING })),
                ],
            ]
            .spacing(8),
        )
        .padding(20)
        .style(iced::theme::Container::Box)
        .into()
    } else {
        Space::with_height(Length::Fixed(0.0)).into()
    };

    // Header section
    let header = column![
        text("Convert to M4B")
            .size(28)
            .style(iced::theme::Text::Color(app.palette().background.base.text)),
        Space::with_height(Length::Fixed(10.0)),
        text("Configure output settings and create your audiobook")
            .size(14)
            .style(iced::theme::Text::Color(app.palette().background.weak.text)),
    ];

    // Presets Section (Inspired by Audiobookshelf)
    let note_text = if app.conversion_codec == "copy" {
        "Note: 'Copy' codec preserves original streams and is extremely fast, but metadata/cover art embedding might not work in all players."
    } else {
        "Re-encoding uses FFmpeg and creates a single consolidated file."
    };
    let note_color = if app.conversion_codec == "copy" {
        colors::WARNING
    } else {
        app.palette().secondary.base.text
    };

    let presets = container(
        column![
            text("Conversion Settings")
                .size(18)
                .style(iced::theme::Text::Color(app.palette().background.base.text)),
            Space::with_height(Length::Fixed(15.0)),
            
            row![
                // Codec Pick List
                column![
                    text("Codec")
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_height(Length::Fixed(5.0)),
                    pick_list(
                        vec!["aac".to_string(), "copy".to_string(), "opus".to_string()],
                        Some(app.conversion_codec.clone()),
                        Message::ConversionCodecChanged
                    )
                    .width(Length::Fixed(120.0)),
                ]
                .spacing(5),
                
                Space::with_width(Length::Fixed(20.0)),
                
                // Bitrate Pick List
                column![
                    text("Bitrate")
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_height(Length::Fixed(5.0)),
                    pick_list(
                        vec!["auto".to_string(), "64k".to_string(), "96k".to_string(), "128k".to_string(), "192k".to_string()],
                        Some(app.conversion_bitrate.clone()),
                        Message::ConversionBitrateChanged
                    )
                    .width(Length::Fixed(120.0)),
                ]
                .spacing(5),
                
                Space::with_width(Length::Fixed(20.0)),

                // Channels Pick List
                column![
                    text("Channels")
                        .size(14)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_height(Length::Fixed(5.0)),
                    pick_list(
                        vec!["auto".to_string(), "1".to_string(), "2".to_string()],
                        Some(app.conversion_channels.clone()),
                        Message::ConversionChannelsChanged
                    )
                    .width(Length::Fixed(120.0)),
                ]
                .spacing(5),
            ]
            .align_items(Alignment::Center),
            
            Space::with_height(Length::Fixed(15.0)),
            
            checkbox(
                "Normalize Volume (slower, re-encodes)",
                app.conversion_normalize_volume
            )
            .style(iced::theme::Checkbox::Custom(Box::new(crate::ui::theme::ThemedCheckbox(app.theme_id))))
            .on_toggle(Message::ConversionNormalizeVolumeToggled)
            .size(18)
            .text_size(14),
            
            text(note_text)
                .size(11)
                .style(iced::theme::Text::Color(note_color))
        ]
        .spacing(10)
    )
    .padding(20)
    .style(iced::theme::Container::Box);

    // Output location section
    let output_location = container(
        column![
            text("Output Location")
                .size(18)
                .style(iced::theme::Text::Color(app.palette().background.base.text)),
            Space::with_height(Length::Fixed(10.0)),
            if app.local_library_path.is_some() {
                Element::from(column![
                    text("Using Local Library path with template")
                        .size(12)
                        .style(iced::theme::Text::Color(app.palette().background.weak.text)),
                    Space::with_height(Length::Fixed(5.0)),
                    text(output_path_display.as_str())
                        .size(12)
                        .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
                ]
                .spacing(5))
            } else {
                Element::from(column![
                    row![
                        text_input("Output file path", &output_path_display)
                            .on_input(|_| Message::OutputPathSelected(None))
                            .width(Length::Fill),
                        Space::with_width(Length::Fixed(10.0)),
                        button("Browse...")
                            .on_press(Message::BrowseOutputPath)
                            .style(iced::theme::Button::custom(crate::ui::theme::RoundedSecondary(app.theme_id)))
                            .padding([10, 15]),
                    ]
                    .spacing(10)
                    .align_items(Alignment::Center)
                ])
            },
        ]
        .spacing(10),
    )
    .padding(20)
    .style(iced::theme::Container::Box);

    // Audio tracks summary section
    let source_summary = if !app.file.audio_file_paths.is_empty() {
        let footer: Element<'_, Message> = if app.file.audio_file_paths.len() > 20 {
            text(format!("... and {} more", app.file.audio_file_paths.len() - 20))
                .size(11)
                .style(iced::theme::Text::Color(app.palette().secondary.base.text))
                .into()
        } else {
            Space::with_height(Length::Fixed(0.0)).into()
        };

        container(
            column![
                text(format!("Source Tracks ({} files)", app.file.audio_file_paths.len()))
                    .size(16)
                    .style(iced::theme::Text::Color(app.palette().background.base.text)),
                Space::with_height(Length::Fixed(5.0)),
                container(
                    column(
                        app.file.audio_file_paths.iter().take(20).map(|path| {
                            let filename = std::path::Path::new(path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or("unknown");
                            text(filename)
                                .size(11)
                                .style(iced::theme::Text::Color(app.palette().background.weak.text))
                                .into()
                        }).collect::<Vec<_>>()
                    )
                    .spacing(5)
                )
                .height(Length::Shrink),
                footer
            ]
            .spacing(5)
        )
        .padding(20)
        .style(iced::theme::Container::Box)
    } else {
        container(text("No audio files selected"))
            .padding(20)
            .style(iced::theme::Container::Box)
    };

    // Action buttons display
    let action_display: Element<'_, Message> = if app.is_converting {
        column![
            text("Converting...")
                .size(20)
                .style(iced::theme::Text::Color(app.palette().success.base.color)),
            text("Check the terminal for detailed progress")
                .size(12)
                .style(iced::theme::Text::Color(app.palette().secondary.base.text)),
        ]
        .spacing(10)
        .align_items(Alignment::Center)
        .into()
    } else if let Some(ref error) = app.conversion_error {
        column![
            text(format!("Error: {}", error))
                .size(14)
                .style(iced::theme::Text::Color(app.palette().danger.base.color)),
            Space::with_height(Length::Fixed(10.0)),
            button("Try Again")
                .on_press(Message::StartConversion)
                .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
                .padding([15, 40]),
        ]
        .spacing(5)
        .align_items(Alignment::Center)
        .into()
    } else {
        column![
            button(
                text("Start Conversion")
                    .size(18)
                    .horizontal_alignment(iced::alignment::Horizontal::Center)
            )
            .on_press(Message::StartConversion)
            .style(iced::theme::Button::custom(crate::ui::theme::RoundedPrimary(app.theme_id)))
            .padding([15, 60])
            .width(Length::Shrink)
        ]
        .align_items(Alignment::Center)
        .into()
    };

    // Main layout
    container(
        column![
            tab_bar,
            scrollable(
                column![
                    Space::with_height(Length::Fixed(20.0)),
                    header,
                    Space::with_height(Length::Fixed(30.0)),
                    
                    // Row 1: Conversion Settings & Metadata Preview (Metadata between Settings and Output)
                    row![
                        presets.width(Length::FillPortion(2)),
                        Space::with_width(Length::Fixed(20.0)),
                        container(
                            column![
                                text("Metadata Preview")
                                    .size(17)
                                    .style(iced::theme::Text::Color(app.palette().background.base.text)),
                                Space::with_height(Length::Fixed(10.0)),
                                text(format!("Title: {}", app.metadata.editing_title)).size(13).style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                text(format!("Author: {}", app.metadata.editing_author)).size(13).style(iced::theme::Text::Color(app.palette().background.weak.text)),
                                text(format!("Chapters: {}", app.chapters.chapters.len())).size(13).style(iced::theme::Text::Color(app.palette().background.weak.text)),
                            ]
                            .spacing(8)
                        )
                        .padding(20)
                        .style(iced::theme::Container::Box)
                        .width(Length::FillPortion(1)),
                    ],
                    
                    Space::with_height(Length::Fixed(20.0)),

                    // Output Location, then Source Tracks directly below
                    output_location,
                    Space::with_height(Length::Fixed(15.0)),
                    source_summary,
                    
                    Space::with_height(Length::Fixed(40.0)),
                    
                    // Action section - Centered at the bottom
                    container(action_display)
                        .align_x(iced::alignment::Horizontal::Center)
                        .width(Length::Fill),
                    
                    Space::with_height(Length::Fixed(20.0)),
                    conversion_result,
                    Space::with_height(Length::Fixed(40.0)),
                ]
                .max_width(1000)
            )
            .width(Length::Fill),
        ]
        .spacing(10)
        .align_items(Alignment::Center),
    )
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}
