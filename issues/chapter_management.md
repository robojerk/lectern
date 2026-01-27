# Chapter Management Architecture

## Overview
This document outlines how chapters are managed in Lectern, incorporating insights from Audiobookshelf's architecture while maintaining our own implementation.

**Status**: Basic chapter functionality exists, but needs enhancements based on ABS patterns.

## Key Insights from Audiobookshelf

### 1. Virtual vs Embedded Chapters
- **Virtual Chapters**: Stored in application state/database, not in audio file
- **Embedded Chapters**: Written into the audio file (M4B) metadata
- **Design Choice**: ABS keeps chapters virtual until user explicitly embeds them
- **Benefit**: Allows editing without modifying source files

### 2. Chapter Sources
- **Extract from File**: Use `ffprobe` to read embedded chapters
- **Generate from Files**: One chapter per file for multi-file books
- **Lookup from Provider**: Fetch chapters from Audible/Audnexus with timestamp shifting
- **Manual Entry**: User creates chapters manually

### 3. Timestamp Shifting
- **Problem**: External provider chapters may not align with local file
- **Solution**: Offset all timestamps by calculated amount
- **Use Case**: Remove "This is Audible" intro (~2-3 seconds)
- **Duration Matching**: Compare total duration, warn if mismatch

### 4. Shift Operations
- **Ripple Effect**: Moving one chapter shifts subsequent chapters
- **Gap Prevention**: Ensure no gaps or overlaps in timeline
- **Locked Chapters**: Some chapters can be "locked" to prevent shifting

## Our Architecture

### Current Implementation Status

#### ✅ Already Implemented
- Basic Chapter struct with title, start_time, duration, is_locked
- Chapter lookup from Audnexus API
- Map chapters from files (one per file, but uses placeholder durations)
- Basic chapter editing UI (edit title, adjust time, delete, lock)
- Chapter list display with time formatting

#### ❌ Missing Features
- Extract chapters from file using ffprobe
- Accurate duration calculation using ffprobe (currently uses placeholder)
- Timestamp shifting (global and individual)
- Chapter validation (gaps, overlaps, duration checks)
- Chapter source tracking (where chapters came from)
- Ripple effect when shifting individual chapters

### Data Model

#### Chapter Structure (Current)
```rust
pub struct Chapter {
    pub title: String,
    pub start_time: u64,  // milliseconds
    pub duration: u64,    // milliseconds
    pub is_locked: bool,
}
```

#### Proposed Enhancements
```rust
// Add to Chapter struct (optional, backward compatible)
pub struct Chapter {
    pub title: String,
    pub start_time: u64,  // milliseconds
    pub duration: u64,    // milliseconds
    pub is_locked: bool,
    pub source: Option<ChapterSource>, // Track where chapter came from
}

pub enum ChapterSource {
    Embedded,      // Extracted from file
    Generated,     // Auto-generated from files
    Lookup,        // From external provider (Audnexus)
    Manual,        // User-created
}
```

#### Chapter State Management (Current)
Currently chapters are stored directly in `Lectern.chapters: Vec<Chapter>`.

#### Proposed Enhancements
Could add to `Lectern` struct:
```rust
pub struct ChapterState {
    pub chapters: Vec<Chapter>,
    pub total_duration_ms: Option<u64>,  // Total file duration
    pub is_dirty: bool,  // Has been edited but not saved/embedded
    pub chapter_source: Option<ChapterSource>, // Overall source (if all from same source)
}
```

### Chapter Operations

#### 1. Extract from File
- **Function**: `extract_chapters_from_file(path: &str) -> Result<Vec<Chapter>>`
- **Method**: Use `ffprobe -print_format json -show_chapters`
- **Parse**: Convert FFmpeg chapter format to our Chapter struct
- **Timebase**: Convert from file's timebase to milliseconds
- **Store**: Set source to `ChapterSource::Embedded`

#### 2. Generate from Files
- **Function**: `generate_chapters_from_files(files: &[String]) -> Result<Vec<Chapter>>`
- **Current**: Uses placeholder duration (1 hour per file) - **NEEDS FIXING**
- **Method**: 
  1. Use `ffprobe` to get **accurate duration** of each file (decimal seconds)
  2. Convert to milliseconds: `duration_ms = (duration_sec * 1000.0).round() as u64`
  3. Calculate cumulative start times (sum previous durations)
  4. Create one chapter per file
  5. Use filename (without extension) as title
