#[derive(Debug, Clone)]
pub struct CoverResult {
    pub url: String,
    pub width: u32,
    pub height: u32,
    pub source: String, // Provider name
}

// Download image from URL
pub async fn download_image(url: &str) -> Result<(String, Vec<u8>), String> {
    let client = reqwest::Client::new();
    println!("[DEBUG] Downloading image from: {}", url);
    
    let response = client.get(url)
        .timeout(std::time::Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| format!("Failed to download image: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("HTTP error: {}", response.status()));
    }
    
    let bytes = response.bytes().await
        .map_err(|e| format!("Failed to read image data: {}", e))?;
    
    println!("[DEBUG] Successfully downloaded {} bytes from {}", bytes.len(), url);
    Ok((url.to_string(), bytes.to_vec()))
}

// Download multiple images in parallel (non-blocking, runs in background thread)
pub fn download_images_parallel_threaded(urls: Vec<String>) -> std::thread::JoinHandle<Vec<(String, Result<Vec<u8>, String>)>> {
    std::thread::spawn(move || {
        use futures::future::join_all;
        
        // Create a new Tokio runtime in this background thread
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        
        // Create a single HTTP client with connection pooling
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(8))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        
        let mut tasks = Vec::new();
        
        for url in urls {
            let client_clone = client.clone();
            let task = async move {
                let url_clone = url.clone();
                let result = async {
                    let response = client_clone.get(&url)
                        .send()
                        .await
                        .map_err(|e| format!("Failed to download image: {}", e))?;
                    
                    if !response.status().is_success() {
                        return Err(format!("HTTP error: {}", response.status()));
                    }
                    
                    let bytes = response.bytes().await
                        .map_err(|e| format!("Failed to read image data: {}", e))?;
                    
                    Ok(bytes.to_vec())
                }.await;
                
                (url_clone, result)
            };
            tasks.push(task);
        }
        
        rt.block_on(join_all(tasks))
    })
}

pub async fn search_cover_art(
    title: &str,
    author: &str,
    _isbn: Option<&str>,
    asin: Option<&str>,
) -> Result<Vec<CoverResult>, String> {
    let mut results = Vec::new();
    
    // Try Open Library first (has good cover art)
    if !title.is_empty() {
        let query = format!("{} {}", title, author);
        if let Ok(covers) = search_open_library_covers(&query).await {
            results.extend(covers);
        }
    }
    
    // Try Google Books
    if !title.is_empty() {
        let query = format!("{} {}", title, author);
        if let Ok(covers) = search_google_books_covers(&query).await {
            results.extend(covers);
        }
    }
    
    // Try using ASIN/ISBN for more specific results
    if let Some(asin_val) = asin {
        if let Ok(covers) = search_audnexus_cover(asin_val).await {
            results.extend(covers);
        }
    }
    
    if results.is_empty() {
        Err("No cover art found".to_string())
    } else {
        Ok(results)
    }
}

async fn search_open_library_covers(query: &str) -> Result<Vec<CoverResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://openlibrary.org/search.json?q={}&limit=5", 
                     urlencoding::encode(query));
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Open Library request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Open Library returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let mut covers = Vec::new();
    if let Some(docs) = json.get("docs").and_then(|d| d.as_array()) {
        for doc in docs.iter().take(5) {
            if let Some(cover_id) = doc.get("cover_i").and_then(|c| c.as_i64()) {
                let cover_url = format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_id);
                covers.push(CoverResult {
                    url: cover_url,
                    width: 500,
                    height: 500,
                    source: "Open Library".to_string(),
                });
            }
        }
    }
    
    Ok(covers)
}

async fn search_google_books_covers(query: &str) -> Result<Vec<CoverResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://www.googleapis.com/books/v1/volumes?q={}&maxResults=5", 
                     urlencoding::encode(query));
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Google Books request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Google Books returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    let mut covers = Vec::new();
    if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
        for item in items.iter().take(5) {
            if let Some(volume_info) = item.get("volumeInfo") {
                if let Some(image_links) = volume_info.get("imageLinks") {
                    if let Some(thumbnail) = image_links.get("thumbnail").and_then(|t| t.as_str()) {
                        // Replace thumbnail size with large size
                        let large_url = thumbnail.replace("zoom=1", "zoom=5").replace("&edge=curl", "");
                        covers.push(CoverResult {
                            url: large_url.to_string(),
                            width: 1280,
                            height: 1280,
                            source: "Google Books".to_string(),
                        });
                    }
                }
            }
        }
    }
    
    Ok(covers)
}

async fn search_audnexus_cover(asin: &str) -> Result<Vec<CoverResult>, String> {
    let client = reqwest::Client::new();
    let url = format!("https://api.audnex.us/books/{}", asin);
    
    let response = client.get(&url)
        .send()
        .await
        .map_err(|e| format!("Audnexus request failed: {}", e))?;
    
    if !response.status().is_success() {
        return Err(format!("Audnexus returned status: {}", response.status()));
    }
    
    let json: serde_json::Value = response.json().await
        .map_err(|e| format!("Failed to parse response: {}", e))?;
    
    if let Some(image_url) = json.get("image").and_then(|i| i.as_str()) {
        Ok(vec![CoverResult {
            url: image_url.to_string(),
            width: 500,
            height: 500,
            source: "Audnexus".to_string(),
        }])
    } else {
        Ok(Vec::new())
    }
}
