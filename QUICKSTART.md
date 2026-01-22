# Lectern Quick Start Guide

## 🚀 Build & Run (5 minutes)

### 1. Install Dependencies

Using your system package manager:
```bash
# Fedora/RHEL
sudo dnf install qt6-qtbase-devel qt6-qtdeclarative-devel qt6-qtquickcontrols2-devel ffmpeg

# Ubuntu/Debian
sudo apt install qt6-base-dev qt6-declarative-dev qt6-quickcontrols2-dev ffmpeg

# Arch Linux
sudo pacman -S qt6-base qt6-declarative qt6-quickcontrols2 ffmpeg
```

### 2. Build & Run

```bash
cd /home/rob/Documents/Projects/lectern
cargo run --release
```

That's it! Uses system Qt6 automatically.

## 📖 First Use

### Configure Audiobookshelf

1. Click **⚙ Settings** in the header
2. Enter your server details:
   - **Server URL**: `https://your-abs-server.com`
   - **API Token**: (from ABS Settings → Users → API Token)
   - **Library ID**: (from ABS Libraries page URL)
3. Click **Save**

### Convert Your First Audiobook

1. **Drag** a folder containing MP3 files into the app
2. **Wait** for metadata to load from Audible
3. **Review/Edit** the metadata (title, author, series, narrator)
4. Click **🚀 Convert & Upload to ABS**
5. **Watch** the real-time log for progress

## 📋 What You Need

### Audiobook Folder Structure

```
My_Audiobook/
├── 01_Chapter_One.mp3
├── 02_Chapter_Two.mp3
├── 03_Chapter_Three.mp3
└── ...
```

**Important**: 
- Files should be named in order (01, 02, 03, etc.)
- All files must be MP3 format
- Filenames become chapter titles

### Audiobookshelf Server

- Running ABS instance (v2.0+)
- API access enabled
- Valid API token
- Target library UUID

## ⚡ Features at a Glance

- ✅ Automatic metadata from Audible
- ✅ Editable fields for corrections
- ✅ Chapter generation from file durations
- ✅ High-quality M4B with AAC 128kbps
- ✅ Embedded metadata and chapters
- ✅ Direct upload and library scan
- ✅ Real-time FFmpeg output
- ✅ Progress tracking

## 🔧 Troubleshooting

### Build fails with GTK4 not found

The project includes `.cargo/config.toml` which auto-configures Homebrew paths. If you still have issues:

```bash
brew install gtk4 ffmpeg
cargo clean
cargo build --release
```

### Application won't start

```bash
# Just use cargo run - it handles everything
cargo run --release
```

### FFmpeg not found error

```bash
brew install ffmpeg
# OR
sudo dnf install ffmpeg
```

### Upload fails

- Verify server URL includes `https://` or `http://`
- Check API token hasn't expired
- Confirm library ID is correct
- Ensure network connectivity to ABS server

## 📚 Example Workflow

1. **Start**: Launch Lectern
2. **Configure**: One-time ABS setup
3. **Drag folder**: "The_Martian_by_Andy_Weir"
4. **Auto-fetch**: Gets metadata from Audible
5. **Review**: Author = "Andy Weir", Narrator = "R.C. Bray"
6. **Convert**: Click button, watch progress
7. **Result**: "The_Martian_by_Andy_Weir.m4b" in ABS!

## 🎯 Tips

- **Large books** (10+ hours): Be patient, encoding takes time
- **Network uploads**: Ensure stable connection for large files
- **Metadata quality**: Audible API is good but verify results
- **File naming**: Use descriptive chapter names in filenames
- **Test first**: Try with a small audiobook to verify setup

## 📞 Need Help?

Check the logs in the application window - they show detailed error messages and FFmpeg output to help diagnose issues.

---

**Ready to go!** Drop a folder and start converting! 🎉
