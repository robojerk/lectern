pub mod search;
pub mod metadata;
pub mod cover;
pub mod chapters;
pub mod file;
pub mod settings;
pub mod convert;
pub mod navigation;

pub use search::handle_search;
pub use metadata::handle_metadata;
pub use cover::handle_cover;
pub use chapters::handle_chapters;
pub use file::handle_file;
pub use settings::handle_settings;
pub use convert::handle_convert;
pub use navigation::handle_navigation;