- **Store**: Set source to `ChapterSource::Generated`
- **Fix**: Replace placeholder in `Message::MapChaptersFromFiles` handler

#### 3. Lookup from Provider
- **Function**: `lookup_chapters_from_provider(asin: &str, provider: &str) -> Result<Vec<Chapter>>`
- **Current**: ✅ Implemented in `AudioService::fetch_chapters_by_asin()`
- **Providers**: Audnexus (already working)
- **Format**: Audnexus returns `startOffsetMs` and `lengthMs` (already in milliseconds)
- **Note**: Current implementation already handles milliseconds correctly
- **Enhancement**: Add timestamp shifting support (remove intro offset)
- **Store**: Set source to `ChapterSource::Lookup`

#### 4. Timestamp Shifting
- **Function**: `shift_chapters(chapters: &mut [Chapter], offset_ms: i64)`
- **Purpose**: Align external chapters with local file
- **Logic**: 
  - Add offset to all chapter start times
  - Adjust durations if needed
  - Skip locked chapters
- **Use Case**: Remove intro, align with file start

#### 5. Global Time Shift
- **Function**: `shift_all_chapters(chapters: &mut [Chapter], offset_ms: i64)`
- **Purpose**: Shift entire timeline (e.g., remove intro from all chapters)
- **Logic**: Apply offset to all unlocked chapters
- **Validation**: Ensure no negative start times

#### 6. Individual Chapter Shift
- **Function**: `shift_chapter(chapters: &mut [Chapter], index: usize, new_start_ms: u64)`
- **Purpose**: Move single chapter, ripple to subsequent
- **Logic**:
  1. Calculate offset: `new_start - old_start`
  2. Shift this chapter and all subsequent unlocked chapters
  3. Adjust durations to prevent gaps
  4. Validate no overlaps

#### 7. Chapter Editing
- **Add**: `add_chapter(chapters: &mut Vec<Chapter>, title: String, start_ms: u64)`
- **Edit**: `edit_chapter(chapters: &mut [Chapter], index: usize, title: Option<String>, start_ms: Option<u64>)`
- **Delete**: `delete_chapter(chapters: &mut Vec<Chapter>, index: usize)`
- **Lock/Unlock**: `toggle_chapter_lock(chapters: &mut [Chapter], index: usize)`

### Chapter Validation

#### Validation Rules
- [ ] Chapters must be in chronological order
- [ ] No negative start times
- [ ] No gaps between chapters (unless intentional)
- [ ] No overlaps between chapters
- [ ] Total duration should not exceed file duration
- [ ] Locked chapters cannot be shifted by global operations

#### Validation Function
- **Function**: `validate_chapters(chapters: &[Chapter], total_duration_ms: u64) -> Result<(), Vec<String>>`
- **Returns**: List of validation errors (if any)
- **Use**: Before saving or embedding chapters

### Chapter Storage

#### In Application State
- **Current**: Stored in `Lectern.chapters: Vec<Chapter>`
- **Persistence**: Not yet persisted (lost on app close)
- **Future**: Could save to sidecar file (like ABS `metadata.json`)

#### In Audio File
- **When**: Only when user converts to M4B
- **Format**: FFMETADATA1 chapters section
- **Timebase**: TIMEBASE=1/1000 (milliseconds)
- **Note**: This is a one-way operation (embeds virtual chapters into file)

### Chapter Lookup Implementation

#### Audnexus Chapter Format
From Audnexus API, chapters come as:
```json
{
  "chapters": [
    {
      "start": 0.0,
      "end": 356.257959,
      "title": "001 of 152"
    }
  ]
}
```

#### Conversion Logic
```rust
fn convert_audnexus_chapters(json: &serde_json::Value) -> Vec<Chapter> {
    // Extract chapters array
    // Convert start/end (seconds) to milliseconds
    // Create Chapter structs
    // Return vector
}
```

#### Timestamp Alignment
- **Problem**: Audnexus chapters may start at 0, but local file may have intro
- **Solution**: 
  1. Detect intro duration (user input or auto-detect)
  2. Shift all chapters by intro duration
  3. Or: Match first chapter start to file start

