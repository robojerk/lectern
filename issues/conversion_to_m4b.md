# M4B Conversion Implementation Plan

## Overview
Implement FFmpeg-based conversion to create M4B files with embedded metadata, cover art, and chapters.

## Input Scenarios

### 1. Existing M4B File
- **Detection**: Check file extension `.m4b` or `.m4a`
- **Approach**: Re-encode with new metadata/cover/chapters
- **Strategy**: 
  - Extract existing audio stream
  - Replace metadata, cover, and chapters
  - Re-encode to ensure compatibility

### 2. Directory of Audio Files
- **Detection**: Check if input path is a directory
- **File Types**: MP3, AAC, WAV, FLAC, M4A, etc.
- **Approach**: 
  - Sort files (by filename or user-defined order)
  - Concatenate into single stream
  - Convert to AAC format
  - Embed metadata, cover, chapters

### 3. Single Audio File
- **Detection**: Single file (not M4B, not directory)
- **Approach**: 
  - Convert format if needed (to AAC)
  - Embed metadata, cover, chapters

## FFmpeg Command Structure

### Base Command Template
```bash
ffmpeg [input_options] -i <audio_input> -i <metadata_file> -i <cover_image> \
  -map 0:a -map 1:m:chapters -map 2:v \
  -c:a aac -b:a 128k -c:v copy \
  -disposition:v attached_pic \
  [metadata_flags] \
  output.m4b
```

### Multiple Files (Concat Demuxer - Matching Codecs)
```bash
# Step 1: Create concat file (with proper escaping)
# concat_list.txt:
file '/path/to/Chapter 1.mp3'
file '/path/to/Chapter 2 - John'\''s Story.mp3'  # Note: escaped single quote

ffmpeg -f concat -safe 0 -i concat_list.txt -i metadata.txt -i cover.jpg \
  -map 0:a -map 1:m:chapters -map 2:v \
  -c:a aac -b:a 128k -c:v copy \
  -disposition:v attached_pic \
  [metadata_flags] \
  output.m4b
```

### Multiple Files (Filter Complex - Mismatched Codecs)
```bash
# For files with different codecs/sample rates - ensures gapless playback
ffmpeg -i file1.mp3 -i file2.aac -i file3.wav -i metadata.txt -i cover.jpg \
  -filter_complex "[0:a][1:a][2:a]concat=n=3:v=0:a=1[outa]" \
  -map "[outa]" -map 1:m:chapters -map 2:v \
  -c:a aac -b:a 128k -ac 2 -c:v copy \
  -disposition:v attached_pic \
  [metadata_flags] \
  output.m4b
```

### Multiple Files (Filter Complex Script - Long File Lists)
```bash
# For hundreds of files, use filter script to avoid command-line length limit
# filter_script.txt:
[0:a][1:a][2:a]...[199:a]concat=n=200:v=0:a=1[outa]

ffmpeg -i file1.mp3 -i file2.mp3 ... -i file200.mp3 -i metadata.txt -i cover.jpg \
  -filter_complex_script filter_script.txt \
  -map "[outa]" -map 1:m:chapters -map 2:v \
  -c:a aac -b:a 128k -ac 2 -c:v copy \
  -disposition:v attached_pic \
  [metadata_flags] \
  output.m4b
```

### With Volume Normalization
```bash
ffmpeg -i input.mp3 -i metadata.txt -i cover.jpg \
  -map 0:a -map 1:m:chapters -map 2:v \
  -af "speechnorm=e=6.5:r=0.0001:l=1" \
  -c:a aac -b:a 128k -c:v copy \
  -disposition:v attached_pic \
  [metadata_flags] \
  output.m4b
```

## Implementation Steps

### Phase 1: Input Detection & Preparation

#### Task 1.1: Detect Input Type
- [ ] Function: `detect_input_type(path: &str) -> InputType`
- [ ] Return enum: `SingleM4B`, `Directory`, `SingleAudioFile`
- [ ] Check file extensions and directory existence

#### Task 1.2: Collect Audio Files
- [ ] Function: `collect_audio_files(path: &str) -> Result<Vec<String>>`
- [ ] For directories: scan and filter audio files
- [ ] Sort files (alphabetically or preserve order)
- [ ] Validate all files exist and are readable

