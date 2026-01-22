use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use serde::{Deserialize, Serialize};
use reqwest::Client;

// --- Data Structures ---

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub narrator_names: Option<Vec<String>>,
    pub series_name: Option<String>,
    pub cover_url: Option<String>,
    pub asin: Option<String>,
    pub duration_minutes: Option<u64>,
    pub release_date: Option<String>,
}

#[derive(Clone, Debug)]
pub struct Chapter {
    pub title: String,
    pub start_time: f64,
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
pub struct AudnexusSearchResult {
    pub result: Vec<AudnexusSearchResponse>,
}

#[derive(Deserialize, Debug)]
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

// --- Audio Service Implementation ---

pub struct AudioService;

impl AudioService {
    /// Fetch metadata from Audnexus API (free alternative to Audible API)
    pub async fn fetch_metadata(query: &str) -> Result<BookMetadata, String> {
        let client = Client::new();

        // Try Audnexus API first (handles both ASIN and title searches)
        match Self::search_audnexus(&client, query).await {
            Ok(metadata) => return Ok(metadata),
            Err(e) => println!("Audnexus search failed: {}, trying Audible API", e),
        }

        // Fallback to Audible search if available
        Self::search_audible(&client, query).await
    }

    /// Search using Audnexus API
    async fn search_audnexus(client: &Client, query: &str) -> Result<BookMetadata, String> {
        let url = format!("https://api.audnex.us/books/{}", urlencoding::encode(query));

        let response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Network request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Audnexus API returned status: {}", response.status()));
        }

        let data: AudnexusSearchResponse = response
            .json()
            .await
            .map_err(|e| format!("Failed to parse JSON response: {}", e))?;

        // Convert to our BookMetadata format
        Ok(BookMetadata {
            title: data.title,
            authors: data.authors,
            narrator_names: if data.narrators.is_empty() { None } else { Some(data.narrators) },
            series_name: data.series,
            cover_url: data.image,
            asin: Some(data.asin),
            duration_minutes: data.runtime_length_min,
            release_date: data.release_date,
        })
    }

    /// Search using Audible's unofficial search API (fallback)
    async fn search_audible(client: &Client, query: &str) -> Result<BookMetadata, String> {
        // This is a simplified fallback - in practice you'd use a more robust search
        let search_url = format!(
            "https://www.audible.com/search?keywords={}",
            urlencoding::encode(query)
        );

        // For now, return a mock result since Audible's API is restricted
        // In a real implementation, you'd scrape or use a proxy service
        Err("Audible search not implemented - use Audnexus for now".to_string())
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

        // Run ffmpeg conversion
        let status = Command::new("ffmpeg")
            .args(&[
                "-f", "concat",
                "-safe", "0",
                "-i", &concat_file,
                "-c:a", "aac",
                "-b:a", "128k",
                "-movflags", "+faststart",
                "-metadata", &format!("title={}", metadata.title),
                "-metadata", &format!("artist={}", metadata.authors.join(", ")),
                &output_path
            ])
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
    async fn find_mp3_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
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
            let path_str = file.to_string_lossy();
            content.push_str(&format!("file '{}'\n", path_str));
        }

        tokio::fs::write(concat_path, content)
            .await
            .map_err(|e| format!("Failed to write concat file: {}", e))?;

        Ok(())
    }

    /// Apply metadata tags to M4B file
    pub async fn apply_tags(file_path: &str, metadata: &BookMetadata) -> Result<(), String> {
        // TODO: Implement metadata tagging with audiotags
        // For now, just log that we'd apply tags
        println!("TODO: Would apply metadata tags to {}", file_path);
        println!("  Title: {}", metadata.title);
        println!("  Author: {:?}", metadata.authors);
        if let Some(series) = &metadata.series_name {
            println!("  Series: {}", series);
        }
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
            id: Option<String>,
        }

        let upload_result: UploadResponse = response
            .json()
            .await
            .unwrap_or(UploadResponse { id: None });

        if let Some(item_id) = upload_result.id {
            // Trigger library scan
            Self::trigger_library_scan(&client, &config, &item_id).await?;
        }

        Ok(())
    }

    /// Trigger a library scan for the uploaded item
    async fn trigger_library_scan(client: &Client, config: &ABSConfig, item_id: &str) -> Result<(), String> {
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
}
