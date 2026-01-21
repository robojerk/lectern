use std::path::{Path, PathBuf};
use std::process::Stdio;
use tokio::process::Command;
use tokio::io::{AsyncBufReadExt, BufReader};
use serde::Deserialize;
use crate::models::settings::AppConfig;

// --- Data Structures ---

#[derive(Deserialize, Debug)]
struct AudibleAuthor {
    #[serde(default)]
    name: String,
}

#[derive(Deserialize, Debug)]
struct AudibleSeries {
    #[serde(default)]
    title: String,
    #[serde(default)]
    sequence: Option<String>,
}

#[derive(Deserialize, Debug)]
struct AudibleProduct {
    #[serde(default, alias = "product_asin")]
    asin: String,
    #[serde(default, alias = "name")]
    title: String,
    #[serde(default)]
    subtitle: Option<String>,
    #[serde(default)]
    authors: Option<Vec<AudibleAuthor>>,
    #[serde(default)]
    narrators: Option<Vec<AudibleAuthor>>,
    #[serde(default)]
    series: Option<Vec<AudibleSeries>>,
    #[serde(default)]
    publication_name: Option<String>,
    #[serde(default)]
    product_images: Option<std::collections::HashMap<String, String>>,
    #[serde(default)]
    runtime_length_min: Option<u64>,
    #[serde(default)]
    publication_datetime: Option<String>,
    #[serde(default)]
    release_date: Option<String>,
    #[serde(default)]
    publisher_name: Option<String>,
    #[serde(default)]
    language: Option<String>,
    #[serde(default)]
    merchandising_summary: Option<String>,
    #[serde(default)]
    thesaurus_subject_keywords: Option<Vec<String>>,
    #[serde(default)]
    format_type: Option<String>,
}

impl From<AudibleProduct> for BookMetadata {
    fn from(p: AudibleProduct) -> Self {
        let authors = p.authors.unwrap_or_default()
            .into_iter().map(|a| a.name).collect();
        let narrators = p.narrators.map(|ns| ns.into_iter().map(|n| n.name).collect());

        // Series Detection Logic
        let (series_name, mut series_number) = if let Some(s) = p.series.and_then(|s| s.into_iter().next()) {
            (Some(s.title), s.sequence)
        } else {
            (None, None)
        };
        
        let mut series_name = series_name;

        // Fallback 1: Publication Name
        if series_name.is_none() {
            series_name = p.publication_name.clone();
        }
        
        // Fallback 2: Check Subtitle for "Book X" patterns
        if series_name.is_none() || series_number.is_none() {
             if let Some(sub) = &p.subtitle {
                 if sub.to_lowercase().contains("book") {
                     if series_name.is_none() {
                         series_name = Some(sub.split(',').next().unwrap_or(sub).trim().to_string());
                     }
                     if series_number.is_none() {
                         // Try to extract number after "Book"
                         if let Some(pos) = sub.to_lowercase().find("book") {
                             let after_book = &sub[pos + 4..].trim();
                             series_number = after_book.split(|c: char| !c.is_numeric() && c != '.').next().map(|s| s.to_string());
                         }
                     }
                 }
             }
        }
        
        // Date Detection: Prioritize release_date, fallback to publication_datetime
        let raw_date = p.release_date.or(p.publication_datetime);
        let cleaned_date = raw_date.map(|d| d.split('T').next().unwrap_or(&d).to_string());

        // Strip HTML tags from description
        let description = p.merchandising_summary.map(|s| {
            // Very basic HTML tag removal
            let mut result = String::new();
            let mut inside_tag = false;
            for c in s.chars() {
                if c == '<' { inside_tag = true; }
                else if c == '>' { inside_tag = false; }
                else if !inside_tag { result.push(c); }
            }
            // Replace common entities
            result.replace("&nbsp;", " ").replace("&quot;", "\"").replace("&amp;", "&").trim().to_string()
        });

        let cover_url = p.product_images.and_then(|imgs| {
            imgs.get("1215")
                .or_else(|| imgs.get("500"))
                .or_else(|| imgs.get("400"))
                .or_else(|| imgs.values().next())
                .cloned()
        });

        BookMetadata {
            title: p.title,
            subtitle: p.subtitle,
            authors,
            narrator_names: narrators,
            series_name,
            series_number,
            cover_url,
            asin: Some(p.asin),
            duration_minutes: p.runtime_length_min,
            release_date: cleaned_date,
            description,
            genres: p.thesaurus_subject_keywords,
            publisher: p.publisher_name,
            language: p.language,
            quality: Some("128k".to_string()), // Default quality
            is_abridged: p.format_type.map(|t| t.to_lowercase() == "abridged").unwrap_or(false),
            ..Default::default()
        }
    }
}