#### Task 1.3: Validate Inputs
- [ ] Check book metadata exists
- [ ] Check cover image exists (if provided)
- [ ] Check chapters exist (if provided)
- [ ] Validate output path is writable

#### Task 1.4: Detect Audio File Parameters
- [ ] Function: `probe_audio_file(path: &str) -> Result<AudioParams>`
- [ ] Use `ffprobe -v quiet -print_format json -show_format -show_streams`
- [ ] Parse JSON output (use `serde_json`)
- [ ] Extract: codec, sample_rate, channels, bitrate, duration
- [ ] **Critical**: Use `format.duration` (decimal string) for precision
- [ ] Convert duration: `f64::from_str(duration_str)? * 1000.0` → milliseconds
- [ ] Struct: `AudioParams { codec, sample_rate, channels, bitrate, duration_ms }`
- [ ] Determine if files are compatible for concat demuxer (same params)
- [ ] If incompatible, use filter_complex instead

### Phase 2: Metadata File Generation

#### Task 2.1: Generate FFMETADATA1 File
- [ ] Function: `generate_ffmetadata(book: &BookMetadata, chapters: &[Chapter]) -> String`
- [ ] Format: FFMETADATA1 text format
- [ ] Include all metadata fields:
  - `title`, `artist` (author), `album`, `genre`
  - `date` (publish_year), `publisher`, `description`
  - `comment` (description or custom)
  - Custom tags: `series`, `narrator`, `isbn`, `asin`, etc.

#### Task 2.2: Generate Chapters Section
- [ ] Function: `generate_chapters_ffmetadata(chapters: &[Chapter]) -> String`
- [ ] Format: FFmpeg chapter format
- [ ] Timebase: Use `TIMEBASE=1/1000` (milliseconds) to match our Chapter struct
- [ ] Convert each chapter:
  ```
  [CHAPTER]
  TIMEBASE=1/1000
  START=<start_time_ms>
  END=<end_time_ms>
  title=<chapter_title>
  ```

#### Task 2.3: Auto-Generate Chapters from Files
- [ ] Function: `generate_chapters_from_files(files: &[String]) -> Result<Vec<Chapter>>`
- [ ] Use `ffprobe` to get duration of each file
- [ ] **Critical**: Use `format.duration` (decimal seconds) NOT integer seconds
- [ ] Multiply duration by 1000 and round to nearest integer for milliseconds
- [ ] Example: `format.duration = "356.257959"` → `356258` milliseconds
- [ ] **Precision matters**: 0.5-second drift per file = minutes of misalignment in 50-file book
- [ ] Create one chapter per file
- [ ] Calculate cumulative start times (sum previous durations)
- [ ] Use filename (without extension) as chapter title
- [ ] Only used if `rewrite_chapters: true` and no chapters provided

#### Task 2.4: Write Metadata File
- [ ] Function: `write_metadata_file(metadata: &str, temp_dir: &Path) -> Result<PathBuf>`
- [ ] Use `tempfile` crate for proper temp file management
- [ ] Create temporary file
- [ ] Write FFMETADATA1 content
- [ ] Return path for FFmpeg
- [ ] Ensure file is cleaned up after conversion

### Phase 3: FFmpeg Command Construction

#### Task 3.1: Build Base Command
- [ ] Function: `build_ffmpeg_command(config: &ConversionConfig) -> Command`
- [ ] Use `std::process::Command`
- [ ] Set executable path (check for `ffmpeg` in PATH)
- [ ] Add input files in order: audio, metadata, cover

#### Task 3.2: Handle Multiple Files
- [ ] Function: `create_concat_file(files: &[String]) -> Result<PathBuf>`
- [ ] Generate concat demuxer file format:
  ```
  file 'path/to/file1.mp3'
  file 'path/to/file2.mp3'
  ```
- [ ] Use absolute paths
- [ ] Handle special characters in paths
- [ ] Use `tempfile` crate for temp file management

