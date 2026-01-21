use crate::models::chapter::ChapterObject;
use crate::models::metadata::BookMetadata;
use std::path::PathBuf;

#[derive(Default, Clone)]
pub struct Project {
    pub files: Vec<PathBuf>,
    pub file_durations: Vec<u64>,
    pub metadata: BookMetadata,
    pub chapters: Vec<ChapterObject>,
}