#[derive(Deserialize, Debug)]
struct AudibleResult {
    products: Vec<AudibleProduct>,
}

#[derive(Deserialize, Debug)]
struct AudibleChapter {
    #[serde(default)]
    title: String,
    #[serde(default)]
    start_offset_ms: u64,
    #[serde(default)]
    length_ms: u64,
}

#[derive(Deserialize, Debug)]
struct AudibleContent {
    #[serde(default)]
    chapters: Option<Vec<AudibleChapter>>,
}

#[derive(Deserialize, Debug)]
struct AudibleChapterResponse {
    #[serde(default)]
    content: Option<AudibleContent>,
}

#[derive(Deserialize, Debug)]
struct AudnexusChapter {
    #[serde(default)]
    title: String,
    #[serde(rename = "startOffsetMs")]
    start_ms: u64,
    #[serde(rename = "lengthMs")]
    length_ms: u64,
}

#[derive(Deserialize, Debug)]
struct AudnexusResponse {
    #[serde(default)]
    chapters: Vec<AudnexusChapter>,
}

use crate::models::metadata::BookMetadata;

#[derive(Deserialize)]
struct ProbeOutput {
    format: ProbeFormat,
}

#[derive(Deserialize)]
struct ProbeFormat {
    duration: String,
}

#[derive(Debug, Clone)]
pub struct SimpleChapter {
    pub title: String,
    pub start_ms: u64,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, Default)]
pub struct AudioMap {
    pub files: Vec<PathBuf>,
    pub durations: Vec<u64>,
}

impl AudioMap {
    /// Translates a global audiobook timestamp into (file_path, local_offset)
    pub fn resolve_timestamp(&self, global_ms: u64) -> Option<(PathBuf, u64)> {
        let mut accumulated = 0;
        for (i, file) in self.files.iter().enumerate() {
            let dur = self.durations.get(i).cloned().unwrap_or(0);
            if global_ms < accumulated + dur {
                return Some((file.clone(), global_ms - accumulated));
            }
            accumulated += dur;
        }
        // If it's the exact end or slightly over, just return the last file at its end
        if !self.files.is_empty() && global_ms >= accumulated {
            let last_idx = self.files.len() - 1;
            return Some((self.files[last_idx].clone(), self.durations[last_idx]));
        }
        None
    }
}



#[derive(Deserialize)]
struct ProbeChapter {
    #[serde(default)]
    title: String,
    start_time_ms: f64,
    end_time_ms: f64,
}

#[derive(Deserialize)]
struct ProbeResult {
    chapters: Option<Vec<ProbeChapter>>,
}

// --- Audio Service Implementation ---

pub struct AudioService;

#[allow(dead_code)]
impl AudioService {
    /// Search for metadata - returns multiple results
    pub async fn search_metadata(query: &str, by_asin: bool) -> Result<Vec<BookMetadata>, String> {
        let url = if by_asin {
            format!(
                "https://api.audible.com/1.0/catalog/products/{}?response_groups=product_desc,product_attrs,media",
                urlencoding::encode(query)
            )
        } else {
            format!(
                "https://api.audible.com/1.0/catalog/products?title={}&response_groups=product_desc,product_attrs,media",
                urlencoding::encode(query)
            )
        };
        
        let client = reqwest::Client::new();
        
        let response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        let body = response.text().await.map_err(|e| format!("Failed to read body: {}", e))?;

        if by_asin {
            #[derive(Deserialize)]
            struct SingleProductResponse { product: AudibleProduct }
            let res: SingleProductResponse = serde_json::from_str(&body)
                .map_err(|e| format!("JSON parse error (asin): {}", e))?;
            Ok(vec![BookMetadata::from(res.product)])
        } else {
            let result: AudibleResult = serde_json::from_str(&body)
                .map_err(|e| format!("JSON parse error (products): {}", e))?;
            
            if result.products.is_empty() {
                Err("No results found".to_string())
            } else {
                Ok(result.products.into_iter().map(BookMetadata::from).collect())
            }
        }
    }
    
