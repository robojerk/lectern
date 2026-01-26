# Lectern Implementation Status

## ✅ Implemented

### Core Infrastructure
- ✅ Iced GUI framework (migrated from QML)
- ✅ Async search functionality
- ✅ Tab navigation (Metadata, Cover, Chapters, Convert, Search)

### Metadata
- ✅ Search by title/author or ASIN
- ✅ Search providers: Open Library, Google Books, Audnexus
- ✅ Display search results
- ✅ Select book to populate metadata
- ✅ Edit metadata fields:
  - ✅ Title
  - ✅ Author
  - ✅ Series
  - ✅ Narrator
  - ✅ Description

## ❌ Missing Features

### File Selection
- ❌ Drag & drop audiobook folder/file
- ❌ XDG portal file chooser button
- ❌ Support for MP3 directories, M4B files, and other formats (AAC, WAV, FLAC)

### Metadata (Incomplete)
- ❌ Subtitle field
- ❌ Genre field
- ❌ Tags field
- ❌ ISBN field (display/edit)
- ❌ Publisher field
- ❌ Language field
- ❌ Explicit (yes/no) toggle
- ❌ Abridged (yes/no) toggle
- ❌ Publish Year field (in UI - exists in struct)

### Search Providers
- ❌ Audible.com
- ❌ iTunes
- ❌ FantLab.ru
- ❌ Audible.ca

### Cover Tab
- ❌ Display existing cover from file/dir
- ❌ Manual cover upload
- ❌ Cover search from providers
- ❌ Cover preview/editing

### Chapters Tab
- ❌ Chapter list display
- ❌ Get chapters from provider (Audible)
- ❌ Auto-map chapters from filenames
- ❌ Chapter editing (add, edit, remove)
- ❌ Chapter locking
- ❌ Global time shift
- ❌ Individual chapter time adjustment
- ❌ Chapter playback

### Convert Tab
- ❌ Local Library path setting
- ❌ Path template configuration
- ❌ Auto-populate save location
- ❌ XDG portal file chooser for save location
- ❌ M4B conversion with FFmpeg
- ❌ Embed metadata in M4B
- ❌ Embed cover in M4B
- ❌ Embed chapters in M4B

### Settings
- ❌ Settings dialog/window
- ❌ Local Library path configuration
- ❌ Path template editor with preview
- ❌ Audiobookshelf settings:
  - ❌ Host URL
  - ❌ API token
  - ❌ Library ID
- ❌ Auto-upload to Audiobookshelf after conversion

## 📋 Priority Recommendations

### Phase 1: Complete Metadata (High Priority)
1. Add missing metadata fields to UI
2. Update `BookMetadata` struct if needed
3. Ensure all fields save properly

### Phase 2: File Selection (High Priority)
1. Implement drag & drop
2. Add file chooser button
3. Parse selected files (MP3 dir or M4B)

### Phase 3: Cover Tab (Medium Priority)
1. Display cover from file
2. Manual upload
3. Cover search integration

### Phase 4: Convert Tab (High Priority)
1. M4B conversion with FFmpeg
2. Metadata embedding
3. Save location handling

### Phase 5: Chapters Tab (Complex - Medium Priority)
1. Basic chapter list
2. Provider lookup
3. Editing interface

### Phase 6: Settings (Medium Priority)
1. Settings UI
2. Path template system
3. Audiobookshelf integration

## Notes

- The current `BookMetadata` struct in `services.rs` has most fields but some are `Option<String>` that aren't exposed in the UI
- The Iced architecture makes it easy to add new messages and state
- All async operations are properly handled with `Command::perform`
