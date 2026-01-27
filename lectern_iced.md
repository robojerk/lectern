# Prompt for Creating Lectern Audiobook Manager in Rust/Iced

I need help creating a desktop audiobook management application called "Lectern" using Rust and the Iced GUI framework. The application should allow users to import audiobooks (as directories of audio files or single M4B files), edit metadata, manage chapters, and export everything as a single M4B file.

## Core Requirements

### Main Window Structure
- Tabbed interface with the following tabs:
  - **Details** - Metadata editing
  - **Cover** - Cover art management
  - **Chapters** - Chapter editing and mapping
  - **Files** - Source file management
  - **Convert** - Conversion settings and execution
  - **Settings** - Application configuration

### Details Tab (Image 1 reference)
Create a metadata editing form with these fields:
- Title and Subtitle (side by side)
- Authors and Publish Year (side by side)
- Series with order number (e.g., "Dune #04") with a + button to add
- Description (rich text editor with formatting buttons: Bold, Italic, Strikethrough, Link, Bullet list, Numbered list, Undo/Redo)
- Genres and Tags (side by side)
- Narrators, ISBN, and ASIN fields
- Publisher and Language (side by side)
- Explicit and Abridged checkboxes
- Bottom action buttons: "Quick Match", "Re-Scan", "Save", "Save & Close"

### Chapters Tab (Image 2 reference)
Create a chapter management interface with:
- Top controls: "Remove All", "Shift Times", "Lookup" buttons
- "Show seconds" checkbox in top right
- Chapter list with columns:
  - Chapter number (#1, #2, etc.)
  - START time with +/- buttons (format: HH:MM:SS)
  - TITLE field (editable text)
  - Action icons: Lock, Delete, Insert Below, Play
- Global lock icon in header to lock all chapters


## Technical Implementation Needs

### File Handling
- Drag-and-drop support for directories and files
- File picker dialog integration (XDG portal)
- Support for MP3, AAC, WAV, FLAC, M4B formats

### Data Structures
```rust
struct Audiobook {
    title: String,
    subtitle: Option<String>,
    authors: Vec<String>,
    series: Option<String>,
    series_order: Option<f32>,
    publish_year: Option<u32>,
    description: String,
    genres: Vec<String>,
    tags: Vec<String>,
    narrators: Vec<String>,
    isbn: Option<String>,
    asin: Option<String>,
    publisher: Option<String>,
    language: Option<String>,
    explicit: bool,
    abridged: bool,
    cover: Option<PathBuf>,
    chapters: Vec<Chapter>,
}

struct Chapter {
    number: usize,
    start_time: Duration,
    title: String,
    locked: bool,
}
```

### UI Styling
- Dark theme with dark gray background (#2a2a2a)
- Input fields with slightly lighter gray background (#3a3a3a)
- White text
- Rounded corners on buttons and inputs
- Blue accent color for active buttons and confidence badges
- Green accent for high confidence matches

### Key Features to Implement
1. Tab navigation system
2. Text input fields with proper validation
3. Rich text editor for description
4. Time picker/editor for chapter timestamps
5. Chapter manipulation (add, remove, shift times, lock/unlock)
6. Search results display with confidence scoring
7. Settings persistence
8. M4B file generation with embedded metadata

Please provide the Rust/Iced code structure to get started with this application, including the main application state, tab system, and the Details tab implementation. Focus on creating a clean, modular architecture that will be easy to extend.