#### Task 3.2a: Choose Concat Method
- [ ] Function: `should_use_concat_demuxer(files: &[String], normalize_volume: bool) -> Result<ConcatMethod>`
- [ ] **Decision Hierarchy**:
  1. Probe all files to check if parameters match (codec, sample_rate, channels)
  2. If **volume normalization is enabled**: Always use FilterComplex (must re-encode anyway)
  3. If **ALL** parameters match: Use Concat Demuxer (fast, stream copy)
  4. If **ANY** parameters differ: Use FilterComplex (prevents clicks/pops, ensures gapless)
- [ ] Return method choice: `ConcatMethod::Demuxer` or `ConcatMethod::FilterComplex`

#### Task 3.2b: Handle Long File Lists (FilterComplex)
- [ ] Function: `create_filter_complex_script(files: &[String]) -> Result<PathBuf>`
- [ ] **Problem**: Command-line length limit (especially Windows) with hundreds of files
- [ ] **Solution**: Write filter script file instead of inline `-filter_complex`
- [ ] Use `-filter_complex_script` flag instead of `-filter_complex`
- [ ] Format: Write filter graph as text file
- [ ] Use `tempfile` crate for script file management

#### Task 3.3: Add Metadata Flags
- [ ] Function: `add_metadata_args(cmd: &mut Command, book: &BookMetadata)`
- [ ] Add `-metadata` flags for each field:
  - `-metadata title="..."` 
  - `-metadata artist="..."` (author)
  - `-metadata album="..."` (series if available)
  - `-metadata genre="..."`
  - `-metadata date="..."` (publish_year)
  - `-metadata publisher="..."`
  - `-metadata comment="..."` (description)
  - Custom: `-metadata series="..."`, `-metadata narrator="..."`, etc.
- [ ] **Important**: Keep both FFMETADATA1 file AND CLI flags
- [ ] Some players (older iPods, specific Android apps) prioritize different sources
- [ ] Redundancy ensures maximum compatibility across hardware players

#### Task 3.4: Add Audio/Video Options
- [ ] Set audio codec: `-c:a aac`
- [ ] Set bitrate: `-b:a 128k` (configurable)
- [ ] Set cover: `-c:v copy -disposition:v attached_pic`
- [ ] Set mapping: `-map 0:a -map 1:m:chapters -map 2:v`

#### Task 3.5: Add Volume Normalization
- [ ] Function: `add_volume_normalization(cmd: &mut Command, enabled: bool)`
- [ ] If enabled: Add `-af speechnorm` filter
- [ ] Use `speechnorm` (not `loudnorm`) - designed for speech/audiobooks
- [ ] Prevents clipping while ensuring consistent levels
- [ ] Example: `-af "speechnorm=e=6.5:r=0.0001:l=1"`
- [ ] **Note**: Volume normalization requires re-encoding, so always use FilterComplex when enabled

#### Task 3.7: Handle Filename Escaping
- [ ] Function: `escape_concat_filename(path: &str) -> String`
- [ ] **Problem**: Filenames with single quotes break concat file format
- [ ] **Solution**: Properly escape single quotes in concat file
- [ ] Format: `file 'Chapter 1 - John'\''s Story.mp3'` (single quote escaped as `'\''`)
- [ ] Also handle other special characters if needed
- [ ] Test with: `Chapter 1 - John's Story.mp3`, `File "Name".mp3`, etc.

#### Task 3.8: Ensure Consistent Audio Channels
- [ ] Function: `normalize_audio_channels(cmd: &mut Command, target_channels: u32)`
- [ ] **Problem**: Mixed mono/stereo files cause concat filter failures
- [ ] **Solution**: Force consistent channel layout before concatenation
- [ ] Add `-ac 2` (stereo) or `-ac 1` (mono) to output mapping
- [ ] Default: Stereo (`-ac 2`) for better compatibility
- [ ] Apply in filter_complex: `aformat=channel_layouts=stereo` or use `-ac` flag

#### Task 3.6: Add Cover Image Scaling
- [ ] Function: `scale_cover_image(input: &str, max_size: u32, temp_dir: &Path) -> Result<PathBuf>`
- [ ] Use FFmpeg to resize cover before embedding
- [ ] Command: `ffmpeg -i input.jpg -vf scale=1000:1000:force_original_aspect_ratio=decrease output.jpg`
- [ ] Preserve aspect ratio
- [ ] Only scale if image exceeds max_size
- [ ] Return path to scaled image (or original if no scaling needed)