### UI Workflow

#### Chapter Editor Tab
1. **Display**: Show list of chapters with start time, duration, title
2. **Edit**: Allow inline editing of title and start time
3. **Add/Delete**: Buttons to add/remove chapters
4. **Shift Controls**: 
   - Global shift input (seconds)
   - Individual chapter drag-to-reposition
5. **Lock Toggle**: Lock/unlock individual chapters
6. **Source Indicator**: Show where chapters came from (Embedded/Generated/Lookup/Manual)

#### Chapter Actions
- **Extract from File**: Button to re-probe file for chapters
- **Generate from Files**: Button to create one-per-file chapters
- **Lookup**: Button to fetch from provider (requires ASIN)
- **Validate**: Check button to validate chapter timeline
- **Reset**: Button to clear all chapters

### Integration with Conversion

#### Before Conversion
- [ ] Validate chapters are correct
- [ ] Warn if chapters exceed file duration
- [ ] Allow user to adjust if needed

#### During Conversion
- [ ] Generate FFMETADATA1 chapters section
- [ ] Include all virtual chapters
- [ ] Convert to file's timebase
- [ ] Embed into M4B file

#### After Conversion
- [ ] Chapters are now embedded in file
- [ ] Virtual chapters remain in app state
- [ ] User can still edit (creates new virtual set)

## Implementation Plan

### Phase 1: Fix Current Implementation
- [ ] **Fix `MapChaptersFromFiles`**: Replace placeholder duration with real ffprobe duration
- [ ] **Add ffprobe duration extraction**: Function to get file duration accurately
- [ ] **Improve chapter lookup**: Add timestamp shifting support for Audnexus chapters

### Phase 2: Extract from File
- [ ] Implement `extract_chapters_from_file()` using ffprobe
- [ ] Parse FFmpeg chapter JSON format
- [ ] Handle different timebases (convert to milliseconds)
- [ ] Add "Extract from File" button to UI

### Phase 3: Chapter Operations & Validation
- [ ] Implement `shift_chapters()` and `shift_all_chapters()`
- [ ] Implement individual chapter shift with ripple effect
- [ ] Implement validation function (gaps, overlaps, duration)
- [ ] Add validation warnings to UI

### Phase 4: UI Enhancements
- [ ] Add shift controls (global time shift input)
- [ ] Improve individual chapter editing (drag-to-reposition)
- [ ] Show chapter source indicator
- [ ] Add validation status display

### Phase 2: Lookup & Shifting
- [ ] Implement `lookup_chapters_from_provider()`
- [ ] Implement `shift_chapters()` and `shift_all_chapters()`
- [ ] Implement individual chapter shift with ripple
- [ ] Add timestamp alignment logic

### Phase 3: UI Enhancements
- [ ] Improve chapter editor UI
- [ ] Add shift controls
- [ ] Add lock/unlock functionality
- [ ] Show chapter source indicator
- [ ] Add validation warnings

### Phase 5: Persistence (Future)
- [ ] Save chapters to sidecar file (metadata.json)
- [ ] Load chapters on app start
- [ ] Sync with file if it has embedded chapters
- [ ] Track chapter source in state

## Priority Fixes Needed

### High Priority
1. **Fix duration calculation in `MapChaptersFromFiles`**
   - Currently uses placeholder (1 hour per file)
   - Should use `ffprobe` to get actual file durations
   - Location: `src/ui/mod.rs` line ~1387

2. **Add timestamp shifting for looked-up chapters**
   - Audnexus chapters may need offset (remove intro)
   - Add UI input for intro duration offset
   - Apply offset to all chapters after lookup

3. **Add chapter extraction from file**
   - Use `ffprobe -show_chapters` to read embedded chapters
   - Add "Extract from File" button to UI
   - Parse FFmpeg chapter format

### Medium Priority
4. **Add chapter validation**
   - Check for gaps, overlaps, duration mismatches
   - Show warnings in UI
   - Prevent invalid states

5. **Improve shift operations**
   - Global time shift (shift all chapters)
   - Individual chapter shift with ripple
   - Lock mechanism (prevent shifting locked chapters)

