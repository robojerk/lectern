use tokio;
use anyhow::Result;
use serde::{Deserialize, Serialize};

// Define the BookMetadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BookMetadata {
    pub title: String,
    pub subtitle: Option<String>,
    pub author: String,
    pub isbn: Option<String>,
    pub asin: Option<String>,
    pub description: Option<String>,
    pub cover_url: Option<String>,
    pub duration: Option<String>,
    pub narrator: Option<String>,
    pub publisher: Option<String>,
    pub publish_year: Option<String>,
    pub series: Option<String>,
    pub series_number: Option<String>,
    pub genre: Option<String>,
    pub tags: Option<String>, // Comma-separated tags
    pub language: Option<String>,
    pub explicit: Option<bool>,
    pub abridged: Option<bool>,
}

impl Default for BookMetadata {
    fn default() -> Self {
        BookMetadata {
            title: String::new(),
            subtitle: None,
            author: String::new(),
            isbn: None,
            asin: None,
            description: None,
            cover_url: None,
            duration: None,
            narrator: None,
            publisher: None,
            publish_year: None,
            series: None,
            series_number: None,
            genre: None,
            tags: None,
            language: None,
            explicit: None,
            abridged: None,
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
            subtitle: None,
            author: "Sample Author".to_string(),
            isbn: Some("1234567890".to_string()),
            asin: Some("B012345678".to_string()),
            description: Some("This is a sample book description.".to_string()),
            cover_url: Some("https://example.com/cover.jpg".to_string()),
            duration: Some("10 hours".to_string()),
            narrator: Some("Sample Narrator".to_string()),
            publisher: Some("Sample Publisher".to_string()),
            publish_year: Some("2023".to_string()),
            series: None,
            series_number: None,
            genre: None,
            tags: None,
            language: None,
            explicit: None,
            abridged: None,
        })
    }
    
    // Method to search for multiple results
    pub async fn search_metadata(query: &str, by_asin: bool, provider: Option<&str>) -> Result<Vec<BookMetadata>, String> {
        let provider = provider.unwrap_or("auto");
        
        // Check if query looks like an ISBN or ASIN (even if by_asin is false)
        let looks_like_identifier = query.len() >= 10 && (query.len() <= 13) && 
            (query.chars().all(|c| c.is_ascii_alphanumeric()) || 
             (query.len() == 13 && query.chars().all(|c| c.is_ascii_digit() || c == '-')));
        let is_asin_format = query.len() == 10 && query.starts_with('B');
        
        // If provider is audnexus and query looks like identifier, treat as identifier search
        if provider == "audnexus" && (by_asin || looks_like_identifier) {
            println!("[DEBUG] Audnexus identifier search requested: {} (ASIN format: {})", query, is_asin_format);
            if let Ok(results) = Self::search_audnexus_by_asin(query).await {
                if !results.is_empty() {
                    return Ok(results);
                }
            }
            // If Audnexus fails and it's definitely an ASIN, return error
            if is_asin_format {
                return Err(format!("No results found for ASIN: {}. The book may not be available in the Audnexus database.", query));
            }
            // For ISBNs, fall through to other providers
        }
        
        if by_asin || (provider == "audnexus" && looks_like_identifier) {
            // Search by ASIN/ISBN
            Self::search_by_identifier(query).await
        } else {
            // Search by title/author with provider selection
            Self::search_by_query_with_provider(query, Some(provider)).await
        }
    }
    
    // Search by query with provider selection
    async fn search_by_query_with_provider(query: &str, provider: Option<&str>) -> Result<Vec<BookMetadata>, String> {
        let provider = provider.unwrap_or("auto");
        
        println!("[DEBUG] search_by_query_with_provider called with provider: '{}', query: '{}'", provider, query);
        
        match provider {
            "audnexus" => {
                // Audnexus doesn't support general title/author search, only ASIN/ISBN
                // But we can try to use Audible catalog API as a fallback
                println!("[DEBUG] Audnexus provider selected for title search, trying Audible catalog API as fallback");
                // Try US region first
                Self::search_audible(query, "us").await
            }
            "audible_com" => {
                println!("[DEBUG] EXCLUSIVELY using Audible.com provider for query: '{}'", query);
                Self::search_audible(query, "us").await
            }
            "audible_ca" => {
                println!("[DEBUG] EXCLUSIVELY using Audible.ca provider for query: '{}'", query);
                Self::search_audible(query, "ca").await
            }
            "open_library" => {
                println!("[DEBUG] EXCLUSIVELY using Open Library provider for query: '{}'", query);
                Self::search_open_library(query).await
            }
            "google_books" => {
                println!("[DEBUG] EXCLUSIVELY using Google Books provider for query: '{}'", query);
                Self::search_google_books(query).await
            }
            "itunes" => {
                println!("[DEBUG] EXCLUSIVELY using iTunes provider for query: '{}'", query);
                Self::search_itunes(query).await
            }
            "fantlab" => {
                println!("[DEBUG] EXCLUSIVELY using FantLab.ru provider for query: '{}'", query);
                Self::search_fantlab(query).await
            }
            _ => {
                // "auto" - try all providers
                println!("[DEBUG] Using AUTO mode - trying all providers for query: '{}'", query);
                Self::search_by_query(query).await
            }
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
        
        // Extract subtitle
        let subtitle = book.get("subtitle").and_then(|s| s.as_str()).map(|s| s.to_string());
        
        // Extract series information
        let series = book.get("seriesName").and_then(|s| s.as_str()).map(|s| s.to_string());
        let series_number = book.get("seriesSequence")
            .and_then(|n| {
                if let Some(s) = n.as_str() {
                    Some(s.to_string())
                } else if let Some(u) = n.as_u64() {
                    Some(u.to_string())
                } else {
                    None
                }
            });
        
        // Extract genre (from genres array)
        let genre = book.get("genres")
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|g| g.get("name").and_then(|n| n.as_str()))
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty());
        
        // Extract language
        let language = book.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());
        
        // Extract explicit flag
        let explicit = book.get("isExplicit").and_then(|e| e.as_bool());
        
        // Extract abridged flag
        let abridged = book.get("formatType")
            .and_then(|f| f.as_str())
            .map(|f| f.to_lowercase().contains("abridged"));
        
        let metadata = BookMetadata {
            title,
            subtitle,
            author,
            isbn,
            asin,
            description,
            cover_url,
            duration,
            narrator: if narrator.is_empty() { None } else { Some(narrator) },
            publisher,
            publish_year,
            series,
            series_number,
            genre,
            tags: None, // Audnexus doesn't provide tags
            language,
            explicit,
            abridged,
        };
        
        // Debug: log what fields were extracted
        println!("[DEBUG] Audnexus extracted metadata - Title: '{}', Author: '{}', Subtitle: {:?}, ISBN: {:?}, ASIN: {:?}, Publisher: {:?}, Year: {:?}, Series: {:?}, Series#: {:?}, Genre: {:?}, Language: {:?}, Narrator: {:?}, Explicit: {:?}, Abridged: {:?}", 
            metadata.title, metadata.author, metadata.subtitle, metadata.isbn, metadata.asin,
            metadata.publisher, metadata.publish_year, metadata.series, metadata.series_number,
            metadata.genre, metadata.language, metadata.narrator, metadata.explicit, metadata.abridged);
        
        Some(metadata)
    }
    
    // Search by title/author query (auto mode - tries all providers)
    // Default priority: Audible (best results) -> Open Library -> Google Books -> iTunes
    async fn search_by_query(query: &str) -> Result<Vec<BookMetadata>, String> {
        println!("[DEBUG] search_by_query (AUTO mode) called with: '{}'", query);
        
        let mut results = Vec::new();
        
        // Try providers in order of preference
        // 1. Audible.com (best results for audiobooks, uses catalog API + Audnexus)
        match Self::search_audible(query, "us").await {
            Ok(mut audible_results) if !audible_results.is_empty() => {
                println!("[DEBUG] Audible.com returned {} results", audible_results.len());
                results = audible_results;
            },
            Ok(_) => println!("[DEBUG] Audible.com returned empty results"),
            Err(e) => println!("[DEBUG] Audible.com error: {}", e),
        }
        
        // 2. Open Library (good coverage, free)
        if results.is_empty() {
            match Self::search_open_library(query).await {
                Ok(mut open_lib_results) if !open_lib_results.is_empty() => {
                    println!("[DEBUG] Open Library returned {} results", open_lib_results.len());
                    results = open_lib_results;
                },
                Ok(_) => println!("[DEBUG] Open Library returned empty results"),
                Err(e) => println!("[DEBUG] Open Library error: {}", e),
            }
        }
        
        // 3. Google Books (good coverage, free)
        if results.is_empty() {
            match Self::search_google_books(query).await {
                Ok(mut google_results) if !google_results.is_empty() => {
                    println!("[DEBUG] Google Books returned {} results", google_results.len());
                    results = google_results;
                },
                Ok(_) => println!("[DEBUG] Google Books returned empty results"),
                Err(e) => println!("[DEBUG] Google Books error: {}", e),
            }
        }
        
        // 4. iTunes (good for audiobooks)
        if results.is_empty() {
            match Self::search_itunes(query).await {
                Ok(mut itunes_results) if !itunes_results.is_empty() => {
                    println!("[DEBUG] iTunes returned {} results", itunes_results.len());
                    results = itunes_results;
                },
                Ok(_) => println!("[DEBUG] iTunes returned empty results"),
                Err(e) => println!("[DEBUG] iTunes error: {}", e),
            }
        }
        
        // Try to enrich results with ASIN from Audnexus if we have ISBN
        for book in &mut results {
            if book.asin.is_none() && book.isbn.is_some() {
                if let Ok(asin_result) = Self::search_audnexus_by_isbn(book.isbn.as_ref().unwrap()).await {
                    if let Some(enriched_book) = asin_result.first() {
                        if enriched_book.asin.is_some() {
                            book.asin = enriched_book.asin.clone();
                            println!("[DEBUG] Enriched book '{}' with ASIN: {:?}", book.title, book.asin);
                        }
                    }
                }
            }
        }
        
        if results.is_empty() {
            println!("[DEBUG] No results found from any provider for: '{}'", query);
            Err(format!("No results found for query: {}", query))
        } else {
            Ok(results)
        }
    }
    
    // Search Audnexus by ISBN to get ASIN
    async fn search_audnexus_by_isbn(isbn: &str) -> Result<Vec<BookMetadata>, String> {
        // Audnexus doesn't have a direct ISBN endpoint, but we can try searching by ISBN
        // For now, return empty - this would need a different approach
        Ok(Vec::new())
    }
    
    // Search Open Library API
    async fn search_open_library(query: &str) -> Result<Vec<BookMetadata>, String> {
        println!("[DEBUG] Searching Open Library for: {}", query);
        let client = reqwest::Client::new();
        let url = format!("https://openlibrary.org/search.json?q={}&limit=10", 
                         urlencoding::encode(query));
        println!("[DEBUG] Open Library URL: {}", url);
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| {
                println!("[DEBUG] Open Library request error: {}", e);
                format!("Open Library request failed: {}", e)
            })?;
        
        println!("[DEBUG] Open Library response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(format!("Open Library returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| {
                println!("[DEBUG] Open Library parse error: {}", e);
                format!("Failed to parse Open Library response: {}", e)
            })?;
        
        let mut results = Vec::new();
        if let Some(docs) = json.get("docs").and_then(|d| d.as_array()) {
            println!("[DEBUG] Open Library found {} documents", docs.len());
            for doc in docs.iter().take(10) {
                if let Some(metadata) = Self::parse_open_library_doc(doc) {
                    results.push(metadata);
                }
            }
        } else {
            println!("[DEBUG] Open Library: No 'docs' array in response");
        }
        
        println!("[DEBUG] Open Library returning {} results", results.len());
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
        
        // Extract subtitle
        let subtitle = doc.get("subtitle").and_then(|s| s.as_str()).map(|s| s.to_string());
        
        // Extract description - try multiple fields
        let description = doc.get("first_sentence")
            .and_then(|d| {
                if let Some(sentences) = d.as_array() {
                    sentences.first()
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                } else {
                    d.as_str().map(|s| s.to_string())
                }
            })
            .or_else(|| {
                // Try alternative description fields
                doc.get("description")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string())
            })
            .or_else(|| {
                doc.get("abstract")
                    .and_then(|d| d.as_str())
                    .map(|s| s.to_string())
            });
        
        // Extract subject/genre (Open Library uses "subject" field)
        let genre = doc.get("subject")
            .and_then(|s| s.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|s| s.as_str())
                    .take(3) // Limit to first 3 subjects
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty());
        
        // Extract language
        let language = doc.get("language")
            .and_then(|l| {
                if let Some(lang_arr) = l.as_array() {
                    lang_arr.first()
                        .and_then(|l| l.as_str())
                        .map(|s| s.to_string())
                } else {
                    l.as_str().map(|s| s.to_string())
                }
            });
        
        let metadata = BookMetadata {
            title,
            subtitle,
            author,
            isbn,
            asin: None, // Open Library doesn't provide ASIN
            description,
            cover_url,
            duration: None,
            narrator: None,
            publisher,
            publish_year,
            series: None,
            series_number: None,
            genre,
            tags: None,
            language,
            explicit: None,
            abridged: None,
        };
        
        // Debug: log what fields were extracted
        println!("[DEBUG] Open Library extracted metadata - Title: '{}', Author: '{}', Subtitle: {:?}, ISBN: {:?}, Publisher: {:?}, Year: {:?}, Genre: {:?}, Language: {:?}, Description: {:?}", 
            metadata.title, metadata.author, metadata.subtitle, metadata.isbn, 
            metadata.publisher, metadata.publish_year, metadata.genre, metadata.language,
            metadata.description.as_ref().map(|d| if d.len() > 50 { format!("{}...", &d[..50]) } else { d.clone() }));
        
        Some(metadata)
    }
    
    // Search Google Books API
    async fn search_google_books(query: &str) -> Result<Vec<BookMetadata>, String> {
        println!("[DEBUG] Searching Google Books for: {}", query);
        let client = reqwest::Client::new();
        let url = format!("https://www.googleapis.com/books/v1/volumes?q={}&maxResults=10", 
                         urlencoding::encode(query));
        println!("[DEBUG] Google Books URL: {}", url);
        
        let response = client.get(&url)
            .send()
            .await
            .map_err(|e| {
                println!("[DEBUG] Google Books request error: {}", e);
                format!("Google Books request failed: {}", e)
            })?;
        
        println!("[DEBUG] Google Books response status: {}", response.status());
        
        if !response.status().is_success() {
            return Err(format!("Google Books returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| {
                println!("[DEBUG] Google Books parse error: {}", e);
                format!("Failed to parse Google Books response: {}", e)
            })?;
        
        let mut results = Vec::new();
        if let Some(items) = json.get("items").and_then(|i| i.as_array()) {
            println!("[DEBUG] Google Books found {} items", items.len());
            for item in items.iter().take(10) {
                if let Some(metadata) = Self::parse_google_books_item(item) {
                    results.push(metadata);
                }
            }
        } else {
            println!("[DEBUG] Google Books: No 'items' array in response");
        }
        
        println!("[DEBUG] Google Books returning {} results", results.len());
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
        
        // Extract cover URL - prefer largest available image (like Audiobookshelf)
        let cover_url = volume_info.get("imageLinks")
            .and_then(|img| {
                // Try to get the largest image - check for extraLarge, large, medium, small, thumbnail
                // Select the largest available (assuming keys are ordered or we check in order)
                img.get("extraLarge")
                    .or_else(|| img.get("large"))
                    .or_else(|| img.get("medium"))
                    .or_else(|| img.get("small"))
                    .or_else(|| img.get("thumbnail"))
            })
            .and_then(|url| url.as_str())
            .map(|s| {
                // Ensure HTTPS (like Audiobookshelf does)
                s.replace("http://", "https://")
            });
        
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
        
        // Extract subtitle
        let subtitle = volume_info.get("subtitle").and_then(|s| s.as_str()).map(|s| s.to_string());
        
        // Extract publisher
        let publisher = volume_info.get("publisher")
            .and_then(|p| p.as_str())
            .map(|s| s.to_string());
        
        // Extract categories/genre
        let genre = volume_info.get("categories")
            .and_then(|c| c.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|c| c.as_str())
                    .take(3) // Limit to first 3 categories
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .filter(|s| !s.is_empty());
        
        // Extract language
        let language = volume_info.get("language").and_then(|l| l.as_str()).map(|s| s.to_string());
        
        let metadata = BookMetadata {
            title,
            subtitle,
            author,
            isbn,
            asin: None, // Google Books doesn't provide ASIN
            description,
            cover_url,
            duration: None,
            narrator: None,
            publisher,
            publish_year,
            series: None,
            series_number: None,
            genre,
            tags: None,
            language,
            explicit: None,
            abridged: None,
        };
        
        // Debug: log what fields were extracted
        println!("[DEBUG] Google Books extracted metadata - Title: '{}', Author: '{}', Subtitle: {:?}, ISBN: {:?}, Publisher: {:?}, Year: {:?}, Genre: {:?}, Language: {:?}, Description: {:?}", 
            metadata.title, metadata.author, metadata.subtitle, metadata.isbn, 
            metadata.publisher, metadata.publish_year, metadata.genre, metadata.language,
            metadata.description.as_ref().map(|d| if d.len() > 50 { format!("{}...", &d[..50]) } else { d.clone() }));
        
        Some(metadata)
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
    
    // Fetch chapters from Audnexus by ASIN
    pub async fn fetch_chapters_by_asin(asin: &str) -> Result<Vec<crate::models::Chapter>, String> {
        
        let client = reqwest::Client::new();
        let url = format!("https://api.audnex.us/books/{}/chapters", urlencoding::encode(asin));
        
        println!("[DEBUG] Fetching chapters from Audnexus: {}", url);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Audnexus chapters request failed: {}", e))?;
        
        if !response.status().is_success() {
            if response.status() == 404 {
                return Err("No chapters found for this ASIN".to_string());
            }
            return Err(format!("Audnexus returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Audnexus chapters response: {}", e))?;
        
        // Audnexus returns chapters in a "chapters" array
        let chapters_array = json.get("chapters")
            .and_then(|c| c.as_array())
            .ok_or_else(|| "No chapters array in response".to_string())?;
        
        let mut chapters = Vec::new();
        for (index, chapter_json) in chapters_array.iter().enumerate() {
            // Audnexus chapter format: { "asin": "...", "brandIntroDurationMs": 0, "brandOutroDurationMs": 0, "isAccurate": true, "runtimeLengthMs": 1234567, "runtimeLengthSec": 1234, "chapters": [...] }
            // Each chapter has: "lengthMs", "startOffsetMs", "startOffsetSec", "title"
            
            let title = chapter_json.get("title")
                .and_then(|t| t.as_str())
                .unwrap_or(&format!("Chapter {}", index + 1))
                .to_string();
            
            let start_time_ms = chapter_json.get("startOffsetMs")
                .and_then(|s| s.as_u64())
                .unwrap_or(0);
            
            let duration_ms = chapter_json.get("lengthMs")
                .and_then(|d| d.as_u64())
                .unwrap_or(0);
            
            chapters.push(crate::models::Chapter {
                title,
                start_time: start_time_ms,
                duration: duration_ms,
                is_locked: false,
            });
        }
        
        println!("[DEBUG] Parsed {} chapters from Audnexus", chapters.len());
        Ok(chapters)
    }
    
    // Search Audible.com/ca directly
    // For ASIN: uses Audnexus API directly
    // For title/author: uses Audible Catalog API to find ASINs, then Audnexus for full details
    async fn search_audible(query: &str, region: &str) -> Result<Vec<BookMetadata>, String> {
        // Check if query looks like an ASIN
        let is_asin = query.len() == 10 && query.starts_with('B');
        
        if is_asin {
            // Direct ASIN lookup via Audnexus
            let client = reqwest::Client::new();
            let url = format!("https://api.audnex.us/books/{}?region={}", 
                             urlencoding::encode(query), region);
            
            println!("[DEBUG] Audible (region: {}) ASIN search URL: {}", region, url);
            
            let response = client.get(&url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| format!("Audible request failed: {}", e))?;
            
            if !response.status().is_success() {
                if response.status() == 404 {
                    return Ok(vec![]);
                }
                return Err(format!("Audible returned status: {}", response.status()));
            }
            
            let json: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse Audible response: {}", e))?;
            
            if json.get("asin").is_none() {
                return Ok(vec![]);
            }
            
            if let Some(metadata) = Self::parse_audnexus_book(&json) {
                Ok(vec![metadata])
            } else {
                Ok(vec![])
            }
        } else {
            // Title/author search: Use Audible Catalog API to find ASINs, then fetch details from Audnexus
            let client = reqwest::Client::new();
            
            // Map region to TLD
            let tld = match region {
                "ca" => ".ca",
                "uk" => ".co.uk",
                "au" => ".com.au",
                "fr" => ".fr",
                "de" => ".de",
                "jp" => ".co.jp",
                "it" => ".it",
                "in" => ".in",
                "es" => ".es",
                _ => ".com", // default to US
            };
            
            // Build query parameters for Audible Catalog API
            let mut query_params = vec![
                ("num_results", "10"),
                ("products_sort_by", "Relevance"),
                ("title", query),
            ];
            
            // Note: We could add author parameter if we had it, but for now just use title
            let query_string: String = query_params.iter()
                .map(|(k, v)| format!("{}={}", k, urlencoding::encode(v)))
                .collect::<Vec<_>>()
                .join("&");
            
            let url = format!("https://api.audible{}/1.0/catalog/products?{}", tld, query_string);
            
            println!("[DEBUG] Audible (region: {}) Catalog search URL: {}", region, url);
            
            let response = client.get(&url)
                .timeout(std::time::Duration::from_secs(10))
                .send()
                .await
                .map_err(|e| format!("Audible catalog request failed: {}", e))?;
            
            if !response.status().is_success() {
                return Err(format!("Audible catalog returned status: {}", response.status()));
            }
            
            let json: serde_json::Value = response.json().await
                .map_err(|e| format!("Failed to parse Audible catalog response: {}", e))?;
            
            // Extract ASINs from products
            let asins: Vec<String> = if let Some(products) = json.get("products").and_then(|p| p.as_array()) {
                products.iter()
                    .filter_map(|p| p.get("asin").and_then(|a| a.as_str()).map(|s| s.to_string()))
                    .collect()
            } else {
                return Ok(vec![]);
            };
            
            println!("[DEBUG] Audible catalog found {} products (ASINs)", asins.len());
            
            // Fetch full details for each ASIN from Audnexus
            let mut results = Vec::new();
            for asin in asins.iter().take(10) {
                if let Ok(mut asin_results) = Self::search_audnexus_by_asin_with_region(asin, region).await {
                    results.append(&mut asin_results);
                }
            }
            
            if results.is_empty() {
                Err(format!("No results found on Audible{} for: {}", tld, query))
            } else {
                Ok(results)
            }
        }
    }
    
    // Search Audnexus API by ASIN with region parameter
    async fn search_audnexus_by_asin_with_region(asin: &str, region: &str) -> Result<Vec<BookMetadata>, String> {
        let client = reqwest::Client::new();
        let url = format!("https://api.audnex.us/books/{}?region={}", 
                         urlencoding::encode(asin), region);
        
        println!("[DEBUG] Audnexus (region: {}) URL: {}", region, url);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("Audnexus request failed: {}", e))?;
        
        if !response.status().is_success() {
            if response.status() == 404 {
                return Ok(vec![]);
            }
            return Err(format!("Audnexus returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse Audnexus response: {}", e))?;
        
        if json.get("asin").is_none() {
            return Ok(vec![]);
        }
        
        if let Some(metadata) = Self::parse_audnexus_book(&json) {
            Ok(vec![metadata])
        } else {
            Ok(vec![])
        }
    }
    
    // Search iTunes Store API
    async fn search_itunes(query: &str) -> Result<Vec<BookMetadata>, String> {
        let client = reqwest::Client::new();
        // iTunes Search API - audiobook media type
        let url = format!("https://itunes.apple.com/search?term={}&media=audiobook&limit=10", 
                         urlencoding::encode(query));
        
        println!("[DEBUG] iTunes URL: {}", url);
        
        let response = client.get(&url)
            .timeout(std::time::Duration::from_secs(10))
            .send()
            .await
            .map_err(|e| format!("iTunes request failed: {}", e))?;
        
        if !response.status().is_success() {
            return Err(format!("iTunes returned status: {}", response.status()));
        }
        
        let json: serde_json::Value = response.json().await
            .map_err(|e| format!("Failed to parse iTunes response: {}", e))?;
        
        let mut results = Vec::new();
        if let Some(results_array) = json.get("results").and_then(|r| r.as_array()) {
            println!("[DEBUG] iTunes found {} results", results_array.len());
            for item in results_array.iter().take(10) {
                if let Some(metadata) = Self::parse_itunes_item(item) {
                    results.push(metadata);
                }
            }
        }
        
        if results.is_empty() {
            Err(format!("No results found on iTunes for: {}", query))
        } else {
            Ok(results)
        }
    }
    
    // Parse iTunes search result
    fn parse_itunes_item(item: &serde_json::Value) -> Option<BookMetadata> {
        let track_name = item.get("trackName")?.as_str()?.to_string();
        let artist_name = item.get("artistName")?.as_str()?.to_string();
        
        // Extract description
        let description = item.get("description")
            .and_then(|d| d.as_str())
            .map(|s| s.to_string());
        
        // Extract cover URL (artworkUrl100, artworkUrl512, or artworkUrl60)
        let cover_url = item.get("artworkUrl512")
            .or_else(|| item.get("artworkUrl100"))
            .or_else(|| item.get("artworkUrl60"))
            .and_then(|url| url.as_str())
            .map(|s| s.to_string());
        
        // Extract release date
        let publish_year = item.get("releaseDate")
            .and_then(|d| d.as_str())
            .and_then(|date| date.split('T').next())
            .and_then(|date| date.split('-').next())
            .map(|s| s.to_string());
        
        // Extract genre
        let genre = item.get("primaryGenreName")
            .and_then(|g| g.as_str())
            .map(|s| s.to_string());
        
        // Extract duration (trackTimeMillis is in milliseconds)
        let duration = item.get("trackTimeMillis")
            .and_then(|t| t.as_u64())
            .map(|ms| {
                let hours = ms / 3600000;
                let minutes = (ms % 3600000) / 60000;
                if hours > 0 {
                    format!("{} hours {} minutes", hours, minutes)
                } else {
                    format!("{} minutes", minutes)
                }
            });
        
        // Extract collection name (series)
        let series = item.get("collectionName")
            .and_then(|c| c.as_str())
            .map(|s| s.to_string());
        
        // Extract collection number (series position)
        let series_number = item.get("collectionViewUrl")
            .and_then(|_| item.get("trackNumber"))
            .and_then(|n| n.as_u64())
            .map(|n| n.to_string());
        
        // Extract language
        let language = item.get("country")
            .and_then(|c| c.as_str())
            .map(|_| "English".to_string()); // iTunes doesn't provide language directly
        
        // Extract ISBN (if available)
        let isbn = None; // iTunes doesn't provide ISBN in search results
        
        Some(BookMetadata {
            title: track_name,
            subtitle: None,
            author: artist_name,
            isbn,
            asin: None, // iTunes doesn't provide ASIN
            description,
            cover_url,
            duration,
            narrator: None, // iTunes search results don't include narrator
            publisher: None,
            publish_year,
            series,
            series_number,
            genre,
            tags: None,
            language,
            explicit: None, // iTunes has contentAdvisoryRating but it's not in search results
            abridged: None,
        })
    }
    
    // Search FantLab.ru API
    async fn search_fantlab(query: &str) -> Result<Vec<BookMetadata>, String> {
        // FantLab.ru doesn't have a public API, so this would require web scraping
        // For now, return an error indicating it's not implemented
        Err("FantLab.ru search is not yet implemented. FantLab.ru does not provide a public API.".to_string())
    }
}

// Add a helper function to get metadata from a file
pub async fn get_file_metadata(file_path: &str) -> Result<BookMetadata, String> {
    // This would extract metadata from audio files
    // For now, return mock data
    Ok(BookMetadata {
        title: format!("Audio File: {}", file_path),
        subtitle: None,
        author: "Unknown Author".to_string(),
        isbn: None,
        asin: None,
        description: None,
        cover_url: None,
        duration: Some("00:00:00".to_string()),
        narrator: None,
        publisher: None,
        publish_year: None,
        series: None,
        series_number: None,
        genre: None,
        tags: None,
        language: None,
        explicit: None,
        abridged: None,
    })
}