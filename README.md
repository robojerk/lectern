# Lectern - Audiobook Preparation Tool

An Iced-based desktop application for preparing audiobooks for Audiobookshelf servers. Lectern automates the conversion of MP3 directories into properly tagged M4B files with direct upload to your Audiobookshelf server.

## Features

✅ **Drag-and-Drop Interface** - Simply drag your audiobook folder into the app
✅ **Automatic Metadata Fetching** - Queries Audnexus API for book information  
✅ **Editable Metadata** - Review and modify title, author, series, and narrator  
✅ **Chapter Management** - Create, edit, and manage chapter markers with playback preview  
✅ **Cover Art Search** - Search and download cover images from multiple sources  
✅ **M4B Conversion** - High-quality AAC encoding with FFmpeg  
✅ **Metadata Tagging** - Embeds metadata during FFmpeg conversion  
✅ **Direct Upload** - Uploads to Audiobookshelf and triggers library scan  
✅ **Real-Time Logging** - See exactly what's happening during processing  
✅ **Progress Tracking** - Visual feedback throughout the conversion pipeline

## Project Structure

```
lectern/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── main.rs            # Application entry point
│   ├── models/            # Data models (chapters, metadata)
│   ├── services/          # Business logic (conversion, playback, ffprobe)
│   ├── ui/                # UI layer
│   │   ├── handlers/      # Message handlers for each feature
│   │   ├── views/         # UI view components
│   │   ├── state/         # Application state management
│   │   ├── colors.rs      # Color theme definitions
│   │   └── helpers.rs     # UI helper functions
│   └── utils/             # Utility functions (formatting, time)
├── assets/                # Icons and images
│   └── png/               # Material Symbols icons
├── .gitignore             # Git ignore rules
└── README.md              # This file
```

## System Requirements

### Rust

