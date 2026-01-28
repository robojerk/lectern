use crate::ui::{Lectern, Message};
use crate::ui::state::MetadataProvider;
use crate::services::AudioService;
use crate::ui::cover_search::{download_images_parallel_threaded, download_image};
use crate::ui::views::ViewMode;
use iced::Command;

pub fn handle_search(app: &mut Lectern, message: Message) -> Option<Command<Message>> {
    match message {
        Message::SearchQueryChanged(query) => {
            app.search.query = query;
            Some(Command::none())
        }
        Message::SearchAuthorChanged(author) => {
            app.search.author = author;
            Some(Command::none())
        }
        Message::NextPage => {
            let total_pages = (app.search.results.len() + app.search.results_per_page - 1) / app.search.results_per_page;
            if app.search.current_page < total_pages.saturating_sub(1) {
                app.search.current_page += 1;
                // Download covers for new page
                let start_idx = app.search.current_page * app.search.results_per_page;
                let end_idx = (start_idx + app.search.results_per_page).min(app.search.results.len());
                let page_results = &app.search.results[start_idx..end_idx];
                
                let urls_to_download: Vec<String> = page_results.iter()
                    .filter_map(|book| book.cover_url.as_ref())
                    .filter(|url| (url.starts_with("http://") || url.starts_with("https://")) 
                        && !app.search.result_covers.contains_key(*url))
                    .cloned()
                    .collect();
                
                if !urls_to_download.is_empty() {
                    println!("[DEBUG] Starting background download of {} cover images for page {}", urls_to_download.len(), app.search.current_page + 1);
                    let urls_clone = urls_to_download.clone();
                    return Some(Command::perform(
                        async move {
                            let join_handle = download_images_parallel_threaded(urls_clone);
                            let results = join_handle.join().unwrap_or_else(|_| vec![]);
                            
                            // Decode in background
                            results.into_iter().map(|(url, res)| {
                                let res_handle = match res {
                                    Ok(data) => {
                                        if let Ok(img) = ::image::load_from_memory(&data) {
                                            let rgba = img.to_rgba8();
                                            let (width, height) = rgba.dimensions();
                                            let pixels: Vec<u8> = rgba.into_raw();
                                            Ok(iced::widget::image::Handle::from_pixels(width, height, pixels))
                                        } else {
                                            Err("Failed to decode image".to_string())
                                        }
                                    },
                                    Err(e) => Err(e),
                                };
                                (url, res_handle)
                            }).collect::<Vec<_>>()
                        },
                        Message::SearchCoverImagesDownloaded,
                    ));
                }
            }
            Some(Command::none())
        }
        Message::PreviousPage => {
            if app.search.current_page > 0 {
                app.search.current_page -= 1;
                // Download covers for new page
                let start_idx = app.search.current_page * app.search.results_per_page;
                let end_idx = (start_idx + app.search.results_per_page).min(app.search.results.len());
                let page_results = &app.search.results[start_idx..end_idx];
                
                let urls_to_download: Vec<String> = page_results.iter()
                    .filter_map(|book| book.cover_url.as_ref())
                    .filter(|url| (url.starts_with("http://") || url.starts_with("https://")) 
                        && !app.search.result_covers.contains_key(*url))
                    .cloned()
                    .collect();
                
                if !urls_to_download.is_empty() {
                    println!("[DEBUG] Starting background download of {} cover images for page {}", urls_to_download.len(), app.search.current_page + 1);
                    let urls_clone = urls_to_download.clone();
                    return Some(Command::perform(
                        async move {
                            let join_handle = download_images_parallel_threaded(urls_clone);
                            let results = join_handle.join().unwrap_or_else(|_| vec![]);
                            
                            // Decode in background
                            results.into_iter().map(|(url, res)| {
                                let res_handle = match res {
                                    Ok(data) => {
                                        if let Ok(img) = ::image::load_from_memory(&data) {
                                            let rgba = img.to_rgba8();
                                            let (width, height) = rgba.dimensions();
                                            let pixels: Vec<u8> = rgba.into_raw();
                                            Ok(iced::widget::image::Handle::from_pixels(width, height, pixels))
                                        } else {
                                            Err("Failed to decode image".to_string())
                                        }
                                    },
                                    Err(e) => Err(e),
                                };
                                (url, res_handle)
                            }).collect::<Vec<_>>()
                        },
                        Message::SearchCoverImagesDownloaded,
                    ));
                }
            }
            Some(Command::none())
        }
        Message::SearchByAsinToggled(enabled) => {
            app.search.by_asin = enabled;
            Some(Command::none())
        }
        Message::PerformSearch => {
            if app.search.query.trim().is_empty() {
                return Some(Command::none());
            }
            
            app.search.is_searching = true;
            app.search.error = None;
            app.search.results.clear();
            
            let query = app.search.query.clone();
            let author = app.search.author.clone();
            let by_asin = app.search.by_asin;
            let provider = app.metadata.metadata_provider;
            app.search.current_page = 0; // Reset to first page on new search
            
            // Combine query and author if both provided
            let search_query = if !author.trim().is_empty() && !query.trim().is_empty() {
                format!("{} {}", query, author)
            } else if !author.trim().is_empty() {
                author
            } else {
                query
            };
            
            // Spawn async search task
            println!("[DEBUG] Starting search for: '{}' (Author: '{}', ASIN: {}, Provider: {:?})", 
                app.search.query, app.search.author, by_asin, provider);
            Some(Command::perform(
                async move {
                    println!("[DEBUG] Calling search_metadata...");
                    let provider_id = provider.to_id();
                    let provider_opt = if provider == MetadataProvider::Auto {
                        None
                    } else {
                        Some(provider_id.as_str())
                    };
                    let result = AudioService::search_metadata(&search_query, by_asin, provider_opt).await;
                    match &result {
                        Ok(books) => println!("[DEBUG] Search returned {} results", books.len()),
                        Err(e) => println!("[DEBUG] Search error: {}", e),
                    }
                    result
                },
                Message::SearchCompleted,
            ))
        }
        Message::SearchCompleted(Ok(results)) => {
            app.search.is_searching = false;
            app.search.results = results.clone();
            app.view_mode = ViewMode::Search;
            app.search.current_page = 0; // Reset to first page
            
            // Only download covers for the current page (first 10 results)
            let start_idx = app.search.current_page * app.search.results_per_page;
            let end_idx = (start_idx + app.search.results_per_page).min(results.len());
            let page_results = &results[start_idx..end_idx];
            
            // Collect cover URLs for current page only
            let urls_to_download: Vec<String> = page_results.iter()
                .filter_map(|book| book.cover_url.as_ref())
                .filter(|url| (url.starts_with("http://") || url.starts_with("https://")) 
                    && !app.search.result_covers.contains_key(*url))
                .cloned()
                .collect();
            
            // Download covers for current page using async Command (non-blocking)
            if !urls_to_download.is_empty() {
                println!("[DEBUG] Starting async download of {} cover images for page {}", urls_to_download.len(), app.search.current_page + 1);
                let urls_clone = urls_to_download.clone();
                let downloading = app.search.downloading.clone();
                
                // Mark URLs as downloading
                {
                    let mut downloading_list = downloading.lock().unwrap();
                    downloading_list.extend(urls_clone.iter().cloned());
                }
                
                return Some(Command::perform(
                    async move {
                        // Download images in background thread
                        let join_handle = download_images_parallel_threaded(urls_clone);
                        let results = join_handle.join().unwrap_or_else(|_| vec![]);
                        
                        // Decode each image in the background as well
                        results.into_iter().map(|(url, res)| {
                            let res_handle = match res {
                                Ok(data) => {
                                    if let Ok(img) = ::image::load_from_memory(&data) {
                                        let rgba = img.to_rgba8();
                                        let (width, height) = rgba.dimensions();
                                        let pixels: Vec<u8> = rgba.into_raw();
                                        Ok(iced::widget::image::Handle::from_pixels(width, height, pixels))
                                    } else {
                                        Err("Failed to decode image".to_string())
                                    }
                                },
                                Err(e) => Err(e),
                            };
                            (url, res_handle)
                        }).collect::<Vec<_>>()
                    },
                    Message::SearchCoverImagesDownloaded,
                ));
            }
            Some(Command::none())
        }
        Message::SearchCompleted(Err(e)) => {
            app.search.is_searching = false;
            app.search.error = Some(format!("Search failed: {}", e));
            println!("[ERROR] Search failed: {}", e);
            Some(Command::none())
        }
        Message::SelectBook(index) => {
            if let Some(book) = app.search.results.get(index).cloned() {
                println!("[DEBUG] SelectBook - Populating fields from book: '{}' by '{}'", book.title, book.author);
                println!("[DEBUG] SelectBook - Book fields: subtitle={:?}, series={:?}, series_number={:?}, narrator={:?}, description={:?}, isbn={:?}, publisher={:?}, publish_year={:?}, genre={:?}, language={:?}, explicit={:?}, abridged={:?}",
                    book.subtitle, book.series, book.series_number, book.narrator, 
                    book.description.as_ref().map(|d| if d.len() > 50 { format!("{}...", &d[..50]) } else { d.clone() }),
                    book.isbn, book.publisher, book.publish_year, book.genre, book.language, book.explicit, book.abridged);
                
                app.metadata.selected_book = Some(book.clone());
                app.metadata.editing_title = book.title;
                app.metadata.editing_subtitle = book.subtitle.unwrap_or_default();
                app.metadata.editing_author = book.author;
                app.metadata.editing_series = book.series.unwrap_or_default();
                app.metadata.editing_series_number = book.series_number.unwrap_or_default();
                app.metadata.editing_narrator = book.narrator.unwrap_or_default();
                app.metadata.editing_description = book.description.unwrap_or_default();
                app.metadata.editing_description_content = iced::widget::text_editor::Content::with_text(&app.metadata.editing_description);
                app.metadata.editing_isbn = book.isbn.unwrap_or_default();
                app.metadata.editing_asin = book.asin.unwrap_or_default();
                app.metadata.editing_publisher = book.publisher.unwrap_or_default();
                app.metadata.editing_publish_year = book.publish_year.unwrap_or_default();
                app.metadata.editing_genre = book.genre.unwrap_or_default();
                app.metadata.editing_tags = book.tags.unwrap_or_default();
                app.metadata.editing_language = book.language.unwrap_or_default();
                app.metadata.editing_explicit = book.explicit.unwrap_or(false);
                app.metadata.editing_abridged = book.abridged.unwrap_or(false);
                
                println!("[DEBUG] SelectBook - Populated editing fields: subtitle='{}', series='{}', narrator='{}', isbn='{}', publisher='{}', year='{}', genre='{}', language='{}'",
                    app.metadata.editing_subtitle, app.metadata.editing_series, app.metadata.editing_narrator, 
                    app.metadata.editing_isbn, app.metadata.editing_publisher, app.metadata.editing_publish_year, 
                    app.metadata.editing_genre, app.metadata.editing_language);
                // Initialize cover image path
                app.cover.cover_image_path = book.cover_url.clone();
                app.view_mode = ViewMode::Metadata;
                app.search.results.clear(); // Close search view
                // If cover is a URL, check if we already have it cached
                if let Some(ref cover_url) = book.cover_url {
                    if cover_url.starts_with("http://") || cover_url.starts_with("https://") {
                        // First check if it's already in the main cover cache
                        if app.cover.cover_image_url_cached.as_ref() == Some(cover_url) {
                            println!("[DEBUG] Cover image already in main cache");
                        } else if let Some(handle) = app.search.result_covers.get(cover_url).cloned() {
                            // If it's in the search result cache, promote it to main cache
                            println!("[DEBUG] Promoting cover handle from search cache to main cache");
                            app.cover.cover_image_handle = Some(handle);
                            app.cover.cover_image_url_cached = Some(cover_url.clone());
                            // We don't have the raw data here easily but the handle is enough for display
                        } else {
                            // URL changed or not cached - download it
                            app.cover.cover_image_data = None; // Clear old cached data
                            app.cover.cover_image_handle = None;
                            app.cover.cover_image_url_cached = None;
                            app.cover.is_downloading_cover = true;
                            let url_clone = cover_url.clone();
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
                }
            }
            Some(Command::none())
        }
        Message::SearchCoverImagesDownloaded(results) => {
            // Process downloaded cover images
            for (url, result) in results {
                match result {
                    Ok(handle) => {
                        app.search.result_covers.insert(url.clone(), handle);
                        println!("[DEBUG] Cached cover handle from batch download: {}", url);
                    },
                    Err(e) => {
                        println!("[WARNING] Failed to download cover image {}: {}", url, e);
                    }
                }
            }
            // Remove URLs from downloading list
            {
                let mut downloading_list = app.search.downloading.lock().unwrap();
                downloading_list.clear(); // Clear all since we just processed a batch
            }
            Some(Command::none())
        }
        _ => None, // This handler doesn't handle this message
    }
}
