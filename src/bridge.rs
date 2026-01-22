use cxx_qt_lib::QString;

// Bridge between Rust backend and Qt frontend
#[cxx::bridge]
mod ffi {
    // Shared types
    #[derive(Clone, Debug)]
    pub struct BookMetadata {
        pub title: String,
        pub authors: Vec<String>,
        pub narrator_names: Vec<String>,
        pub series_name: String,
        pub cover_url: String,
        pub asin: String,
        pub duration_minutes: u64,
        pub release_date: String,
    }

    #[derive(Clone, Debug)]
    pub struct Chapter {
        pub title: String,
        pub start_time: f64,
        pub locked: bool,
    }

    #[derive(Clone, Debug)]
    pub struct SearchResult {
        pub metadata: BookMetadata,
        pub confidence: f64,
    }

    // Qt object that exposes Rust functionality to QML
    unsafe extern "C++" {
        include!("cxx-qt-lib/include/qt.h");
        include!("cxx-qt-lib/include/qurl.h");

        type QString = cxx_qt_lib::QString;
    }

    #[cxx_qt::bridge]
    pub struct LecternController {
        // Properties exposed to QML
        #[qproperty]
        pub current_folder: QString,

        #[qproperty]
        pub status_message: QString,

        #[qproperty]
        pub progress_value: f64,

        #[qproperty]
        pub is_processing: bool,

        // Metadata properties
        #[qproperty]
        pub metadata_title: QString,

        #[qproperty]
        pub metadata_author: QString,

        #[qproperty]
        pub metadata_series: QString,

        #[qproperty]
        pub metadata_narrator: QString,

        // ABS settings
        #[qproperty]
        pub abs_host: QString,

        #[qproperty]
        pub abs_token: QString,

        #[qproperty]
        pub abs_library_id: QString,
    }

    impl cxx_qt::Constructor<()> for LecternController {}
    impl cxx_qt::Threading for LecternController {}

    impl LecternController {
        // Signals
        #[qsignal]
        pub fn folder_dropped(self: Pin<&mut Self>, url: QString);

        #[qsignal]
        pub fn metadata_loaded(self: Pin<&mut Self>);

        #[qsignal]
        pub fn search_results_updated(self: Pin<&mut Self>);

        #[qsignal]
        pub fn log_message(self: Pin<&mut Self>, message: QString);

        #[qsignal]
        pub fn conversion_completed(self: Pin<&mut Self>);

        #[qsignal]
        pub fn error_occurred(self: Pin<&mut Self>, message: QString);

        // Invokable methods from QML
        #[qinvokable]
        pub fn load_config(self: Pin<&mut Self>);

        #[qinvokable]
        pub fn save_config(self: Pin<&mut Self>);

        #[qinvokable]
        pub fn set_folder_path(self: Pin<&mut Self>, path: QString);

        #[qinvokable]
        pub fn search_metadata(self: Pin<&mut Self>, query: QString, by_asin: bool);

        #[qinvokable]
        pub fn use_search_result(self: Pin<&mut Self>, index: usize);

        #[qinvokable]
        pub fn start_conversion(self: Pin<&mut Self>);

        #[qinvokable]
        pub fn cancel_conversion(self: Pin<&mut Self>);

        // Chapter management
        #[qinvokable]
        pub fn add_chapter(self: Pin<&mut Self>, title: QString, start_time: f64);

        #[qinvokable]
        pub fn remove_chapter(self: Pin<&mut Self>, index: usize);

        #[qinvokable]
        pub fn update_chapter(self: Pin<&mut Self>, index: usize, title: QString, start_time: f64);

        #[qinvokable]
        pub fn lock_chapter(self: Pin<&mut Self>, index: usize, locked: bool);

        #[qinvokable]
        pub fn shift_chapters(self: Pin<&mut Self>, offset: f64);

        // Playback (for chapter preview)
        #[qinvokable]
        pub fn play_chapter(self: Pin<&mut Self>, index: usize);

        #[qinvokable]
        pub fn pause_playback(self: Pin<&mut Self>);

        #[qinvokable]
        pub fn stop_playback(self: Pin<&mut Self>);
    }
}

// Re-export for use in main.rs
pub use ffi::{BookMetadata, Chapter, LecternController, SearchResult};

/// Register Rust types with QML engine
pub fn register_types(engine: &mut cxx_qt_lib::QQmlApplicationEngine) {
    // Register LecternController as a QML type
    cxx_qt_lib::qml_register_type::<LecternController>(
        engine,
        "Lectern",
        1,
        0,
        "LecternController"
    );
}