use std::path::{Path, PathBuf};
use tokio::process::Command;
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Clone, Debug)]
pub enum SearchProvider {
    Audnexus,
    GoogleBooks,
    OpenLibrary,
    ITunes,
    FantLab,
}

// --- Data Structures ---

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub narrator_names: Option<Vec<String>>,
    pub series_name: Option<String>,
    pub image_url: String,
    pub asin: String,
    pub duration_minutes: Option<u64>,
    pub release_date: Option<String>,
}

// Convenience methods for QML compatibility
// Note: author() method removed as it's no longer used

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Chapter {
    pub title: String,
    pub start_time: f64, // in seconds
    pub end_time: Option<f64>, // in seconds
    pub locked: bool,
}

#[derive(Clone, Debug)]
pub struct ABSConfig {
    pub host: String,
    pub token: String,
    pub library_id: String,
}

// --- API Response Structures ---

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Some fields may be unused but kept for future API features
pub struct AudnexusAuthorSearchResponse {
    pub asin: String,
    pub name: String,
    // Full author details are only available when fetching by ASIN
    pub description: Option<String>,
    pub image: Option<String>,
    pub region: Option<String>,
    #[serde(default)]
    pub genres: Vec<serde_json::Value>,
    #[serde(default)]
    pub similar: Vec<serde_json::Value>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Some fields may be unused but kept for future API features
pub struct AudnexusBookResponse {
    pub asin: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<serde_json::Value>,
    pub narrators: Option<Vec<serde_json::Value>>,
    pub series: Option<String>,
    pub image: Option<String>,
    pub runtime_length_min: Option<u64>,
    pub release_date: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
    pub publisher_name: Option<String>,
    pub copyright: Option<i32>,
    pub format_type: Option<String>,
    pub is_adult: Option<bool>,
    pub isbn: Option<String>,
    pub literature_type: Option<String>,
    pub rating: Option<String>,
    #[serde(default)]
    pub genres: Vec<serde_json::Value>,
    pub region: String,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Some fields may be unused but kept for future API features
pub struct AudnexusSearchResponse {
    pub asin: String,
    pub authors: Vec<String>,
    pub narrators: Vec<String>,
    pub title: String,
    pub series: Option<String>,
    pub image: Option<String>,
    pub runtime_length_min: Option<u64>,
    pub release_date: Option<String>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Some fields may be unused but kept for future API features
pub struct AudnexusSearchResult {
    pub result: Vec<AudnexusSearchResponse>,
}

// --- Google Books API Structures ---

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBooksVolumeInfo {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Option<Vec<String>>,
    pub published_date: Option<String>,
    pub description: Option<String>,
    pub industry_identifiers: Option<Vec<GoogleBooksIndustryIdentifier>>,
    pub image_links: Option<GoogleBooksImageLinks>,
    pub categories: Option<Vec<String>>,
    pub language: Option<String>,
    pub publisher: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBooksIndustryIdentifier {
    #[serde(rename = "type")]
    pub identifier_type: String,
    pub identifier: String,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBooksImageLinks {
    pub small_thumbnail: Option<String>,
    pub thumbnail: Option<String>,
    pub small: Option<String>,
    pub medium: Option<String>,
    pub large: Option<String>,
    pub extra_large: Option<String>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBooksItem {
    pub volume_info: Option<GoogleBooksVolumeInfo>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct GoogleBooksResponse {
    pub items: Option<Vec<GoogleBooksItem>>,
}

// --- Open Library API Structures ---

#[derive(Deserialize, Debug)]
pub struct OpenLibraryWork {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub authors: Option<Vec<OpenLibraryAuthorRef>>,
    pub first_publish_date: Option<String>,
    pub description: Option<serde_json::Value>, // Can be string or object
    pub covers: Option<Vec<i64>>,
    pub subject_places: Option<Vec<String>>,
    pub subjects: Option<Vec<String>>,
    pub language: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenLibraryAuthorRef {
    pub author: Option<OpenLibraryAuthor>,
    #[serde(rename = "type")]
    pub author_type: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenLibraryAuthor {
    pub key: String,
    pub name: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct OpenLibrarySearchResult {
    pub docs: Vec<OpenLibraryDoc>,
}

#[derive(Deserialize, Debug)]
pub struct OpenLibraryDoc {
    pub key: String,
    pub title: Option<String>,
    pub author_name: Option<Vec<String>>,
    pub first_publish_year: Option<i32>,
    pub cover_i: Option<i64>,
    pub subject: Option<Vec<String>>,
    pub language: Option<Vec<String>>,
    pub publisher: Option<Vec<String>>,
    pub isbn: Option<Vec<String>>,
}

#[derive(Deserialize, Debug)]
#[allow(dead_code)] // Some fields may be unused but kept for future API features
pub struct AudibleProduct {
    pub asin: String,
    pub authors: Vec<String>,
    pub narrators: Vec<String>,
    pub title: String,
    pub series: Option<String>,
    pub image: Option<String>,
    pub runtime_length_min: Option<u64>,
    pub release_date: Option<String>,
}

// --- Chapter Structures ---

// --- Audnexus Chapters API Structures ---

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudnexusChaptersResponse {
    pub asin: String,
    pub brand_intro_duration_ms: Option<i64>,
    pub brand_outro_duration_ms: Option<i64>,
    pub chapters: Vec<AudnexusChapter>,
    pub is_accurate: Option<bool>,
    pub region: String,
    pub runtime_length_ms: i64,
    pub runtime_length_sec: i64,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
pub struct AudnexusChapter {
    pub length_ms: i64,
    pub start_offset_ms: i64,
    pub start_offset_sec: i64,
    pub title: String,
}

// --- Audio Service Implementation ---

#[allow(dead_code)] // Never constructed - used only for associated functions
pub struct AudioService;

impl AudioService {
    /// Parse folder name to extract potential search terms for metadata
    pub fn parse_folder_name_for_search(folder_path: &str) -> Option<String> {
        use std::path::Path;

        let path = Path::new(folder_path);
        let folder_name = path.file_name()?.to_str()?;

        // Common audiobook folder naming patterns:
        // 1. "Author - Title"
        // 2. "Title - Author"
        // 3. "Author/Title"
        // 4. "Title [Author]"
        // 5. Remove file extensions, part numbers, etc.

        let mut search_term = folder_name.to_string();

        // Remove common prefixes/suffixes that aren't part of the title
        search_term = search_term
            .replace("Re__", "")  // Remove "Re__" prefix
            .replace("__", "_")   // Clean up double underscores
            .replace("_[", " [")  // Fix spacing before brackets
            .replace("]_", "] ")  // Fix spacing after brackets
            .replace("_-_", " - ") // Fix dash separators
            .trim()
            .to_string();

        // Try to extract author name - look for patterns like "REQ - Author - [" or " [Author] "
        let author_patterns = [
            r"REQ\s*-\s*([^-\[\]]+)\s*-\s*\[",  // "REQ - Author - ["
            r" - ([^-\[\]]+) - ",              // " - Author - "
            r" \[([^\]]+)\]",                   // " [Author]"
        ];

        for pattern in &author_patterns {
            if let Ok(regex) = regex::Regex::new(pattern) {
                if let Some(captures) = regex.captures(&search_term) {
                    if let Some(author_match) = captures.get(1) {
                        let mut author = author_match.as_str().trim().to_string();
                        // Clean up underscores
                        author = author.replace("_", " ");
                        // If we found an author, use it as the primary search term
                        if !author.is_empty() && author.len() > 2 && author.chars().any(|c| c.is_alphabetic()) {
                            return Some(author);
                        }
                    }
                }
            }
        }

        // If no clear author found, clean up the folder name and use it as a general search
        // Remove part numbers, file extensions, etc.
        let cleaned = regex::Regex::new(r"\s*\[\d+[_-]\d+\]\s*$")
            .unwrap()
            .replace(&search_term, "")
            .to_string();

        let cleaned = regex::Regex::new(r"\s*\(\d+\)\s*$")
            .unwrap()
            .replace(&cleaned, "")
            .to_string();

        if cleaned.len() > 3 {
            Some(cleaned.trim().to_string())
        } else {
            None
        }
    }

    /// Validate that a search query looks reasonable for metadata APIs
    fn is_valid_search_query(query: &str) -> bool {
        let query = query.trim();

        // Must be at least 3 characters
        if query.len() < 3 {
            return false;
        }

        // Should not be a file path (contain / or \)
        if query.contains('/') || query.contains('\\') {
            return false;
        }

        // Should not start with common file path indicators
        if query.starts_with("./") || query.starts_with("../") || query.starts_with("~/") {
            return false;
        }

        // Should not be just numbers (likely not a book title)
        if query.chars().all(|c| c.is_numeric() || c.is_whitespace()) {
            return false;
        }

        // Should contain at least some letters
        if !query.chars().any(|c| c.is_alphabetic()) {
            return false;
        }

        true
    }

    /// Fetch metadata using the specified search provider
    pub async fn fetch_metadata_with_provider(query: &str, provider: SearchProvider) -> Result<BookMetadata, String> {
        // Validate the query before proceeding
        if !Self::is_valid_search_query(query) {
            return Err(format!("Invalid search query: '{}'. Query must be a book title, author name, or ASIN, not a file path.", query));
        }

        let client = Client::new();

        match provider {
            SearchProvider::Audnexus => Self::search_audnexus(&client, query).await,
            SearchProvider::GoogleBooks => Self::search_google_books(&client, query).await,
            SearchProvider::OpenLibrary => Self::search_open_library(&client, query).await,
            SearchProvider::ITunes => Self::search_itunes(&client, query).await,
            SearchProvider::FantLab => Self::search_fantlab(&client, query).await,
        }
    }

    /// Fetch metadata from multiple providers (tries each one until success)
    pub async fn fetch_metadata(query: &str) -> Result<BookMetadata, String> {
        // Validate the query before proceeding
        if !Self::is_valid_search_query(query) {
            return Err(format!("Invalid search query: '{}'. Query must be a book title, author name, or ASIN, not a file path.", query));
        }

        let client = Client::new();
        let providers = vec![
            SearchProvider::Audnexus,
            SearchProvider::GoogleBooks,
            SearchProvider::OpenLibrary,
        ];

        for provider in providers {
            match Self::fetch_metadata_with_provider(query, provider.clone()).await {
                Ok(metadata) => return Ok(metadata),
                Err(e) => println!("{} search failed: {}", Self::provider_name(&provider), e),
            }
        }

        Err(format!("All search providers failed for query: '{}'", query))
    }

    pub fn provider_name(provider: &SearchProvider) -> &'static str {
        match provider {
            SearchProvider::Audnexus => "Audnexus",
            SearchProvider::GoogleBooks => "Google Books",
            SearchProvider::OpenLibrary => "Open Library",
            SearchProvider::ITunes => "iTunes",
            SearchProvider::FantLab => "FantLab",
        }
    }

    /// Search for authors using Audnexus API (based on Audiobookshelf implementation)
    async fn search_audnexus_authors(client: &Client, name: &str, region: &str) -> Result<Vec<AudnexusAuthorSearchResponse>, String> {
        // Rate limiting: wait 150ms between requests (based on Audiobookshelf's 1 request per 150ms limit)
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let mut url = format!("https://api.audnex.us/authors?name={}", urlencoding::encode(name));
        if !region.is_empty() && region != "us" {
            url.push_str(&format!("&region={}", region));
        }

        println!("Audnexus: Searching for author '{}'", name);

        let response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Audnexus API returned status: {}", response.status()));
        }

        let authors: Vec<AudnexusAuthorSearchResponse> = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        Ok(authors)
    }

    /// Get book details by ASIN
    async fn get_audnexus_book(client: &Client, asin: &str, region: &str) -> Result<AudnexusBookResponse, String> {
        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let mut url = format!("https://api.audnex.us/books/{}", urlencoding::encode(asin));
        if !region.is_empty() && region != "us" {
            url.push_str(&format!("&region={}", region));
        }

        println!("Audnexus: Getting book details for ASIN '{}'", asin);

        let response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Audnexus API returned status: {}", response.status()));
        }

        let book: AudnexusBookResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        Ok(book)
    }

    /// Validate ASIN format (based on Audiobookshelf logic)
    fn is_valid_asin(asin: &str) -> bool {
        let asin = asin.to_uppercase();
        asin.len() == 10 && asin.chars().all(|c| c.is_alphanumeric())
    }

    /// Search using Audnexus API with author-based approach
    async fn search_audnexus(client: &Client, query: &str) -> Result<BookMetadata, String> {
        // First, try to treat the query as an ASIN
        if Self::is_valid_asin(query) {
            match Self::get_audnexus_book(client, query, "us").await {
                Ok(book) => return Self::convert_book_response_to_metadata(book),
                Err(e) => {
                    println!("ASIN lookup failed: {}", e);
                    // Fall through to author search
                }
            }
        }

        // Search for authors by name
        let authors = Self::search_audnexus_authors(client, query, "us").await?;

        if authors.is_empty() {
            return Err(format!("No authors found matching '{}'. Try searching with a book ASIN (10-character code like B0123456789) or use manual metadata entry.", query));
        }

        // Find the best matching author (exact match preferred, then first result)
        let author = authors.iter()
            .find(|a| a.name.to_lowercase() == query.to_lowercase())
            .or_else(|| authors.first())
            .ok_or("No authors found")?;

        println!("Found author: {} (ASIN: {})", author.name, author.asin);

        // Since Audnexus doesn't support searching books by author, we need to inform the user
        // In a production app, you'd integrate with another API like Google Books or Audible
        Err(format!("Author '{}' found, but Audnexus API doesn't support searching books by author. Try searching with a specific book ASIN (like B0123456789) or enter metadata manually.", author.name))
    }

    /// Convert AudnexusBookResponse to our BookMetadata format
    fn convert_book_response_to_metadata(book: AudnexusBookResponse) -> Result<BookMetadata, String> {
        // Extract author names from the authors array
        let authors: Vec<String> = book.authors
            .iter()
            .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
            .map(|s| s.to_string())
            .collect();

        let narrators: Option<Vec<String>> = book.narrators
            .as_ref()
            .map(|n| n.iter()
                .filter_map(|narr| narr.get("name").and_then(|name| name.as_str()))
                .map(|s| s.to_string())
                .collect()
            );

        Ok(BookMetadata {
            title: book.title,
            authors,
            narrator_names: narrators,
            series_name: book.series,
            image_url: book.image.unwrap_or_default(),
            asin: book.asin,
            duration_minutes: book.runtime_length_min,
            release_date: book.release_date,
        })
    }

    /// Search using Audible's unofficial search API (fallback)
    async fn search_audible(_client: &Client, query: &str) -> Result<BookMetadata, String> {
        // Audible API scraping would require significant additional implementation
        // For now, this is a placeholder for future enhancement
        Err(format!("Audible search not implemented. Audnexus API found authors but couldn't find books. Try searching with a specific ASIN or book title. Query: '{}'", query))
    }

    /// Convert directory of MP3s to M4B with chapters
    pub async fn convert_to_m4b_with_chapters(
        input_dir: &Path,
        output_path: &str,
        metadata: &BookMetadata,
    ) -> Result<(), String> {
        // Find all MP3 files in the directory
        let mp3_files = Self::find_mp3_files(input_dir).await?;
        if mp3_files.is_empty() {
            return Err("No MP3 files found in directory".to_string());
        }

        // Sort files by name for proper ordering
        let mut mp3_files = mp3_files;
        mp3_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        // Create a temporary concat file for ffmpeg
        let concat_file = format!("{}.txt", output_path);
        Self::create_concat_file(&mp3_files, &concat_file).await?;

        // Run ffmpeg conversion with metadata
        let mut ffmpeg_args: Vec<String> = vec![
            "-f".to_string(),
            "concat".to_string(),
            "-safe".to_string(),
            "0".to_string(),
            "-i".to_string(),
            concat_file.clone(),
            "-c:a".to_string(),
            "aac".to_string(),
            "-b:a".to_string(),
            "128k".to_string(),
            "-movflags".to_string(),
            "+faststart".to_string(),
        ];

        // Add metadata - push owned strings to avoid lifetime issues
        ffmpeg_args.push("-metadata".to_string());
        ffmpeg_args.push(format!("title={}", metadata.title));
        ffmpeg_args.push("-metadata".to_string());
        ffmpeg_args.push(format!("artist={}", metadata.authors.join(", ")));

        if let Some(series) = &metadata.series_name {
            ffmpeg_args.push("-metadata".to_string());
            ffmpeg_args.push(format!("album={}", series));
        }

        if let Some(narrators) = &metadata.narrator_names {
            ffmpeg_args.push("-metadata".to_string());
            ffmpeg_args.push(format!("composer={}", narrators.join(", ")));
        }

        if let Some(duration) = metadata.duration_minutes {
            ffmpeg_args.push("-metadata".to_string());
            ffmpeg_args.push(format!("duration={}", duration));
        }

        ffmpeg_args.push(output_path.to_string());

        let status = Command::new("ffmpeg")
            .args(&ffmpeg_args)
            .status()
            .await
            .map_err(|e| format!("FFmpeg failed: {}", e))?;

        // Clean up concat file
        let _ = tokio::fs::remove_file(&concat_file).await;

        if status.success() {
            Ok(())
        } else {
            Err("FFmpeg conversion failed".to_string())
        }
    }

    /// Find all MP3 files in a directory
    pub async fn find_mp3_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut mp3_files = Vec::new();

        let mut entries = tokio::fs::read_dir(dir)
            .await
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        while let Some(entry) = entries.next_entry().await.map_err(|e| format!("Failed to read entry: {}", e))? {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("mp3") {
                mp3_files.push(path);
            }
        }

        Ok(mp3_files)
    }

    /// Create ffmpeg concat file
    async fn create_concat_file(files: &[PathBuf], concat_path: &str) -> Result<(), String> {
        let mut content = String::new();

        for file in files {
            // Use absolute paths for FFmpeg concat demuxer to avoid issues
            let abs_path = file.canonicalize()
                .map_err(|e| format!("Failed to get absolute path for {}: {}", file.display(), e))?;
            let path_str = abs_path.to_string_lossy();
            content.push_str(&format!("file '{}'\n", path_str));
        }

        tokio::fs::write(concat_path, content)
            .await
            .map_err(|e| format!("Failed to write concat file: {}", e))?;

        Ok(())
    }

    /// Apply metadata tags to M4B file
    pub async fn apply_tags(_file_path: &str, metadata: &BookMetadata) -> Result<(), String> {
        // For now, use FFmpeg to add metadata during the conversion process
        // This is more reliable than trying to modify tags after creation
        println!("Metadata will be applied during FFmpeg conversion:");
        println!("  Title: {}", metadata.title);
        println!("  Author: {}", metadata.authors.join(", "));
        if let Some(series) = &metadata.series_name {
            println!("  Series: {}", series);
        }
        if let Some(narrators) = &metadata.narrator_names {
            println!("  Narrators: {}", narrators.join(", "));
        }

        // TODO: Implement proper metadata tagging with audiotags after conversion
        // The FFmpeg command in convert_to_m4b_with_chapters already includes basic metadata

        Ok(())
    }

    /// Upload to Audiobookshelf and trigger library scan
    pub async fn upload_and_scan(file_path: &str, config: &ABSConfig) -> Result<(), String> {
        let client = Client::new();

        // Read the file
        let file_data = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let file_name = Path::new(file_path)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or("Invalid file path")?;

        // Create multipart form data
        let form = reqwest::multipart::Form::new()
            .part("audioFile", reqwest::multipart::Part::bytes(file_data)
                .file_name(file_name.to_string())
                .mime_str("audio/m4b")
                .map_err(|e| format!("Failed to set MIME type: {}", e))?);

        // Upload to Audiobookshelf
        let upload_url = format!("{}/api/libraries/{}/upload", config.host.trim_end_matches('/'), config.library_id);

        let response = client
            .post(&upload_url)
            .header("Authorization", format!("Bearer {}", config.token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Upload request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Upload failed with status {}", response.status()));
        }

        // Parse response to get the uploaded item ID
        #[derive(Deserialize)]
        struct UploadResponse {
            _id: Option<String>, // Currently unused but kept for future use
        }

        let upload_result: UploadResponse = response
            .json()
            .await
            .unwrap_or(UploadResponse { _id: None });

        if let Some(item_id) = upload_result._id {
            // Trigger library scan
            Self::trigger_library_scan(&client, config, &item_id).await?;
        }

        Ok(())
    }

    /// Trigger a library scan for the uploaded item
    async fn trigger_library_scan(client: &Client, config: &ABSConfig, _item_id: &str) -> Result<(), String> {
        let scan_url = format!("{}/api/libraries/{}/scan", config.host.trim_end_matches('/'), config.library_id);

        let scan_data = serde_json::json!({
            "force": true
        });

        let response = client
            .post(&scan_url)
            .header("Authorization", format!("Bearer {}", config.token))
            .header("Content-Type", "application/json")
            .json(&scan_data)
            .send()
            .await
            .map_err(|e| format!("Scan request failed: {}", e))?;

        if !response.status().is_success() {
            println!("Warning: Library scan failed with status {}", response.status());
            // Don't return error - upload was successful, scan is optional
        }

        Ok(())
    }

    /// Search using Google Books API
    async fn search_google_books(client: &Client, query: &str) -> Result<BookMetadata, String> {
        // Rate limiting: small delay for Google Books API
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let url = format!(
            "https://www.googleapis.com/books/v1/volumes?q={}&printType=books&maxResults=5",
            urlencoding::encode(&format!("{} audiobook", query))
        );

        println!("Google Books: Searching for '{}'", query);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Google Books API returned status: {}", response.status()));
        }

        let search_result: GoogleBooksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        if let Some(items) = search_result.items {
            for item in items {
                if item.volume_info.is_some() {
                    return Self::convert_google_books_item_to_metadata(item);
                }
            }
        }

        Err("No books found matching the query".to_string())
    }

    fn convert_google_books_item_to_metadata(item: GoogleBooksItem) -> Result<BookMetadata, String> {
        let volume_info = item.volume_info.ok_or("No volume info available")?;


        let authors = volume_info.authors.unwrap_or_default();
        let title = volume_info.title.unwrap_or_default();
        let subtitle = volume_info.subtitle.unwrap_or_default();
        let full_title = if subtitle.is_empty() {
            title
        } else {
            format!("{}: {}", title, subtitle)
        };

        // Get the best available cover image
        let cover_url = if let Some(image_links) = volume_info.image_links {
            image_links.extra_large
                .or(image_links.large)
                .or(image_links.medium)
                .or(image_links.small)
                .or(image_links.thumbnail)
                .or(image_links.small_thumbnail)
        } else {
            None
        };

        // Extract ISBN if available
        let asin = volume_info.industry_identifiers
            .and_then(|identifiers| {
                identifiers.into_iter()
                    .find(|id| id.identifier_type == "ISBN_13" || id.identifier_type == "ISBN_10")
                    .map(|id| id.identifier)
            })
            .unwrap_or_default();

        Ok(BookMetadata {
            title: full_title,
            authors,
            narrator_names: None, // Google Books doesn't typically have narrator info
            series_name: None, // Google Books API doesn't have series info
            image_url: cover_url.unwrap_or_default(),
            asin,
            duration_minutes: None, // Google Books doesn't have duration
            release_date: volume_info.published_date,
        })
    }

    /// Search using Open Library API
    async fn search_open_library(client: &Client, query: &str) -> Result<BookMetadata, String> {
        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let url = format!(
            "https://openlibrary.org/search.json?q={}&limit=5",
            urlencoding::encode(query)
        );

        println!("Open Library: Searching for '{}'", query);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Open Library API returned status: {}", response.status()));
        }

        let search_result: OpenLibrarySearchResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        if let Some(first_doc) = search_result.docs.into_iter().next() {
            return Self::convert_open_library_doc_to_metadata(first_doc);
        }

        Err("No books found matching the query".to_string())
    }

    fn convert_open_library_doc_to_metadata(doc: OpenLibraryDoc) -> Result<BookMetadata, String> {
        let authors = doc.author_name.unwrap_or_default();
        let title = doc.title.unwrap_or_default();

        // Get cover URL if available
        let cover_url = doc.cover_i
            .map(|cover_id| format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_id))
            .unwrap_or_default();

        // Extract ISBN if available
        let asin = doc.isbn
            .and_then(|isbns| isbns.into_iter().next())
            .unwrap_or_default();

        Ok(BookMetadata {
            title,
            authors,
            narrator_names: None,
            series_name: None,
            image_url: cover_url,
            asin,
            duration_minutes: None,
            release_date: doc.first_publish_year.map(|year| year.to_string()),
        })
    }

    /// Search using iTunes API (placeholder - iTunes API is complex and may require authentication)
    async fn search_itunes(_client: &Client, query: &str) -> Result<BookMetadata, String> {
        Err(format!("iTunes search not implemented yet. Query: '{}'", query))
    }

    /// Search using FantLab API (placeholder - Russian sci-fi/fantasy focused)
    async fn search_fantlab(_client: &Client, query: &str) -> Result<BookMetadata, String> {
        Err(format!("FantLab search not implemented yet. Query: '{}'", query))
    }

    /// Generate a file path using template substitution for Local Library
    pub fn generate_local_library_path(
        base_path: &str,
        template: &str,
        metadata: &BookMetadata,
    ) -> Result<String, String> {
        let mut path = template.to_string();

        // Replace placeholders with actual values
        path = path.replace("{Author}", &Self::sanitize_filename(&metadata.authors.join(", ")));
        path = path.replace("{Series}", &Self::sanitize_filename(&metadata.series_name.as_deref().unwrap_or("")));
        path = path.replace("{Title}", &Self::sanitize_filename(&metadata.title));
        path = path.replace("{Year}", &metadata.release_date.as_deref().unwrap_or("").to_string());

        // For series number, we need to parse it from the series name if possible
        // For now, default to empty string - could be enhanced later
        path = path.replace("{SeriesNumber}", "");

        // Disk and chapter numbers are not part of book metadata, default to empty
        path = path.replace("{DiskNumber}", "");
        path = path.replace("{DiskNumber:00}", "");
        path = path.replace("{ChapterNumber}", "");
        path = path.replace("{ChapterNumber:00}", "");

        // Quality is not determined yet, default to empty
        path = path.replace("{Quality}", "");

        // Replace the base path placeholder
        path = path.replace("{Path to Local Library}", base_path.trim_end_matches('/'));

        // Ensure the path ends with .m4b
        if !path.to_lowercase().ends_with(".m4b") {
            path.push_str(".m4b");
        }

        Ok(path)
    }

    /// Sanitize filenames by removing/replacing invalid characters
    fn sanitize_filename(name: &str) -> String {
        name.chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                c => c,
            })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Get chapters from Audnexus API
    pub async fn get_chapters_from_audnexus(client: &Client, asin: &str, region: &str) -> Result<Vec<Chapter>, String> {
        // Rate limiting
        tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;

        let mut url = format!("https://api.audnex.us/books/{}/chapters", urlencoding::encode(asin));
        if !region.is_empty() && region != "us" {
            url.push_str(&format!("&region={}", region));
        }

        println!("Audnexus: Getting chapters for ASIN '{}'", asin);

        let response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(vec![]); // No chapters available, return empty list
        }

        let chapters_response: AudnexusChaptersResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        let mut chapters = Vec::new();

        for chapter in chapters_response.chapters {
            let start_time = chapter.start_offset_sec as f64;
            let duration = chapter.length_ms as f64 / 1000.0;
            let end_time = start_time + duration;

            chapters.push(Chapter {
                title: chapter.title,
                start_time,
                end_time: Some(end_time),
                locked: false,
            });
        }

        Ok(chapters)
    }

    /// Auto-detect chapters from MP3 filenames in a directory
    pub async fn auto_detect_chapters(dir_path: &str) -> Result<Vec<Chapter>, String> {
        use std::fs;
        use regex::Regex;

        let mut chapters = Vec::new();
        let mut mp3_files = Vec::new();

        // Read directory and collect MP3 files
        let entries = fs::read_dir(dir_path)
            .map_err(|e| format!("Failed to read directory: {}", e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if let Some(extension) = path.extension() {
                if extension.to_string_lossy().to_lowercase() == "mp3" {
                    mp3_files.push(path);
                }
            }
        }

        // Sort files by name for proper ordering
        mp3_files.sort_by(|a, b| a.file_name().cmp(&b.file_name()));

        // Try to extract chapter info from filenames
        let chapter_regex = Regex::new(r"(?i)chapter\s*(\d+)|(\d+)\s*-\s*|\b(\d{1,3})\b").unwrap();

        for (index, file_path) in mp3_files.iter().enumerate() {
            let filename = file_path.file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("Unknown");

            // Try to extract chapter number and title
            let (chapter_num, title) = if let Some(captures) = chapter_regex.captures(filename) {
                let num = captures.get(1).or(captures.get(2)).or(captures.get(3))
                    .and_then(|m| m.as_str().parse::<u32>().ok())
                    .unwrap_or((index + 1) as u32);

                // Clean up the title by removing the chapter number part
                let title = chapter_regex.replace(filename, "").trim().to_string();
                let title = if title.is_empty() {
                    format!("Chapter {}", num)
                } else {
                    title.to_string()
                };

                (num, title)
            } else {
                ((index + 1) as u32, filename.to_string())
            };

            chapters.push(Chapter {
                title: format!("Chapter {}: {}", chapter_num, title),
                start_time: 0.0, // Will be calculated based on file durations
                end_time: None,
                locked: false,
            });
        }

        Ok(chapters)
    }

    /// Calculate chapter timestamps based on MP3 file durations
    pub async fn calculate_chapter_timestamps(_dir_path: &str, chapters: &mut Vec<Chapter>) -> Result<(), String> {
        let mut current_time = 0.0;

        for chapter in chapters.iter_mut() {
            chapter.start_time = current_time;

            // For now, we'll use a simple estimation. In a real implementation,
            // you'd get the actual duration of each MP3 file using ffprobe
            // For demonstration, we'll assume each chapter is about 20 minutes
            let estimated_duration = 20.0 * 60.0; // 20 minutes in seconds
            chapter.end_time = Some(current_time + estimated_duration);
            current_time += estimated_duration;
        }

        Ok(())
    }

    /// Search for cover art images
    pub async fn search_cover_art(query: &str, provider: SearchProvider) -> Result<Vec<String>, String> {
        let client = Client::new();

        match provider {
            SearchProvider::Audnexus => Self::search_audnexus_cover(&client, query).await,
            SearchProvider::GoogleBooks => Self::search_google_books_cover(&client, query).await,
            SearchProvider::OpenLibrary => Self::search_open_library_cover(&client, query).await,
            _ => Err(format!("Cover search not implemented for {:?}", provider)),
        }
    }

    /// Search for cover art using Audnexus
    async fn search_audnexus_cover(client: &Client, query: &str) -> Result<Vec<String>, String> {
        // For Audnexus, we can try to get the book first and extract the cover URL
        match Self::search_audnexus(client, query).await {
            Ok(metadata) => {
                if !metadata.image_url.is_empty() {
                    Ok(vec![metadata.image_url])
                } else {
                    Ok(vec![])
                }
            }
            Err(_) => Ok(vec![]),
        }
    }

    /// Search for cover art using Google Books
    async fn search_google_books_cover(client: &Client, query: &str) -> Result<Vec<String>, String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let url = format!(
            "https://www.googleapis.com/books/v1/volumes?q={}&printType=books&maxResults=10",
            urlencoding::encode(&format!("{} audiobook", query))
        );

        println!("Google Books: Searching covers for '{}'", query);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let search_result: GoogleBooksResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        let mut covers = Vec::new();

        if let Some(items) = search_result.items {
            for item in items {
                if let Some(volume_info) = item.volume_info {
                    if let Some(image_links) = volume_info.image_links {
                        // Try different image sizes in order of preference
                        if let Some(url) = image_links.extra_large.or(image_links.large) {
                            covers.push(url);
                        } else if let Some(url) = image_links.medium.or(image_links.small) {
                            covers.push(url);
                        } else if let Some(url) = image_links.thumbnail.or(image_links.small_thumbnail) {
                            covers.push(url);
                        }
                    }
                }
            }
        }

        Ok(covers)
    }

    /// Search for cover art using Open Library
    async fn search_open_library_cover(client: &Client, query: &str) -> Result<Vec<String>, String> {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let url = format!(
            "https://openlibrary.org/search.json?q={}&limit=10",
            urlencoding::encode(query)
        );

        println!("Open Library: Searching covers for '{}'", query);

        let response = client
            .get(&url)
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        let search_result: OpenLibrarySearchResult = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        let mut covers = Vec::new();

        for doc in search_result.docs {
            if let Some(cover_id) = doc.cover_i {
                // Open Library cover URLs
                covers.push(format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_id));
            }
        }

        Ok(covers)
    }

}