    pub async fn fetch_chapters_from_audible(asin: &str, marketplace: &str) -> Result<Vec<SimpleChapter>, String> {
        let tld = match marketplace {
            "ca" => "ca",
            "uk" => "co.uk",
            "au" => "com.au",
            "de" => "de",
            "fr" => "fr",
            "it" => "it",
            "es" => "es",
            "jp" => "co.jp",
            "in" => "in",
            _ => "com",
        };

        let url = format!(
            "https://api.audible.{}/1.0/catalog/products/{}/content?response_groups=chapters",
            tld, asin
        );

        let client = reqwest::Client::new();
        let response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Network error: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("API returned status: {}", response.status()));
        }

        let res: AudibleChapterResponse = response.json().await
            .map_err(|e| format!("JSON parse error: {}", e))?;

        let chapters = res.content
            .and_then(|c| c.chapters)
            .ok_or_else(|| "No chapters found in Audible response".to_string())?;

        Ok(chapters.into_iter().map(|c| SimpleChapter {
            title: c.title,
            start_ms: c.start_offset_ms,
            duration_ms: c.length_ms,
        }).collect())
    }

    pub async fn fetch_chapters_from_audnexus(asin: &str, locale: &str) -> Result<Vec<SimpleChapter>, String> {
        let asin_upper = asin.to_uppercase();
        let url = format!("https://api.audnex.us/books/{}/chapters?region={}", asin_upper, locale);
        println!("🔄 Fetching from Audnexus: {}", url);
        
        let client = reqwest::Client::new();
        let mut response = client
            .get(&url)
            .header("User-Agent", "Lectern/1.0")
            .send()
            .await
            .map_err(|e| format!("Audnexus network error: {}", e))?;

        if response.status() == reqwest::StatusCode::TOO_MANY_REQUESTS {
            // Basic retry once after 2 seconds if rate limited
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
            response = client
                .get(&url)
                .header("User-Agent", "Lectern/1.0")
                .send()
                .await
                .map_err(|e| format!("Audnexus retry error: {}", e))?;
        }

        if !response.status().is_success() {
            return Err(format!("Audnexus returned status: {}", response.status()));
        }

        let res: AudnexusResponse = response.json().await
            .map_err(|e| format!("Audnexus JSON parse error: {}", e))?;

        if res.chapters.is_empty() {
             return Err("No chapters found in Audnexus response".to_string());
        }

        Ok(res.chapters.into_iter().map(|c| SimpleChapter {
            title: c.title,
            start_ms: c.start_ms,
            duration_ms: c.length_ms,
        }).collect())
    }

    /// Fetch metadata from Audible API (returns first match for backward compatibility)
    pub async fn fetch_metadata(query: &str) -> Result<BookMetadata, String> {
        let results = Self::search_metadata(query, false).await?;
        results.into_iter().next().ok_or_else(|| "No results found".to_string())
    }

    /// Convert directory of MP3s to M4B with chapters
    pub async fn convert_to_m4b_with_chapters(
        input_dir: &Path,
        output_path: &str,
        tx: glib::Sender<crate::app_event::AppEvent>,
    ) -> Result<(), String> {
        // Step 1: Get all MP3 files sorted
        let _ = tx.send(crate::app_event::AppEvent::Log("📁 Scanning directory for MP3 files...".to_string()));
        
        let mp3_files = Self::get_sorted_mp3_files(input_dir)?;
        
        let _ = tx.send(crate::app_event::AppEvent::Log(
            format!("✓ Found {} MP3 files", mp3_files.len())
        ));

        if mp3_files.is_empty() {
            return Err("No MP3 files found in directory".to_string());
        }

        // Step 2: Generate chapter metadata
        let _ = tx.send(crate::app_event::AppEvent::Log("⏱️  Analyzing file durations...".to_string()));
        let metadata_content = Self::build_chapters(&mp3_files, tx.clone()).await?;
        
        let metadata_file = "/tmp/ffmetadata.txt";
        tokio::fs::write(metadata_file, &metadata_content)
            .await
            .map_err(|e| format!("Failed to write metadata file: {}", e))?;

        // Step 3: Create concat file list
        let concat_file = "/tmp/concat_list.txt";
        let concat_content = mp3_files
            .iter()
            .map(|p| format!("file '{}'", p.display()))
            .collect::<Vec<_>>()
            .join("\n");
        
        tokio::fs::write(concat_file, concat_content)
            .await
            .map_err(|e| format!("Failed to write concat file: {}", e))?;

        // Step 4: Run FFmpeg with real-time logging
        let _ = tx.send(crate::app_event::AppEvent::Log("🎬 Running FFmpeg conversion...".to_string()));
        
        Self::run_ffmpeg_with_logs(
            vec![
                "-f".to_string(),
                "concat".to_string(),
                "-safe".to_string(),
                "0".to_string(),
                "-i".to_string(),
                concat_file.to_string(),
                "-i".to_string(),
                metadata_file.to_string(),
                "-map_metadata".to_string(),
                "1".to_string(),
                "-c:a".to_string(),
                "aac".to_string(),
                "-b:a".to_string(),
                "128k".to_string(),
                "-f".to_string(),
                "mp4".to_string(),
                output_path.to_string(),
            ],
            tx.clone(),
        )
        .await?;

        // Cleanup temp files
        let _ = tokio::fs::remove_file(metadata_file).await;
        let _ = tokio::fs::remove_file(concat_file).await;

        Ok(())
    }

    /// Get sorted list of MP3 files from directory
    pub fn get_sorted_mp3_files(dir: &Path) -> Result<Vec<PathBuf>, String> {
        let mut entries: Vec<PathBuf> = std::fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory: {}", e))?
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| {
                p.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("mp3"))
                    .unwrap_or(false)
            })
            .collect();

        entries.sort_by(|a, b| {
            a.file_name()
                .cmp(&b.file_name())
        });

        Ok(entries)
    }

    /// Try to find a local cover image in the directory
    pub fn find_local_cover(dir: &Path) -> Option<PathBuf> {
        let common_names = ["cover", "folder", "front", "album", "book"];
        let extensions = ["jpg", "jpeg", "png", "webp"];

        for entry in std::fs::read_dir(dir).ok()?.filter_map(|e| e.ok()) {
            let path = entry.path();
            if path.is_file() {
                let stem = path.file_stem()?.to_str()?.to_lowercase();
                let ext = path.extension()?.to_str()?.to_lowercase();
                
                if common_names.iter().any(|&n| stem.contains(n)) && extensions.iter().any(|&e| ext == e) {
                    return Some(path);
                }
            }
        }
        None
    }

    /// Try to find a local metadata file (e.g. metadata.json)
    pub fn find_local_metadata(dir: &Path) -> Option<BookMetadata> {
        let meta_path = dir.join("metadata.json");
        if meta_path.exists() {
            if let Ok(content) = std::fs::read_to_string(meta_path) {
                if let Ok(meta) = serde_json::from_str::<BookMetadata>(&content) {
                    return Some(meta);
                }
            }
        }
        None
    }

    /// Get chapters from a single M4B file
    pub async fn get_chapters_from_m4b(file_path: &Path) -> Result<Vec<SimpleChapter>, String> {
        let output = Command::new("ffprobe")
            .args([
                "-v", "error",
                "-show_chapters",
                "-of", "json",
                file_path.to_str().unwrap(),
            ])
            .output()
            .await
            .map_err(|e| format!("ffprobe failed: {}", e))?;

        if !output.status.success() {
            return Err("ffprobe execution failed".to_string());
        }

        let probe: ProbeResult = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse ffprobe chapter output: {}", e))?;

        let chapters = probe.chapters.unwrap_or_default();
        Ok(chapters.into_iter().map(|c| SimpleChapter {
            title: c.title,
            start_ms: c.start_time_ms as u64,
            duration_ms: (c.end_time_ms - c.start_time_ms) as u64,
        }).collect())
    }

    /// Get chapters from files
    pub async fn get_chapters(files: &[PathBuf]) -> Result<Vec<SimpleChapter>, String> {
        let mut chapters = Vec::new();
        let mut current_offset_ms: u64 = 0;

        for file in files {
            // Note: Parallelizing this would be faster, but sequential is safer for now
            let duration_ms = Self::get_duration(file).await?;
            let title = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            chapters.push(SimpleChapter {
                title,
                start_ms: current_offset_ms,
                duration_ms,
            });

            current_offset_ms += duration_ms;
        }

        Ok(chapters)
    }

    /// Build chapter metadata from MP3 files
    async fn build_chapters(
        files: &[PathBuf],
        tx: glib::Sender<crate::app_event::AppEvent>,
    ) -> Result<String, String> {
        let mut metadata_file = String::from(";FFMETADATA1\n");
        let mut current_offset_ms: u64 = 0;

        for (idx, file) in files.iter().enumerate() {
            let duration_ms = Self::get_duration(file).await?;
            let title = file
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();
            
            let end_ms = current_offset_ms + duration_ms;

            metadata_file.push_str("[CHAPTER]\n");
            metadata_file.push_str("TIMEBASE=1/1000\n");
            metadata_file.push_str(&format!("START={}\n", current_offset_ms));
            metadata_file.push_str(&format!("END={}\n", end_ms));
            metadata_file.push_str(&format!("title={}\n", title));

            let _ = tx.send(crate::app_event::AppEvent::Log(
                format!("  Chapter {}: {} ({:.1}s)", idx + 1, title, duration_ms as f64 / 1000.0)
            ));

            current_offset_ms = end_ms;
        }

        let _ = tx.send(crate::app_event::AppEvent::Log(
            format!("✓ Generated {} chapters", files.len())
        ));

        Ok(metadata_file)
    }

    /// Get duration of audio file using ffprobe
    pub async fn get_duration(file_path: &Path) -> Result<u64, String> {
        let output = Command::new("ffprobe")
            .args([
                "-v",
                "error",
                "-show_entries",
                "format=duration",
                "-of",
                "json",
                file_path.to_str().unwrap(),
            ])
            .output()
            .await
            .map_err(|e| format!("ffprobe failed: {}", e))?;

        if !output.status.success() {
            return Err("ffprobe execution failed".to_string());
        }

        let probe: ProbeOutput = serde_json::from_slice(&output.stdout)
            .map_err(|e| format!("Failed to parse ffprobe output: {}", e))?;

        let seconds: f64 = probe
            .format
            .duration
            .parse()
            .map_err(|e| format!("Invalid duration format: {}", e))?;

        Ok((seconds * 1000.0) as u64)
    }

    /// Run FFmpeg with real-time logging
    pub async fn run_ffmpeg_with_logs(
        args: Vec<String>,
        tx: glib::Sender<crate::app_event::AppEvent>,
    ) -> Result<(), String> {
        let mut child = Command::new("ffmpeg")
            .args(&args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| format!("Failed to start ffmpeg: {}", e))?;

        // FFmpeg outputs progress to stderr
        if let Some(stderr) = child.stderr.take() {
            let reader = BufReader::new(stderr);
            let mut lines = reader.lines();

            tokio::spawn(async move {
                while let Ok(Some(line)) = lines.next_line().await {
                    // Only log interesting lines (skip verbose output)
                    if line.contains("time=") || line.contains("error") || line.contains("Error") {
                        let _ = tx.send(crate::app_event::AppEvent::Log(format!("  ffmpeg: {}", line.trim())));
                    }
                }
            });
        }

        let status = child
            .wait()
            .await
            .map_err(|e| format!("FFmpeg process error: {}", e))?;

        if !status.success() {
            return Err(format!("FFmpeg exited with code: {:?}", status.code()));
        }

        Ok(())
    }

    /// Apply metadata tags to M4B file
    pub async fn apply_tags(file_path: &str, metadata: &BookMetadata) -> Result<(), String> {
        // Note: audiotags crate doesn't work well with async, so we use blocking operations
        tokio::task::spawn_blocking({
            let file_path = file_path.to_string();
            let metadata = metadata.clone();
            
            move || -> Result<(), String> {
                use audiotags::Tag;
                
                let mut tag = Tag::new()
                    .read_from_path(&file_path)
                    .map_err(|e| format!("Failed to read file for tagging: {}", e))?;

                tag.set_title(&metadata.title);
                tag.set_album_title(&metadata.title);
                tag.set_artist(&metadata.authors.join(", "));
                tag.set_album_artist(&metadata.authors.join(", "));

                if let Some(series) = &metadata.series_name {
                    // Use album field for series (common convention)
                    tag.set_album_title(series);
                }

                if let Some(date) = &metadata.release_date {
                    if let Ok(year) = date.split('-').next().unwrap_or(date).parse::<i32>() {
                        tag.set_year(year);
                    }
                }

                tag.write_to_path(&file_path)
                    .map_err(|e| format!("Failed to write tags: {}", e))?;

                Ok(())
            }
        })
        .await
        .map_err(|e| format!("Task join error: {}", e))?
    }

    /// Upload M4B to Audiobookshelf and trigger scan
    pub async fn upload_and_scan(
        file_path: &str,
        config: &AppConfig,
    ) -> Result<(), String> {
        let client = reqwest::Client::new();

        // Read file
        let file_bytes = tokio::fs::read(file_path)
            .await
            .map_err(|e| format!("Failed to read file: {}", e))?;

        let file_name = Path::new(file_path)
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        // Create multipart form
        let part = reqwest::multipart::Part::bytes(file_bytes)
            .file_name(file_name);

        let form = reqwest::multipart::Form::new().part("file", part);

        // Upload
        let upload_url = format!("{}/api/upload", config.abs_host);
        let response = client
            .post(&upload_url)
            .header("Authorization", format!("Bearer {}", config.abs_token))
            .multipart(form)
            .send()
            .await
            .map_err(|e| format!("Upload request failed: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Upload failed with status: {}", response.status()));
        }

        // Trigger library scan
        let scan_url = format!("{}/api/libraries/{}/scan", config.abs_host, config.abs_library_id);
        let scan_response = client
            .post(&scan_url)
            .header("Authorization", format!("Bearer {}", config.abs_token))
            .send()
            .await
            .map_err(|e| format!("Scan request failed: {}", e))?;

        if !scan_response.status().is_success() {
            return Err(format!("Scan trigger failed with status: {}", scan_response.status()));
        }

        Ok(())
    }

    /// Test connection to Audiobookshelf server
    #[allow(dead_code)]
    pub async fn test_connection(config: &AppConfig) -> Result<String, String> {
        let client = reqwest::Client::new();
        let url = format!("{}/api/libraries", config.abs_host);
        
        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", config.abs_token))
            .send()
            .await
            .map_err(|e| format!("Connection failed: {}", e))?;

        if response.status().is_success() {
            Ok("Connection successful!".to_string())
        } else {
            Err(format!("Authentication failed: {}", response.status()))
        }
    }

    /// Fetch image bytes from URL
    pub async fn fetch_image(url: &str) -> Result<Vec<u8>, String> {
        let client = reqwest::Client::new();
        let response = client.get(url)
            .send()
            .await
            .map_err(|e| format!("Failed to fetch image: {}", e))?;

        if !response.status().is_success() {
            return Err(format!("Image fetch failed: {}", response.status()));
        }

        let bytes = response.bytes()
            .await
            .map_err(|e| format!("Failed to get bytes: {}", e))?;
            
        Ok(bytes.to_vec())
    }

    /// Resolve output path based on template and metadata
    pub fn resolve_output_path(config: &AppConfig, metadata: &crate::models::metadata::BookMetadata) -> Option<PathBuf> {
        let base = config.local_library.as_ref()?;
        let mut path = PathBuf::from(base);

        let mut template = config.path_template.clone();
        
        let author = metadata.authors.first().map(|s| s.as_str()).unwrap_or("Unknown Author");
        let title = &metadata.title;
        let series = metadata.series_name.as_deref().unwrap_or("");
        let series_num = metadata.series_number.as_deref().unwrap_or("");
        let disk_num = metadata.disk_number.as_deref().unwrap_or("");
        let chapter_num = metadata.chapter_number.as_deref().unwrap_or("");
        let year = metadata.release_date.as_deref().map(|d| d.split('-').next().unwrap_or("")).unwrap_or("");
        let quality = metadata.quality.as_deref().unwrap_or("");
        let asin = metadata.asin.as_deref().unwrap_or("");

        // Simple replacements
        template = template.replace("{Author}", &Self::sanitize_filename(author));
        template = template.replace("{Title}", &Self::sanitize_filename(title));
        template = template.replace("{Series}", &Self::sanitize_filename(series));
        template = template.replace("{Year}", &Self::sanitize_filename(year));
        template = template.replace("{ASIN}", &Self::sanitize_filename(asin));
        template = template.replace("{Quality}", &Self::sanitize_filename(quality));

        // Replacements with potential padding
        template = Self::replace_with_padding(template, "SeriesNumber", series_num);
        template = Self::replace_with_padding(template, "DiskNumber", disk_num);
        template = Self::replace_with_padding(template, "ChapterNumber", chapter_num);

        // Fallback for non-padded versions if not already replaced
        template = template.replace("{SeriesNumber}", &Self::sanitize_filename(series_num));
        template = template.replace("{DiskNumber}", &Self::sanitize_filename(disk_num));
        template = template.replace("{ChapterNumber}", &Self::sanitize_filename(chapter_num));

        for part in template.split('/') {
            if !part.is_empty() {
                path.push(part);
            }
        }

        if path.extension().map(|e| e != "m4b").unwrap_or(true) {
            path.set_extension("m4b");
        }

        Some(path)
    }

    fn replace_with_padding(mut template: String, token: &str, value: &str) -> String {
        let pattern = format!("{{{}:", token);
        while let Some(start) = template.find(&pattern) {
            if let Some(end) = template[start..].find('}') {
                let full_token = &template[start..start + end + 1];
                let padding_spec = &full_token[pattern.len()..full_token.len() - 1]; // e.g. "00"
                
                let formatted = if let Ok(val_int) = value.parse::<u32>() {
                    format!("{:0>width$}", val_int, width = padding_spec.len())
                } else {
                    value.to_string()
                };
                
                template = template.replace(full_token, &Self::sanitize_filename(&formatted));
            } else {
                break;
            }
        }
        template
    }

    fn sanitize_filename(s: &str) -> String {
        s.chars()
            .map(|c| if c.is_alphanumeric() || " _-.".contains(c) { c } else { '_' })
            .collect::<String>()
            .trim()
            .to_string()
    }

    /// Build chapter metadata from ChapterObjects (from UI)
    pub fn build_chapters_from_objects(chapters: &[crate::models::chapter::ChapterObject]) -> String {
        let mut metadata_file = String::from(";FFMETADATA1\n");

        for ch in chapters {
            let start_ms = ch.start_time();
            let duration_ms = ch.duration();
            let end_ms = start_ms + duration_ms;
            let title = ch.title();

            metadata_file.push_str("[CHAPTER]\n");
            metadata_file.push_str("TIMEBASE=1/1000\n");
            metadata_file.push_str(&format!("START={}\n", start_ms));
            metadata_file.push_str(&format!("END={}\n", end_ms));
            metadata_file.push_str(&format!("title={}\n", title));
        }

        metadata_file
    }
}
