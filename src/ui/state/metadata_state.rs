use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetadataProvider {
    Auto,
    AudibleCom,
    AudibleCa,
    Audnexus,
    GoogleBooks,
    ITunes,
    OpenLibrary,
    FantLab,
}

impl MetadataProvider {
    pub const ALL: [MetadataProvider; 8] = [
        MetadataProvider::Auto,
        MetadataProvider::AudibleCom,
        MetadataProvider::AudibleCa,
        MetadataProvider::Audnexus,
        MetadataProvider::GoogleBooks,
        MetadataProvider::ITunes,
        MetadataProvider::OpenLibrary,
        MetadataProvider::FantLab,
    ];

    pub fn to_id(&self) -> String {
        match self {
            MetadataProvider::Auto => "auto".to_string(),
            MetadataProvider::AudibleCom => "audible_com".to_string(),
            MetadataProvider::AudibleCa => "audible_ca".to_string(),
            MetadataProvider::Audnexus => "audnexus".to_string(),
            MetadataProvider::GoogleBooks => "google_books".to_string(),
            MetadataProvider::ITunes => "itunes".to_string(),
            MetadataProvider::OpenLibrary => "open_library".to_string(),
            MetadataProvider::FantLab => "fantlab".to_string(),
        }
    }

    #[allow(dead_code)]
    pub fn from_id(id: &str) -> Self {
        match id {
            "audible_com" => MetadataProvider::AudibleCom,
            "audible_ca" => MetadataProvider::AudibleCa,
            "audnexus" => MetadataProvider::Audnexus,
            "google_books" => MetadataProvider::GoogleBooks,
            "itunes" => MetadataProvider::ITunes,
            "open_library" => MetadataProvider::OpenLibrary,
            "fantlab" => MetadataProvider::FantLab,
            _ => MetadataProvider::Auto,
        }
    }
}

impl fmt::Display for MetadataProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MetadataProvider::Auto => write!(f, "Auto"),
            MetadataProvider::AudibleCom => write!(f, "Audible.com"),
            MetadataProvider::AudibleCa => write!(f, "Audible.ca"),
            MetadataProvider::Audnexus => write!(f, "Audnexus"),
            MetadataProvider::GoogleBooks => write!(f, "Google Books"),
            MetadataProvider::ITunes => write!(f, "iTunes"),
            MetadataProvider::OpenLibrary => write!(f, "Open Library"),
            MetadataProvider::FantLab => write!(f, "FantLab.ru"),
        }
    }
}

#[derive(Debug)]
pub struct MetadataState {
    pub selected_book: Option<crate::models::BookMetadata>,
    
    // Editing fields (for metadata tab)
    pub editing_title: String,
    pub editing_subtitle: String,
    pub editing_author: String,
    pub editing_series: String,
    pub editing_series_number: String,
    pub editing_narrator: String,
    pub editing_description: String,
    pub editing_description_content: iced::widget::text_editor::Content,
    pub editing_isbn: String,
    pub editing_asin: String,
    pub editing_publisher: String,
    pub editing_publish_year: String,
    pub editing_genre: String,
    pub editing_tags: String,
    pub editing_language: String,
    pub editing_explicit: bool,
    pub editing_abridged: bool,
    
    // Provider selection
    pub metadata_provider: MetadataProvider,
}

impl Default for MetadataState {
    fn default() -> Self {
        Self {
            selected_book: None,
            editing_title: String::new(),
            editing_subtitle: String::new(),
            editing_author: String::new(),
            editing_series: String::new(),
            editing_series_number: String::new(),
            editing_narrator: String::new(),
            editing_description: String::new(),
            editing_description_content: iced::widget::text_editor::Content::new(),
            editing_isbn: String::new(),
            editing_asin: String::new(),
            editing_publisher: String::new(),
            editing_publish_year: String::new(),
            editing_genre: String::new(),
            editing_tags: String::new(),
            editing_language: String::new(),
            editing_explicit: false,
            editing_abridged: false,
            metadata_provider: MetadataProvider::Auto,
        }
    }
}