### Phase 4: FFmpeg Execution

#### Task 4.1: Execute FFmpeg
- [ ] Function: `execute_ffmpeg(cmd: Command) -> Result<()>`
- [ ] Use `tokio::process::Command` for async execution (prevents UI blocking)
- [ ] Capture stdout/stderr
- [ ] Handle errors gracefully
- [ ] Return success/failure
- [ ] **Atomic Write**: Write to `output.m4b.tmp` first
- [ ] Only rename to `output.m4b` after successful completion
- [ ] Prevents corrupted files if FFmpeg fails mid-process

#### Task 4.2: Progress Reporting (Optional)
- [ ] Function: `get_total_duration(files: &[String]) -> Result<u64>` (milliseconds)
- [ ] Use `ffprobe` to sum durations of all input files
- [ ] **Use decimal duration** (`format.duration`) and convert to milliseconds accurately
- [ ] Parse FFmpeg stderr for progress
- [ ] **Note**: With `speechnorm`, output format may differ slightly
- [ ] Look for `out_time_ms=` or `time=HH:MM:SS.mmm` in stderr
- [ ] Use `tokio::io::BufReader` to read stderr line by line
- [ ] Parse each line for time information
- [ ] Calculate progress: `current_time_ms / total_duration_ms`
- [ ] Send progress updates to UI via channel or callback
- [ ] Message: `ConversionProgress(f64)` (0.0 to 1.0)
- [ ] Update UI periodically (e.g., every 1% or 1 second)

#### Task 4.3: Cleanup Temporary Files
- [ ] Delete metadata file after conversion
- [ ] Delete concat file (if used)
- [ ] Handle errors during cleanup

### Phase 5: Error Handling

#### Task 5.1: Validate FFmpeg Installation
- [ ] Check if `ffmpeg` is in PATH
- [ ] Check version (if needed)
- [ ] Return helpful error if missing

#### Task 5.2: Handle FFmpeg Errors
- [ ] Parse stderr for error messages
- [ ] Return user-friendly error messages
- [ ] Log detailed errors for debugging

#### Task 5.3: Validate Output
- [ ] Check output file exists
- [ ] Check output file size (not zero)
- [ ] Optional: Verify file can be read

## Data Structures

### ConversionConfig
```rust
pub struct ConversionConfig {
    pub input_path: String,
    pub output_path: String,
    pub book_metadata: BookMetadata,
    pub cover_image_path: Option<String>,
    pub chapters: Vec<Chapter>,
    pub audio_bitrate: Option<String>, // e.g., "128k"
    pub audio_codec: String, // default: "aac"
    pub processing_options: ProcessingOptions,
}
```

