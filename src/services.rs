use tokio;
use anyhow::Result;
use serde::{Deserialize, Serialize};

// Define the BookMetadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub author: String,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub duration: Option<String>,
    pub narrator: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<String>,
}

impl Default for BookMetadata {
    fn default() -> Self {
        BookMetadata {
            title: String::new(),
            author: String::new(),
            isbn: None,
            asin: None,
            description: None,
            cover_url: None,
            duration: None,
            narrator: None,
            publisher: None,
            publish_year: None,
        }
    }
}

// Define the AudioService struct
pub struct AudioService;

impl AudioService {
    // Method to fetch single metadata
    pub async fn fetch_metadata(query: &str) -> Result<BookMetadata, String> {
        // This is a placeholder implementation
        // In a real implementation, this would call an API like Audible or Google Books
        
        // Simulate API call delay
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
        
        // Return mock data
        Ok(BookMetadata {
            title: format!("Book Title for {}", query),
            author: "Sample Author".to_string(),
            isbn: Some("1234567890".to_string()),
            asin: Some("B012345678".to_string()),
            description: Some("This is a sample book description.".to_string()),
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            duration: Some("10 hours".to_string()),
            narrator: Some("Sample Narrator".to_string()),
            publisher: Some("Sample Publisher".to_string()),
            publish_year: Some("2023".to_string()),
        })
    }
    
    // Method to search for multiple results
    pub async fn search_metadata(query: &str, by_asin: bool) -> Result<Vec<BookMetadata>, String> {
        if by_asin {
            // Search by ASIN/ISBN
            Self::search_by_identifier(query).await
        } else {
            // Search by title/author
            Self::search_by_query(query).await
        }
    }
    
    // Search by ASIN or ISBN
    async fn search_by_identifier(identifier: &str) -> Result<Vec<BookMetadata>, String> {
        // Check if it looks like an ASIN (starts with B and is 10 characters)
        let is_asin = identifier.len() == 10 && identifier.starts_with('B');
        
        if is_asin {
            // Try Audnexus API first for ASIN searches
            println!("[DEBUG] ASIN search requested: {}", identifier);
            if let Ok(results) = Self::search_audnexus_by_asin(identifier).await {
                if !results.is_empty() {
                    println!("[DEBUG] Found {} results from Audnexus", results.len());
                    return Ok(results);
                }
            }
            println!("[DEBUG] Audnexus search failed, trying Open Library and Google Books as fallback...");
        }
        
        // Try Open Library first (supports ISBN, not ASIN)
        if let Ok(results) = Self::search_open_library_by_isbn(identifier).await {
            if !results.is_empty() {
                return Ok(results);
            }
        }
        
        // Try Google Books as fallback
        if let Ok(results) = Self::search_google_books_by_isbn(identifier).await {
            if !results.is_empty() {
                return Ok(results);
            }
        }
        
        // If it's an ASIN, provide a more helpful error message
        if is_asin {
            Err(format!("No results found for ASIN: {}. The book may not be available in the Audnexus database.", identifier))
        } else {
            Err(format!("No results found for identifier: {}", identifier))
        }
    }
    
