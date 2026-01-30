use crate::ui::{Lectern, Message};
use crate::ui::cover_search::{search_cover_art, download_image, download_images_parallel_threaded};
use crate::ui::views::ViewMode;
use iced::Command;

pub fn handle_cover(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::BrowseCoverImage => {
            Some(Command::perform(async move {
                let (tx, rx) = futures::channel::oneshot::channel();
                std::thread::spawn(move || {
                    let dialog = rfd::FileDialog::new()
                        .add_filter("Image Files", &["jpg", "jpeg", "png", "gif", "webp", "bmp"])
                        .add_filter("All Files", &["*"]);
                    let result = dialog.pick_file()
                        .map(|p| p.to_string_lossy().to_string());
                    let _ = tx.send(result);
                });
                rx.await.unwrap_or(None)
            }, |path| {
                Message::CoverImageSelected(path)
            }))
        }
        Message::CoverImageSelected(Some(path)) => {
            app.cover.cover_image_path = Some(path.clone());
            if let Some(ref mut book) = app.metadata.selected_book {
                book.cover_url = Some(path.clone());
            }
            
            // Try to load and cache the handle for local files
            let path_buf = std::path::Path::new(&path);
            if path_buf.exists() {
                if let Ok(img_data) = std::fs::read(path_buf) {
                    if let Ok(img) = ::image::load_from_memory(&img_data) {
                        let rgba = img.to_rgba8();
                        let (width, height) = rgba.dimensions();
                        let pixels: Vec<u8> = rgba.into_raw();
                        app.cover.cover_image_handle = Some(iced::widget::image::Handle::from_pixels(width, height, pixels));
                    }
                }
            }
            Some(Command::none())
        }
        Message::CoverImageSelected(None) => {
            // User cancelled
            Some(Command::none())
        }
        Message::SearchCover => {
            app.cover.is_searching_cover = true;
            app.cover.cover_search_error = None;
            app.cover.cover_search_results.clear();
            app.cover.cover_search_result_handles.clear();
            
            let title = app.metadata.editing_title.clone();
            let author = app.metadata.editing_author.clone();
            let isbn = if app.metadata.editing_isbn.is_empty() {
                None
            } else {
                Some(app.metadata.editing_isbn.clone())
            };
            let asin = app.metadata.selected_book.as_ref()
                .and_then(|b| b.asin.clone());
            
            println!("[DEBUG] Searching for cover art - Title: '{}', Author: '{}', ASIN: {:?}", title, author, asin);
            
            Some(Command::perform(
                async move {
                    let result = search_cover_art(&title, &author, isbn.as_deref(), asin.as_deref()).await;
                    match &result {
                        Ok(covers) => println!("[DEBUG] Cover search found {} results", covers.len()),
                        Err(e) => println!("[DEBUG] Cover search error: {}", e),
                    }
                    result
                },
                Message::CoverSearchCompleted,
            ))
        }
        Message::CoverSearchCompleted(Ok(results)) => {
            app.cover.is_searching_cover = false;
            app.cover.cover_search_results = results.clone();
            println!("[DEBUG] Cover search completed: {} results displayed", app.cover.cover_search_results.len());

            let urls: Vec<String> = results
                .into_iter()
                .map(|r| r.url)
                .filter(|u| u.starts_with("http://") || u.starts_with("https://"))
                .collect();
            if urls.is_empty() {
                return Some(Command::none());
            }
            Some(Command::perform(
                async move {
                    let join_handle = download_images_parallel_threaded(urls);
                    let results = join_handle.join().unwrap_or_else(|_| vec![]);
                    results
                        .into_iter()
                        .map(|(url, res)| {
                            let handle = match res {
                                Ok(data) => {
                                    if let Ok(img) = ::image::load_from_memory(&data) {
                                        let rgba = img.to_rgba8();
                                        let (width, height) = rgba.dimensions();
                                        let pixels: Vec<u8> = rgba.into_raw();
                                        Ok(iced::widget::image::Handle::from_pixels(width, height, pixels))
                                    } else {
                                        Err("Failed to decode image".to_string())
                                    }
                                }
                                Err(e) => Err(e),
                            };
                            (url, handle)
                        })
                        .collect::<Vec<_>>()
                },
                Message::CoverSearchResultsImagesDownloaded,
            ))
        }
        Message::CoverSearchCompleted(Err(e)) => {
            app.cover.is_searching_cover = false;
            println!("[DEBUG] Cover search error: {}", e);
            app.cover.cover_search_error = Some(e);
            Some(Command::none())
        }
        Message::SelectCover(index) => {
            if let Some(cover) = app.cover.cover_search_results.get(index) {
                app.cover.cover_image_path = Some(cover.url.clone());
                if cover.url.starts_with("http://") || cover.url.starts_with("https://") {
                    if app.cover.cover_image_url_cached.as_ref() != Some(&cover.url) {
                        app.cover.cover_image_data = None;
                        app.cover.cover_image_handle = None;
                        app.cover.cover_image_url_cached = None;
                        app.cover.is_downloading_cover = true;
                        let url_clone = cover.url.clone();
                        if let Some(ref mut book) = app.metadata.selected_book {
                            book.cover_url = Some(cover.url.clone());
                        }
                        return Some(Command::perform(
                            async move {
                                match download_image(&url_clone).await {
                                    Ok((url, data)) => {
                                        if let Ok(img) = ::image::load_from_memory(&data) {
                                            let rgba = img.to_rgba8();
                                            let (width, height) = rgba.dimensions();
                                            let pixels: Vec<u8> = rgba.into_raw();
                                            let handle = iced::widget::image::Handle::from_pixels(width, height, pixels);
                                            Ok((url, data, handle))
                                        } else {
                                            Err("Failed to decode image".to_string())
                                        }
                                    },
                                    Err(e) => Err(e),
                                }
                            },
                            Message::CoverImageDownloaded,
                        ));
                    } else {
                        println!("[DEBUG] Cover image already cached for URL: {}", cover.url);
                        if let Some(ref mut book) = app.metadata.selected_book {
                            book.cover_url = Some(cover.url.clone());
                        }
                    }
                } else {
                    app.cover.cover_image_data = None;
                    app.cover.cover_image_url_cached = None;
                    if let Some(ref mut book) = app.metadata.selected_book {
                        book.cover_url = Some(cover.url.clone());
                    }
                }
            }
            Some(Command::none())
        }
        Message::CoverUrlChanged(url) => {
            let trimmed_url = url.trim();
            println!("[DEBUG] Cover URL changed: '{}'", trimmed_url);
            app.cover.cover_image_path = if trimmed_url.is_empty() {
                None
            } else {
                Some(trimmed_url.to_string())
            };
            app.cover.cover_image_handle = None;
            if let Some(ref url_path) = app.cover.cover_image_path {
                if url_path.starts_with("http://") || url_path.starts_with("https://") {
                    if app.cover.cover_image_url_cached.as_ref() != Some(url_path) {
                        app.cover.cover_image_data = None;
                        app.cover.cover_image_url_cached = None;
                        app.cover.is_downloading_cover = true;
                        let url_clone = url_path.clone();
                        return Some(Command::perform(
                        async move {
                            match download_image(&url_clone).await {
                                Ok((url, data)) => {
                                    if let Ok(img) = ::image::load_from_memory(&data) {
                                        let rgba = img.to_rgba8();
                                        let (width, height) = rgba.dimensions();
                                        let pixels: Vec<u8> = rgba.into_raw();
                                        let handle = iced::widget::image::Handle::from_pixels(width, height, pixels);
                                        Ok((url, data, handle))
                                    } else {
                                        Err("Failed to decode image".to_string())
                                    }
                                },
                                Err(e) => Err(e),
                            }
                        },
                            Message::CoverImageDownloaded,
                        ));
                    } else {
                        println!("[DEBUG] Cover image already cached for URL: {}", url_path);
                    }
                } else {
                    app.cover.cover_image_data = None;
                    app.cover.cover_image_url_cached = None;
                }
            } else {
                app.cover.cover_image_data = None;
                app.cover.cover_image_url_cached = None;
            }
            if let Some(ref mut book) = app.metadata.selected_book {
                book.cover_url = app.cover.cover_image_path.clone();
            }
            println!("[DEBUG] Cover image path set to: {:?}", app.cover.cover_image_path);
            Some(Command::none())
        }
        Message::DownloadCoverImage(url) => {
            app.cover.is_downloading_cover = true;
            Some(Command::perform(
                async move {
                    match download_image(&url).await {
                        Ok((url, data)) => {
                            if let Ok(img) = ::image::load_from_memory(&data) {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let pixels: Vec<u8> = rgba.into_raw();
                                let handle = iced::widget::image::Handle::from_pixels(width, height, pixels);
                                Ok((url, data, handle))
                            } else {
                                Err("Failed to decode image".to_string())
                            }
                        },
                        Err(e) => Err(e),
                    }
                },
                Message::CoverImageDownloaded,
            ))
        }
        Message::CoverImageDownloaded(Ok((url, image_data, handle))) => {
            app.cover.is_downloading_cover = false;
            app.cover.cover_image_data = Some(image_data);
            app.cover.cover_image_url_cached = Some(url.clone());
            app.cover.cover_image_handle = Some(handle);
            
            println!("[DEBUG] Successfully downloaded and cached cover image and handle from: {}", url);
            Some(Command::none())
        }
        Message::CoverImageDownloaded(Err(e)) => {
            app.cover.is_downloading_cover = false;
            println!("[DEBUG] Failed to download cover image: {}", e);
            app.cover.cover_search_error = Some(format!("Failed to download image: {}", e));
            Some(Command::none())
        }
        Message::CoverSearchResultsImagesDownloaded(results) => {
            for (url, res) in results {
                if let Ok(handle) = res {
                    app.cover.cover_search_result_handles.insert(url, handle);
                }
            }
            Some(Command::none())
        }
        Message::SwitchToCover => {
            app.view_mode = ViewMode::Cover;
            // Check if we need to download or load the cover image handle
            if let Some(ref cover_path) = app.cover.cover_image_path {
                if cover_path.starts_with("http://") || cover_path.starts_with("https://") {
                    if app.cover.cover_image_url_cached.as_ref() != Some(cover_path) {
                        // Not in main cache - check search cache
                        if let Some(handle) = app.search.result_covers.get(cover_path).cloned() {
                            println!("[DEBUG] SwitchToCover - Reusing handle from search cache");
                            app.cover.cover_image_handle = Some(handle);
                            app.cover.cover_image_url_cached = Some(cover_path.clone());
                        } else if !app.cover.is_downloading_cover {
                            app.cover.cover_image_data = None;
                            app.cover.cover_image_handle = None;
                            app.cover.cover_image_url_cached = None;
                            app.cover.is_downloading_cover = true;
                            let url_clone = cover_path.clone();
                            return Some(Command::perform(
                                async move {
                                    match download_image(&url_clone).await {
                                        Ok((url, data)) => {
                                            if let Ok(img) = ::image::load_from_memory(&data) {
                                                let rgba = img.to_rgba8();
                                                let (width, height) = rgba.dimensions();
                                                let pixels: Vec<u8> = rgba.into_raw();
                                                let handle = iced::widget::image::Handle::from_pixels(width, height, pixels);
                                                Ok((url, data, handle))
                                            } else {
                                                Err("Failed to decode image".to_string())
                                            }
                                        },
                                        Err(e) => Err(e),
                                    }
                                },
                                Message::CoverImageDownloaded,
                            ));
                        }
                    }
                } else if app.cover.cover_image_handle.is_none() {
                    // Local file and no handle - try to load it
                    let path = std::path::Path::new(cover_path);
                    if path.exists() {
                        if let Ok(img_data) = std::fs::read(path) {
                            if let Ok(img) = ::image::load_from_memory(&img_data) {
                                let rgba = img.to_rgba8();
                                let (width, height) = rgba.dimensions();
                                let pixels: Vec<u8> = rgba.into_raw();
                                app.cover.cover_image_handle = Some(iced::widget::image::Handle::from_pixels(width, height, pixels));
                                println!("[DEBUG] SwitchToCover - Loaded local handle");
                            }
                        }
                    }
                }
            }
            Some(Command::none())
        }
        _ => None,
    }
}