### ProcessingOptions
```rust
pub struct ProcessingOptions {
    pub normalize_volume: bool,      // Use speechnorm filter for consistent levels
    pub rewrite_chapters: bool,      // Force one chapter per file for directory imports
    pub max_cover_size: u32,         // e.g., 1000 for 1000x1000 (0 = no resize)
    pub use_temp_dir: bool,          // Use temp directory for intermediate files
    pub atomic_write: bool,          // Write to .tmp then rename (default: true)
}
```

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            normalize_volume: true,  // Default: normalize for better UX
            rewrite_chapters: false,  // Default: use provided chapters
            max_cover_size: 1000,     // Default: 1000x1000 max
            use_temp_dir: true,       // Default: use temp dir
            atomic_write: true,       // Default: atomic writes
        }
    }
}
```

### InputType
```rust
pub enum InputType {
    SingleM4B(String),
    Directory(Vec<String>), // sorted file paths
    SingleAudioFile(String),
}
```

### AudioParams
```rust
pub struct AudioParams {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: Option<u32>,
    pub duration_ms: u64,  // Precise: from format.duration (decimal) * 1000
}
```

### ConcatMethod
```rust
pub enum ConcatMethod {
    Demuxer,        // Fast, requires matching params
    FilterComplex,  // Slower, re-encodes for gapless
}
```

## File Organization

### New Module: `src/services/conversion.rs`
- All conversion-related functions
- FFmpeg command building
- Metadata file generation
- Progress tracking

### Functions to Implement

1. **Input Detection**
   - `detect_input_type(path: &str) -> Result<InputType>`
   - `collect_audio_files(dir: &str) -> Result<Vec<String>>`

2. **Metadata Generation**
   - `generate_ffmetadata(book: &BookMetadata, chapters: &[Chapter]) -> String`
   - `write_metadata_file(metadata: &str, temp_dir: &Path) -> Result<PathBuf>`
   - `create_concat_file(files: &[String]) -> Result<PathBuf>`
   - `generate_chapters_from_files(files: &[String]) -> Result<Vec<Chapter>>`

3. **Audio Processing**
   - `probe_audio_file(path: &str) -> Result<AudioParams>`
   - `get_total_duration(files: &[String]) -> Result<u64>`
   - `should_use_concat_demuxer(files: &[String], normalize_volume: bool) -> Result<ConcatMethod>`
   - `create_filter_complex_script(files: &[String]) -> Result<PathBuf>`
   - `escape_concat_filename(path: &str) -> String`
   - `normalize_audio_channels(cmd: &mut Command, target_channels: u32)`
   - `scale_cover_image(input: &str, max_size: u32, temp_dir: &Path) -> Result<PathBuf>`

4. **FFmpeg Execution**
   - `build_ffmpeg_command(config: &ConversionConfig) -> Result<tokio::process::Command>`
   - `add_volume_normalization(cmd: &mut Command, enabled: bool)`
   - `execute_ffmpeg(cmd: Command) -> Result<()>`
   - `validate_ffmpeg_installed() -> Result<()>`

5. **Main Conversion Function**
   - `convert_to_m4b(config: ConversionConfig) -> Result<String>` // returns output path
   - Handles atomic writes (tmp file then rename)
   - Manages temporary files cleanup
   - Returns final output path

## Integration Points

### Update `src/services.rs`
- Replace stub `convert_to_m4b` with real implementation
- Call functions from `conversion.rs` module

### Update `src/ui/mod.rs`
- Update `Message::StartConversion` handler
- Pass all required data to conversion function
- Handle progress updates (if implemented)
- Update `Message::ConversionCompleted` with result

### Update `src/ui/views/convert.rs`
- Show conversion progress (if implemented)
- Display better error messages
- Show estimated time remaining (if implemented)

## Configuration Options

### Audio Quality Settings
- Bitrate: 128k (default), 192k, 256k, variable
- Sample rate: 44.1kHz (default), 48kHz
- Codec: AAC (default), could support others

### Chapter Timebase
- Use `TIMEBASE=1/1000` (milliseconds) - matches current Chapter struct
- Alternative: `TIMEBASE=1/1000000` (microseconds) for more precision

### Multiple File Handling
- Use concat demuxer (fast, requires same codec)
- Fallback to filter_complex if codecs differ

## Testing Considerations

### Test Cases
1. Single MP3 file → M4B
2. Directory of MP3s → M4B
3. Existing M4B → M4B (re-encode)
4. Mixed formats (MP3 + AAC) → M4B
5. No chapters → M4B without chapters
6. No cover → M4B without cover
7. All metadata fields populated
8. Special characters in paths/titles
9. Large files (multi-hour audiobooks)
10. FFmpeg not installed (error handling)
11. **Hundreds of files** (test command-line length limits)
12. **Filenames with single quotes** (test escaping: `John's Story.mp3`)
13. **Mixed mono/stereo files** (test channel normalization)
14. **Volume normalization enabled** (test speechnorm filter)
15. **Precise chapter timing** (test decimal duration accuracy)

## Dependencies

### Required
- `std::path::Path` (built-in)
- `std::fs` (built-in)
- `std::io` (built-in)
- `tokio::process::Command` - async process execution (prevents UI blocking)
- `tokio::io::BufReader` - line-by-line stderr reading for progress
- `tempfile` - proper temporary file management (add to Cargo.toml)

### Optional (for progress)
- `regex` (already in Cargo.toml) - parse FFmpeg output
- `serde_json` (already in Cargo.toml) - parse ffprobe JSON output

## Example FFmpeg Commands

