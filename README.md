# Lectern - Audiobook Preparation Tool

A GTK4-based desktop application for preparing audiobooks for Audiobookshelf servers. Lectern automates the conversion of MP3 directories into properly tagged M4B files with automatic chapter generation and direct upload to your Audiobookshelf server.

## Features

✅ **Drag-and-Drop Interface** - Simply drag your audiobook folder into the app  
✅ **Automatic Metadata Fetching** - Queries Audible API for book information  
✅ **Editable Metadata** - Review and modify title, author, series, and narrator  
✅ **Chapter Generation** - Automatically creates chapters from individual MP3 files  
✅ **M4B Conversion** - High-quality AAC encoding with chapter markers  
✅ **Metadata Tagging** - Embeds all metadata directly into the M4B file  
✅ **Direct Upload** - Uploads to Audiobookshelf and triggers library scan  
✅ **Real-Time Logging** - See exactly what's happening during processing  
✅ **Progress Tracking** - Visual feedback throughout the conversion pipeline  

## Project Structure

```
lectern/
├── Cargo.toml              # Rust dependencies
├── src/
│   ├── main.rs            # Main application and UI code
│   └── services.rs        # Audio processing and ABS API services
├── lectern.desktop        # Desktop entry file
├── .gitignore             # Git ignore rules
├── README.md              # This file
└── *.md                   # Documentation files
```

## System Requirements

### Qt6 Development Libraries

Before building, you need to install Qt6 and related development libraries:

#### Fedora/RHEL (System Package Manager)
```bash
sudo dnf install qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtquickcontrols2-devel ffmpeg
```

#### Ubuntu/Debian
```bash
sudo apt install qt6-base-dev qt6-declarative-dev qt6-quickcontrols2-dev ffmpeg
```

#### Arch Linux
```bash
sudo pacman -S qt6-base qt6-declarative qt6-quickcontrols2 ffmpeg
```

#### Homebrew (Alternative)
```bash
brew install qt@6 ffmpeg
```
**Note**: Homebrew Qt may require `PKG_CONFIG_PATH` environment variables.

### Additional Tools

- **ffmpeg** - For audio processing and M4B creation
- **m4b-tool** (optional) - Enhanced M4B processing capabilities

## Building

Once the system dependencies are installed:

```bash
cargo build --release
```

The build system automatically detects Qt6 libraries from your system package manager.

## Running

```bash
cargo run --release
```

**That's it!** No complex environment variables or wrapper scripts needed when using system Qt6.

### Installing System-wide

After testing, you can install for system-wide access:

```bash
sudo cp target/release/lectern /usr/local/bin/
sudo cp lectern.desktop /usr/share/applications/
```

## How It Works

1. **Configure Settings** - Set up your Audiobookshelf server URL, API token, and library ID
2. **Drag Folder** - Drop a folder containing MP3 files into the app
3. **Review Metadata** - The app fetches metadata from Audible and displays it for review
4. **Edit if Needed** - Modify any metadata fields (title, author, series, narrator)
5. **Convert & Upload** - Click the button and watch the magic happen:
   - Analyzes all MP3 files and extracts durations
   - Generates chapter markers based on filenames
   - Converts to M4B with FFmpeg
   - Embeds metadata and cover art
   - Uploads to your Audiobookshelf server
   - Triggers automatic library scan

## Tech Stack

- **Language**: Rust
- **GUI**: GTK4 (no libadwaita)
- **Async Runtime**: Tokio
- **HTTP Client**: reqwest
- **Audio Tagging**: audiotags
- **Audio Processing**: ffmpeg (external)

## Configuration

### Audiobookshelf Setup

1. Open your Audiobookshelf web interface
2. Go to **Settings** → **Users** → Your User
3. Generate an **API Token** and copy it
4. Go to **Settings** → **Libraries** and copy your **Library ID** from the URL or library settings

### In Lectern

1. Click the **⚙ Settings** button in the header
2. Enter:
   - **Server URL**: `https://your-abs-server.com`
   - **API Token**: Your generated token
   - **Library ID**: Your library UUID
3. Click **Save**

## Usage Tips

- **File Naming**: MP3 files should be named in the order you want them (e.g., `01-chapter1.mp3`, `02-chapter2.mp3`)
- **Metadata Override**: If Audible API doesn't find your book, you can manually enter all metadata
- **Large Files**: Be patient with large audiobooks - conversion can take several minutes
- **Network**: Ensure you have a stable connection to your Audiobookshelf server for uploads

## Troubleshooting

### "FFmpeg failed" error
- Ensure `ffmpeg` and `ffprobe` are installed: `sudo dnf install ffmpeg`

### "Upload failed" error
- Verify your ABS server URL is correct (include `https://`)
- Check that your API token hasn't expired
- Ensure the library ID is correct

### No metadata found
- The Audible API might not have your book - use manual entry
- Check your internet connection

### MP3 files not detected
- Ensure files have `.mp3` extension (case-insensitive)
- Check that you're dragging a folder, not individual files

## Development

### Project Architecture

- **UI Layer** (`main.rs`): GTK4 interface with async event handling via `glib::MainContext::channel`
- **Service Layer** (`services.rs`): All business logic for audio processing and API calls
- **Async Runtime**: Tokio for non-blocking I/O operations
- **Communication**: Message passing between UI thread and worker threads

### Key Technologies

- **GTK4**: Native Linux UI (no libadwaita)
- **Tokio**: Async runtime for concurrent operations
- **reqwest**: HTTP client for API calls
- **audiotags**: Audio metadata manipulation
- **FFmpeg**: External process for audio conversion
- **serde**: JSON serialization/deserialization

## License

MIT License - Feel free to use and modify as needed.

## Contributing

Contributions are welcome! Please feel free to submit issues or pull requests.