### Low Priority
6. **Track chapter source**
   - Add source field to Chapter or state
   - Display source in UI
   - Help user understand where chapters came from

## Key Differences from ABS

### What We'll Do Similarly
- ✅ Virtual chapters (stored in app, not file)
- ✅ Multiple chapter sources (extract, generate, lookup, manual)
- ✅ Timestamp shifting for alignment
- ✅ Lock mechanism for chapters
- ✅ Validation before embedding

### What We'll Do Differently
- 🔄 Embed chapters during conversion (not separate "Embed Metadata" step)
- 🔄 Simpler persistence (maybe just in app state for now)
- 🔄 Direct integration with conversion workflow

## Technical Details

### FFprobe Chapter Extraction
```bash
ffprobe -v quiet -print_format json -show_chapters input.m4b
```

Output format:
```json
{
  "chapters": [
    {
      "id": 0,
      "time_base": "1/1000",
      "start": 0,
      "start_time": "0.000000",
      "end": 356257,
      "end_time": "356.257000",
      "tags": {
        "title": "Chapter 1"
      }
    }
  ]
}
```

### Chapter Timebase Conversion
- **File Timebase**: May vary (1/1000, 1/1000000, 1/1)
- **Our Format**: Always milliseconds (u64)
- **Conversion**: `start_ms = (start * timebase_denominator) / timebase_numerator * 1000`
- **Example**: 
  - File timebase: `1/1000`, start: `356257` → `356257` ms
  - File timebase: `1/1`, start: `356.257` → `356257` ms (multiply by 1000)
  - File timebase: `1/1000000`, start: `356257000` → `356257` ms (divide by 1000)

### FFprobe Chapter Extraction Example
```bash
ffprobe -v quiet -print_format json -show_chapters input.m4b
```

Expected output structure:
```json
{
  "chapters": [
    {
      "id": 0,
      "time_base": "1/1000",
      "start": 0,
      "start_time": "0.000000",
      "end": 356257,
      "end_time": "356.257000",
      "tags": {
        "title": "Chapter 1"
      }
    }
  ]
}
```

Parsing logic:
- Read `time_base` (e.g., "1/1000")
- Read `start` and `end` (in time_base units)
- Convert to milliseconds based on time_base
- Read `tags.title` for chapter name

### Duration Calculation
- **From File**: `duration = end_time - start_time` (in file's timebase)
- **Our Format**: `duration_ms = end_ms - start_ms`
- **Validation**: Sum of all chapter durations should ≈ total file duration

## Key Improvements Needed (Based on ABS)

### 1. Accurate Duration Calculation
**Current Problem**: `MapChaptersFromFiles` uses placeholder (1 hour per file)
**Solution**: Use `ffprobe` to get actual file durations
**Impact**: Critical for accurate chapter timing

### 2. Timestamp Shifting
**Missing Feature**: No way to align external chapters with local file
**Solution**: Add offset input (e.g., "Remove 3 second intro")
**Use Case**: Audnexus chapters start at 0, but file has intro

### 3. Chapter Extraction
**Missing Feature**: Cannot read embedded chapters from existing M4B
**Solution**: Implement `extract_chapters_from_file()` using ffprobe
**Benefit**: Users can edit chapters from existing files

### 4. Validation
**Missing Feature**: No validation of chapter timeline
**Solution**: Check for gaps, overlaps, duration mismatches
**Benefit**: Prevent invalid chapter states

### 5. Ripple Effect
**Partial Implementation**: Can adjust individual chapters, but no automatic ripple
**Solution**: When moving chapter, automatically adjust subsequent chapters
**Benefit**: Maintains timeline continuity

## Testing Considerations

### Test Cases
1. Extract chapters from M4B file
2. Generate chapters from directory of MP3s (with real durations)
3. Lookup chapters from Audnexus
4. Shift all chapters by offset (remove intro)
5. Shift single chapter (ripple effect)
6. Lock chapter and attempt global shift
7. Validate chapters (gaps, overlaps, duration)
8. Edit chapter title and start time
9. Add/delete chapters
10. Convert chapters to FFMETADATA1 format
11. **Mixed sources**: Extract some, lookup others, manual entry
12. **Duration mismatch**: Warn when chapters exceed file duration