    // Search Audnexus API by ASIN (https://api.audnex.us)
    async fn search_audnexus_by_asin(asin: &str) -> Result<Vec<BookMetadata>, String> {
        let client = reqwest::Client::new();
        let url = format!("https://api.audnex.us/books/{}", urlencoding::encode(asin));
        
        println!("[DEBUG] Audnexus URL: {}", url);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Audnexus request failed: {}", e))?;
        
        if !response.status().is_success() {
            if response.status() == 404 {
                return Ok(vec![]); // No results, not an error
            }
            return Err(format!("Audnexus returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Audnexus response: {}", e))?;
        
        if json.get("asin").is_none() {
            return Ok(vec![]); // Invalid response
        }
        
        if let Some(metadata) = Self::parse_audnexus_book(&json) {
            Ok(vec![metadata])
        } else {
            Ok(vec![])
        }
    }
    
    // Parse Audnexus book response
    fn parse_audnexus_book(book: &serde_json::Value) -> Option<BookMetadata> {
        let title = book.get("title")?.as_str()?.to_string();
        
        // Extract authors
        let author = if let Some(authors) = book.get("authors").and_then(|a| a.as_array()) {
            authors.iter()
                .filter_map(|a| a.get("name").and_then(|n| n.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };
        
        // Extract narrators
        let narrator = if let Some(narrators) = book.get("narrators").and_then(|n| n.as_array()) {
            narrators.iter()
                .filter_map(|n| n.get("name").and_then(|name| name.as_str()))
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            String::new()
        };
        
        let asin = book.get("asin").and_then(|a| a.as_str()).map(|s| s.to_string());
        let isbn = book.get("isbn").and_then(|i| i.as_str()).map(|s| s.to_string());
        let description = book.get("summary").and_then(|s| s.as_str()).map(|s| s.to_string());
        let cover_url = book.get("image").and_then(|i| i.as_str()).map(|s| s.to_string());
        
        // Extract duration (runtimeLengthMin)
        let duration = book.get("runtimeLengthMin")
            .and_then(|d| d.as_u64())
            .map(|mins| {
                let hours = mins / 60;
                let minutes = mins % 60;
                if hours > 0 {
                    format!("{} hours {} minutes", hours, minutes)
                } else {
                    format!("{} minutes", minutes)
                }
            });
        
        let publisher = book.get("publisherName").and_then(|p| p.as_str()).map(|s| s.to_string());
        
        // Extract publish year from releaseDate
        let publish_year = book.get("releaseDate")
            .and_then(|d| d.as_str())
            .and_then(|d| d.split('-').next())
            .map(|s| s.to_string());
        
        Some(BookMetadata {
            title,
            author,
            isbn,
            asin,
            description,
            cover_url,
            duration,
            narrator: if narrator.is_empty() { None } else { Some(narrator) },
            publisher,
            publish_year,
        })
    }
    
    // Search by title/author query
    async fn search_by_query(query: &str) -> Result<Vec<BookMetadata>, String> {
        // Try Open Library first
        if let Ok(results) = Self::search_open_library(query).await {
            if !results.is_empty() {
                return Ok(results);
            }
        }
        
        // Try Google Books as fallback
        if let Ok(results) = Self::search_google_books(query).await {
            if !results.is_empty() {
                return Ok(results);
            }
        }
        
        Err(format!("No results found for query: {}", query))
    }
    
    // Search Open Library API
    async fn search_open_library(query: &str) -> Result<Vec<BookMetadata>, String> {
        let client = reqwest::Client::new();
        let url = format!("https://openlibrary.org/search.json?q={}&limit=10", 
                         urlencoding::encode(query));
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Open Library request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Open Library returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Open Library response: {}", e))?;
        
        let mut results = Vec::new();
        if let Some(docs) = json.get("docs").and_then(|d| d.as_array()) {
            for doc in docs.iter().take(10) {
                if let Some(metadata) = Self::parse_open_library_doc(doc) {
                    results.push(metadata);
                }
            }
        }
        
        Ok(results)
    }
    
    // Search Open Library by ISBN
    async fn search_open_library_by_isbn(isbn: &str) -> Result<Vec<BookMetadata>, String> {
        let client = reqwest::Client::new();
        let url = format!("https://openlibrary.org/isbn/{}.json", isbn);
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Open Library ISBN request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Open Library ISBN returned status: {}", response.status()));
        }
        
        let doc: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Open Library ISBN response: {}", e))?;
        
        if let Some(metadata) = Self::parse_open_library_doc(&doc) {
            Ok(vec![metadata])
        } else {
            Ok(vec![])
        }
    }
    
    // Parse Open Library document
    fn parse_open_library_doc(doc: &serde_json::Value) -> Option<BookMetadata> {
        let title = doc.get("title")?.as_str()?.to_string();
        
        // Extract author(s)
        let author = if let Some(authors) = doc.get("author_name").and_then(|a| a.as_array()) {
            authors.iter()
                .filter_map(|a| a.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        } else if let Some(author_key) = doc.get("author_key").and_then(|a| a.as_array()) {
            if let Some(first_key) = author_key.first().and_then(|k| k.as_str()) {
                // Try to get author name from key (simplified)
                format!("Author {}", first_key)
            } else {
                "Unknown Author".to_string()
            }
        } else {
            "Unknown Author".to_string()
        };
        
        // Extract ISBN/ASIN
        let isbn = doc.get("isbn")
            .and_then(|i| i.as_array())
            .and_then(|arr| arr.first())
            .and_then(|v| v.as_str())
            .map(|s| s.to_string());
        
        // Extract cover URL
        let cover_url = if let Some(cover_id) = doc.get("cover_i").and_then(|c| c.as_i64()) {
            Some(format!("https://covers.openlibrary.org/b/id/{}-L.jpg", cover_id))
        } else if let Some(cover_id) = doc.get("cover_edition_key").and_then(|c| c.as_str()) {
            Some(format!("https://covers.openlibrary.org/b/olid/{}-L.jpg", cover_id))
        } else {
            None
        };
        
        // Extract publish year
        let publish_year = doc.get("first_publish_year")
            .or_else(|| doc.get("publish_year"))
            .and_then(|y| {
                if let Some(year) = y.as_i64() {
                    Some(year.to_string())
                } else if let Some(years) = y.as_array() {
                    years.first().and_then(|y| y.as_i64()).map(|y| y.to_string())
                } else {
                    None
                }
            });
        
        // Extract publisher
        let publisher = doc.get("publisher")
            .and_then(|p| {
                if let Some(pub_str) = p.as_str() {
                    Some(pub_str.to_string())
                } else if let Some(pub_arr) = p.as_array() {
                    pub_arr.first().and_then(|p| p.as_str()).map(|s| s.to_string())
                } else {
                    None
                }
            });
        
        Some(BookMetadata {
            title,
            author,
            isbn,
            asin: None, // Open Library doesn't provide ASIN
            description: None,
            cover_url,
            duration: None,
            narrator: None,
            publisher,
            publish_year,
        })
    }
    
    // Search Google Books API
    async fn search_google_books(query: &str) -> Result<Vec<BookMetadata>, String> {
        let client = reqwest::Client::new();
        let url = format!("https://www.googleapis.com/books/v1/volumes?q={}&maxResults=10", 
                         urlencoding::encode(query));
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| format!("Google Books request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("Google Books returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Google Books response: {}", e))?;
        
        let mut results = Vec::new();
        if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
            for item in items.iter().take(10) {
                if let Some(metadata) = Self::parse_google_books_item(item) {
                    results.push(metadata);
                }
            }
        }
        
        Ok(results)
    }
    
    // Search Google Books by ISBN
    async fn search_google_books_by_isbn(isbn: &str) -> Result<Vec<BookMetadata>, String> {
        Self::search_google_books(&format!("isbn:{}", isbn)).await
    }
    
    // Parse Google Books item
    fn parse_google_books_item(item: &serde_json::Value) -> Option<BookMetadata> {
        let volume_info = item.get("volumeInfo")?;
        
        let title = volume_info.get("title")?.as_str()?.to_string();
        
        // Extract author(s)
        let author = if let Some(authors) = volume_info.get("authors").and_then(|a| a.as_array()) {
            authors.iter()
                .filter_map(|a| a.as_str())
                .collect::<Vec<_>>()
                .join(", ")
        } else {
            "Unknown Author".to_string()
        };
        
        // Extract ISBN
        let isbn = volume_info.get("industryIdentifiers")
            .and_then(|ids| ids.as_array())
            .and_then(|arr| {
                // Prefer ISBN_13, fallback to ISBN_10
                arr.iter().find(|id| {
                    id.get("type").and_then(|t| t.as_str()) == Some("ISBN_13")
                })
                .or_else(|| arr.first())
            })
            .and_then(|id| id.get("identifier"))
            .and_then(|i| i.as_str())
            .map(|s| s.to_string());
        
        // Extract cover URL
        let cover_url = volume_info.get("imageLinks")
            .and_then(|img| img.get("large"))
            .or_else(|| volume_info.get("imageLinks").and_then(|img| img.get("thumbnail")))
            .and_then(|url| url.as_str())
            .map(|s| s.to_string());
        
        // Extract description
        let description = volume_info.get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());
        
        // Extract publish year
        let publish_year = volume_info.get("publishedDate")
            .and_then(|d| d.as_str())
            .and_then(|date| {
                // Extract year from date string (format: "YYYY" or "YYYY-MM-DD")
                date.split('-').next().map(|s| s.to_string())
            });
        
        // Extract publisher
        let publisher = volume_info.get("publisher")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());
        
        Some(BookMetadata {
            title,
            author,
            isbn,
            asin: None, // Google Books doesn't provide ASIN
            description,
            cover_url,
            duration: None,
            narrator: None,
            publisher,
            publish_year,
        })
    }
    
    // Method to convert audio files to M4B
    pub async fn convert_to_m4b(input_files: Vec<String>, output_path: &str) -> Result<(), String> {
        // Implementation would use FFmpeg to convert files
        // This is a placeholder
        println!("Converting {} files to M4B at {}", input_files.len(), output_path);
        Ok(())
    }
    
    // Method to upload to Audiobookshelf
    pub async fn upload_to_audiobookshelf(
        host: &str,
        token: &str,
        library_id: &str,
        file_path: &str,
    ) -> Result<(), String> {
        // Implementation would use reqwest to upload to Audiobookshelf
        // This is a placeholder
        println!("Uploading {} to Audiobookshelf at {} with library {}", file_path, host, library_id);
        Ok(())
    }
    
    // Method to scan library in Audiobookshelf
    pub async fn scan_library(host: &str, token: &str, library_id: &str) -> Result<(), String> {
        // Implementation would use reqwest to trigger library scan
        // This is a placeholder
        println!("Scanning library {} at {}", library_id, host);
        Ok(())
    }
}

// Add a helper function to get metadata from a file
pub async fn get_file_metadata(file_path: &str) -> Result<BookMetadata, String> {
    // This would extract metadata from audio files
    // For now, return mock data
    Ok(BookMetadata {
        title: format!("Audio File: {}", file_path),
        author: "Unknown Author".to_string(),
        isbn: None,
        asin: None,
        description: None,
        cover_url: None,
        duration: Some("00:00:00".to_string()),
        narrator: None,
        publisher: None,
        publish_year: None,
    })
}