Lectern requires Rust 1.70+ (2021 edition). Install Rust from [rustup.rs](https://rustup.rs/).

### FFmpeg

FFmpeg is required for audio processing and M4B conversion:

#### Fedora/RHEL
```bash
sudo dnf install ffmpeg
```

#### Ubuntu/Debian
```bash
sudo apt install ffmpeg
```

#### Arch Linux
```bash
sudo pacman -S ffmpeg
```

#### macOS (Homebrew)
```bash
brew install ffmpeg
```

## Building

Once Rust and FFmpeg are installed:

```bash
cargo build --release
```

The application is written in pure Rust with no external GUI framework dependencies.

## Running

```bash
cargo run
```

Or use the provided script:

```bash
./run_lectern.sh
```

For release builds:

```bash
cargo run --release
```

### Installing System-wide

After testing, you can install for system-wide access:

```bash
sudo cp target/release/lectern /usr/local/bin/
```

### AppImage (Portable)

An AppImage is available for easy distribution. See [APPDIRECTORY.md](APPDIRECTORY.md) for build instructions.

To use a pre-built AppImage:
1. Download the `.AppImage` file
2. Make it executable: `chmod +x Lectern-*.AppImage`
3. Run it: `./Lectern-*.AppImage`

The AppImage bundles all dependencies including FFmpeg, so no system installation is required.

## How It Works

1. **Configure Settings** - Set up your Audiobookshelf server URL, API token, and library ID
2. **Select Files** - Browse or drag-and-drop a folder containing MP3 files
3. **Search Metadata** - Search for book metadata using title, author, or ASIN
4. **Review Metadata** - The app fetches metadata and displays it for review
5. **Edit if Needed** - Modify any metadata fields (title, author, series, narrator, etc.)
6. **Manage Chapters** - Create, edit, lock/unlock, and preview chapter markers
7. **Select Cover Art** - Search and download cover images or browse local files
8. **Convert & Upload** - Click the convert button and watch the magic happen:
   - Analyzes all MP3 files and extracts durations
   - Generates chapter markers based on file structure
   - Converts to M4B with FFmpeg
   - Embeds metadata and cover art
   - Uploads to your Audiobookshelf server
   - Triggers automatic library scan

## Tech Stack

- **Language**: Rust (2021 edition)
- **GUI Framework**: Iced 0.12 (pure Rust, cross-platform)
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **Serialization**: serde, serde_json
- **Audio Processing**: FFmpeg (external process)
- **File Dialogs**: rfd (Rust File Dialogs)
- **Image Processing**: image crate

## Configuration

### Audiobookshelf Setup

1. Open your Audiobookshelf web interface
2. Go to **Settings** → **Users** → Your User
3. Generate an **API Token** and copy it
4. Go to **Settings** → **Libraries** and copy your **Library ID** from the URL or library settings

### In Lectern

1. Click the **Settings** button in the navigation
2. Enter:
   - **Server URL**: `https://your-abs-server.com`
   - **API Token**: Your generated token
   - **Library ID**: Your library UUID
3. Click **Save**

## Usage Tips

- **File Naming**: MP3 files should be named in the order you want them (e.g., `01-chapter1.mp3`, `02-chapter2.mp3`)
- **Metadata Override**: If Audible API doesn't find your book, you can manually enter all metadata
- **Chapter Management**: Use the chapter editor to fine-tune chapter markers, lock important chapters, and preview playback
- **Cover Art**: Search for covers by title/author or provide a direct URL
- **Large Files**: Be patient with large audiobooks - conversion can take several minutes
- **Network**: Ensure you have a stable connection to your Audiobookshelf server for uploads

## Troubleshooting

### "FFmpeg failed" error
- Ensure `ffmpeg` and `ffprobe` are installed: `sudo dnf install ffmpeg` (or equivalent for your distro)
- Verify FFmpeg is in your PATH: `which ffmpeg`

### "Upload failed" error
- Verify your ABS server URL is correct (include `https://`)
- Check that your API token hasn't expired
- Ensure the library ID is correct

### No metadata found
- The Audible API might not have your book - use manual entry
- Check your internet connection
- Try searching by ASIN if you have it

### MP3 files not detected
- Ensure files have `.mp3` extension (case-insensitive)
- Check that you're selecting a folder, not individual files

### Build errors
- Ensure you have Rust 1.70+ installed: `rustc --version`
- Update Rust toolchain: `rustup update`
- Clean and rebuild: `cargo clean && cargo build`

## Development

### Project Architecture

The application follows a modular architecture:

- **UI Layer** (`src/ui/`): Iced-based interface with message-driven architecture
  - **Views** (`ui/views/`): Screen components (search, metadata, chapters, cover, convert, settings)
  - **Handlers** (`ui/handlers/`): Message handlers for each feature area
  - **State** (`ui/state/`): Application state management
- **Service Layer** (`src/services/`): Business logic for audio processing and API calls
  - **conversion.rs**: M4B conversion and FFmpeg operations
  - **ffprobe.rs**: Audio file analysis
  - **playback.rs**: Chapter playback preview
- **Models** (`src/models/`): Data structures (BookMetadata, Chapter)
- **Utils** (`src/utils/`): Helper functions for formatting and time conversion

### Key Technologies

- **Iced**: Pure Rust GUI framework with immediate mode rendering
- **Tokio**: Async runtime for concurrent operations
- **reqwest**: HTTP client for API calls
- **FFmpeg**: External process for audio conversion and metadata embedding
- **serde**: JSON serialization/deserialization

### Development Workflow

1. Make changes to the code
2. Test with `cargo run`
3. Build release version: `cargo build --release`
4. Run linter: `cargo clippy`
5. Format code: `cargo fmt`

## License

MIT License - Feel free to use and modify as needed.

### Third-Party Licenses

**Material Symbols Icons** (in `assets/png/`):
- Licensed under Apache License 2.0
- Copyright 2024 Google LLC
- Source: https://fonts.google.com/icons
- See `assets/png/license` for full license text

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