### Single File
```bash
ffmpeg -i input.mp3 -i metadata.txt -i cover.jpg \
  -map 0:a -map 1:m:chapters -map 2:v \
  -c:a aac -b:a 128k -c:v copy -disposition:v attached_pic \
  -metadata title="Book Title" \
  -metadata artist="Author Name" \
  -metadata album="Series Name" \
  -metadata genre="Science Fiction" \
  -metadata date="2024" \
  -metadata publisher="Publisher Name" \
  -metadata comment="Description text" \
  output.m4b
```

### Multiple Files (Concat)
```bash
# concat_list.txt:
file '/path/to/file1.mp3'
file '/path/to/file2.mp3'
file '/path/to/file3.mp3'

ffmpeg -f concat -safe 0 -i concat_list.txt -i metadata.txt -i cover.jpg \
  -map 0:a -map 1:m:chapters -map 2:v \
  -c:a aac -b:a 128k -c:v copy -disposition:v attached_pic \
  [metadata flags] \
  output.m4b
```

### Example FFMETADATA1 File
```
;FFMETADATA1
title=Book Title
artist=Author Name
album=Series Name
genre=Science Fiction
date=2024
publisher=Publisher Name
comment=Description text

[CHAPTER]
TIMEBASE=1/1000
START=0
END=356257
title=Chapter 1

[CHAPTER]
TIMEBASE=1/1000
START=356257
END=498599
title=Chapter 2
```

## Additional Technical Considerations

### Volume Normalization Details
- **Why**: Audiobooks from different sources have varying volume levels
- **Solution**: Use `speechnorm` filter (not `loudnorm`) - designed for speech
- **Command**: `-af "speechnorm=e=6.5:r=0.0001:l=1"`
- **Default**: Enabled in `ProcessingOptions`

### Gapless Playback
- **Problem**: Concat demuxer with mismatched codecs/sample rates causes clicks/pops
- **Solution**: 
  - Detect if all files have matching parameters (codec, sample_rate, channels)
  - If **volume normalization enabled**: Always use FilterComplex (must re-encode)
  - If all match: Use concat demuxer (fast, stream copy)
  - If any differ: Use filter_complex with re-encoding (slower but gapless)
- **Channel Consistency**: Force uniform channels (`-ac 2` for stereo) to prevent concat failures

### Cover Art Optimization
- **Problem**: Large covers (4000x4000) bloat M4B header, slow library scanning
- **Solution**: Resize to max 1000x1000 before embedding
- **Implementation**: Use FFmpeg `-vf scale=1000:1000:force_original_aspect_ratio=decrease`
- **Default**: Enabled (max_cover_size: 1000)

### Command-Line Length Limits
- **Problem**: Windows/OS command-line character limits with hundreds of files
- **Solution**: Use `-filter_complex_script` instead of inline `-filter_complex` for long file lists
- **Implementation**: Write filter graph to temporary script file
- **Threshold**: Consider using script if file count > 50 or estimated command length > 8000 chars

### Atomic Writes
- **Problem**: FFmpeg can fail mid-process, leaving corrupted file
- **Solution**: Write to `output.m4b.tmp`, rename only on success
- **Implementation**: Check return code before rename
- **Default**: Enabled (atomic_write: true)

### Filename Escaping
- **Problem**: Single quotes in filenames break concat file format
- **Solution**: Escape single quotes as `'\''` in concat file
- **Example**: `Chapter 1 - John's Story.mp3` → `file 'Chapter 1 - John'\''s Story.mp3'`
- **Implementation**: Function to properly escape filenames for concat format

### Timebase Precision
- **Problem**: Using integer seconds causes cumulative drift in chapter timing
- **Solution**: Use `format.duration` (decimal) from ffprobe, multiply by 1000 for milliseconds
- **Impact**: 0.5-second drift per file = minutes of misalignment in 50-file book
- **Implementation**: Always use decimal duration, round to nearest millisecond

## Future Enhancements

- [ ] Progress bar with time remaining
- [ ] Cancel conversion mid-process
- [ ] Batch conversion (multiple books)
- [ ] Preset quality profiles (low/medium/high)
- [ ] Custom FFmpeg arguments
- [ ] Verify output file integrity
- [ ] Support for other output formats (MP3 with chapters, etc.)
- [ ] Two-pass normalization (analyze then normalize)
- [ ] Chapter title extraction from filename patterns
