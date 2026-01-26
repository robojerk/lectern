# Lectern Iced Implementation Plan

Based on `lectern_iced.md` specification and current implementation status.

## Current Implementation vs Spec

### ✅ Implemented (Matches Spec)
- Tab navigation system
- Basic metadata editing (all fields from spec are present)
- File selection (drag & drop + file picker)
- Cover tab with search and upload
- Search functionality (Match tab equivalent)

### 🔄 Needs Enhancement
- **Details Tab**: Currently basic layout, spec wants side-by-side fields and rich text editor
- **Chapters Tab**: Currently placeholder, spec wants full chapter management
- **Convert Tab**: Currently placeholder, spec wants conversion settings
- **Match Tab**: Currently integrated into search, spec wants separate tab with confidence scoring

### ❌ Missing from Spec
- **Files Tab**: Source file management (not in current implementation)
- **Settings Tab**: Application configuration (not in current implementation)
- Rich text editor for description
- Chapter time picker/editor
- Confidence scoring in search results
- Series management with + button

## Next Implementation Steps

### Priority 1: Enhance Details Tab Layout
- Reorganize fields to match spec (side-by-side layout)
- Add rich text editor for description (or markdown support)
- Add action buttons: "Quick Match", "Re-Scan", "Save", "Save & Close"

### Priority 2: Implement Chapters Tab
- Chapter list with columns (#, START time, TITLE, actions)
- Time editing with +/- buttons
- Lock/unlock functionality
- Global time shift
- Insert/Delete chapters
- Play button (requires audio playback)

### Priority 3: Implement Convert Tab
- Local Library path setting
- Path template editor with preview
- Conversion settings
- Progress display
- Execute conversion

### Priority 4: Add Missing Tabs
- Files tab (source file management)
- Settings tab (application configuration)
- Separate Match tab (if needed)

## Technical Notes

- Iced 0.12 doesn't have built-in rich text editor - may need custom widget or markdown
- Audio playback requires additional crate (rodio/symphonia)
- Time picker needs custom widget or text input with validation
- FFmpeg integration for M4B conversion
