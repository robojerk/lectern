use crate::models::metadata::BookMetadata;

#[derive(Clone, Debug)]
pub enum AppEvent {
    Status(String),
    Log(String),
    Error(String),
    Complete,
    FolderLoaded(std::path::PathBuf),
    FileDurationsLoaded(Vec<u64>),
    ChaptersLoaded(Vec<crate::services::SimpleChapter>),
    CoverImageLoaded(Vec<u8>),
    SearchResultsLoaded(Vec<BookMetadata>),
    PlayRequested(u64),
    StopRequested,
}
