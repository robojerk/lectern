// M4B Conversion Module
// Implements FFmpeg-based conversion to create M4B files with embedded metadata, cover art, and chapters

use crate::models::BookMetadata;
use crate::models::chapters::Chapter;
use std::path::{Path, PathBuf};
use std::process::Command as StdCommand;
use tokio::process::Command as TokioCommand;
use std::fs;
use std::io::Write;
use tempfile::TempDir;
use anyhow::{Result, Context};

// Data structures
#[derive(Debug, Clone)]
pub struct ConversionConfig {
    pub input_path: String,
    pub output_path: String,
    pub book_metadata: BookMetadata,
    pub cover_image_path: Option<String>,
    pub chapters: Vec<Chapter>,
    pub audio_bitrate: Option<String>, // e.g., "128k"
    pub audio_codec: String, // "aac", "copy", "libopus"
    pub audio_channels: Option<u32>,
    pub processing_options: ProcessingOptions,
}

#[derive(Debug, Clone)]
pub struct ProcessingOptions {
    pub normalize_volume: bool,
    pub rewrite_chapters: bool,
    pub max_cover_size: u32,
    pub use_temp_dir: bool,
    pub atomic_write: bool,
}

impl Default for ProcessingOptions {
    fn default() -> Self {
        Self {
            normalize_volume: true,
            rewrite_chapters: false,
            max_cover_size: 1000,
            use_temp_dir: true,
            atomic_write: true,
        }
    }
}

#[derive(Debug)]
pub enum InputType {
    SingleM4B(String),
    Directory(Vec<String>), // sorted file paths
    SingleAudioFile(String),
}

#[derive(Debug)]
pub struct AudioParams {
    pub codec: String,
    pub sample_rate: u32,
    pub channels: u32,
    pub bitrate: Option<u32>,
    pub duration_ms: u64, // Precise: from format.duration (decimal) * 1000
}

#[derive(Debug)]
pub enum ConcatMethod {
    Demuxer,        // Fast, requires matching params
    FilterComplex,  // Slower, re-encodes for gapless
}

// Phase 1: Input Detection & Preparation

/// Detect the type of input (M4B file, directory, or single audio file)
pub fn detect_input_type(path: &str) -> Result<InputType> {
    let path = Path::new(path);
    
    if !path.exists() {
        return Err(anyhow::anyhow!("Input path does not exist: {}", path.display()));
    }
    
    if path.is_dir() {
        let files = collect_audio_files(path.to_str().unwrap())?;
        Ok(InputType::Directory(files))
    } else if path.is_file() {
        let ext = path.extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        
        if ext == "m4b" || ext == "m4a" {
            Ok(InputType::SingleM4B(path.to_string_lossy().to_string()))
        } else {
            // Check if it's an audio file
            let audio_exts = ["mp3", "aac", "wav", "flac", "ogg", "opus"];
            if audio_exts.contains(&ext.as_str()) {
                Ok(InputType::SingleAudioFile(path.to_string_lossy().to_string()))
            } else {
                Err(anyhow::anyhow!("Unsupported file type: {}", ext))
            }
        }
    } else {
        Err(anyhow::anyhow!("Invalid input path: {}", path.display()))
    }
}

/// Collect audio files from a directory
pub fn collect_audio_files(dir: &str) -> Result<Vec<String>> {
    let dir_path = Path::new(dir);
    if !dir_path.is_dir() {
        return Err(anyhow::anyhow!("Not a directory: {}", dir));
    }
    
    let audio_exts = ["mp3", "aac", "wav", "flac", "m4a", "m4b", "ogg", "opus"];
    let mut files = Vec::new();
    
    println!("[DEBUG] collect_audio_files: scanning directory: {}", dir);
    
    for entry in fs::read_dir(dir_path)
        .with_context(|| format!("Failed to read directory: {}", dir))? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                let ext_lower = ext.to_lowercase();
                if audio_exts.contains(&ext_lower.as_str()) {
                    let file_path = path.to_string_lossy().to_string();
                    println!("[DEBUG] Found audio file: {}", file_path);
                    files.push(file_path);
                }
            }
        }
    }
    
    // Sort files alphabetically
    files.sort();
    
    println!("[DEBUG] collect_audio_files: found {} audio files", files.len());
    
    if files.is_empty() {
        return Err(anyhow::anyhow!("No audio files found in directory: {}", dir));
    }
    
    Ok(files)
}

/// Probe an audio file to get its parameters
pub fn probe_audio_file(path: &str) -> Result<AudioParams> {
    let output = StdCommand::new("ffprobe")
        .args(&[
            "-v", "quiet",
            "-print_format", "json",
            "-show_format",
            "-show_streams",
            path,
        ])
        .output()
        .with_context(|| format!("Failed to execute ffprobe on: {}", path))?;
    
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(anyhow::anyhow!("ffprobe failed: {}", stderr));
    }
    
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)
        .with_context(|| "Failed to parse ffprobe JSON output")?;
    
    // Find audio stream
    let streams = json.get("streams")
        .and_then(|s| s.as_array())
        .ok_or_else(|| anyhow::anyhow!("No streams found in ffprobe output"))?;
    
    let audio_stream = streams.iter()
        .find(|s| s.get("codec_type").and_then(|c| c.as_str()) == Some("audio"))
        .ok_or_else(|| anyhow::anyhow!("No audio stream found"))?;
    
    let codec = audio_stream.get("codec_name")
        .and_then(|c| c.as_str())
        .unwrap_or("unknown")
        .to_string();
    
    let sample_rate = audio_stream.get("sample_rate")
        .and_then(|r| r.as_str())
        .and_then(|r| r.parse::<u32>().ok())
        .unwrap_or(44100);
    
    let channels = audio_stream.get("channels")
        .and_then(|c| c.as_u64())
        .map(|c| c as u32)
        .or_else(|| {
            audio_stream.get("channel_layout")
                .and_then(|l| l.as_str())
                .and_then(|l| {
                    // Parse channel layout like "stereo" -> 2, "mono" -> 1
                    if l.contains("stereo") || l.contains("2.0") {
                        Some(2)
                    } else if l.contains("mono") || l.contains("1.0") {
                        Some(1)
                    } else {
                        None
                    }
                })
        })
        .unwrap_or(2);
    
    let bitrate = json.get("format")
        .and_then(|f| f.get("bit_rate"))
        .and_then(|b| b.as_str())
        .and_then(|b| b.parse::<u32>().ok());
    
    // Critical: Use format.duration (decimal string) for precision
    let duration_ms = json.get("format")
        .and_then(|f| f.get("duration"))
        .and_then(|d| d.as_str())
        .and_then(|d| {
            d.parse::<f64>().ok()
        })
        .map(|d| (d * 1000.0).round() as u64)
        .unwrap_or(0);
    
    Ok(AudioParams {
        codec,
        sample_rate,
        channels,
        bitrate,
        duration_ms,
    })
}

/// Get total duration of all input files
pub fn get_total_duration(files: &[String]) -> Result<u64> {
    let mut total_ms = 0u64;
    for file in files {
        let params = probe_audio_file(file)?;
        total_ms += params.duration_ms;
    }
    Ok(total_ms)
}

/// Determine which concat method to use
pub fn should_use_concat_demuxer(files: &[String], normalize_volume: bool) -> Result<ConcatMethod> {
    // If volume normalization is enabled, always use FilterComplex (must re-encode anyway)
    if normalize_volume {
        return Ok(ConcatMethod::FilterComplex);
    }
    
    if files.is_empty() {
        return Err(anyhow::anyhow!("No files provided"));
    }
    
    // Probe first file to get reference parameters
    let first_params = probe_audio_file(&files[0])?;
    
    // Check if all files have matching parameters
    for file in files.iter().skip(1) {
        let params = probe_audio_file(file)?;
        
        if params.codec != first_params.codec
            || params.sample_rate != first_params.sample_rate
            || params.channels != first_params.channels {
            // Parameters differ - use FilterComplex for gapless playback
            return Ok(ConcatMethod::FilterComplex);
        }
    }
    
    // All parameters match - can use fast concat demuxer
    Ok(ConcatMethod::Demuxer)
}

// Phase 2: Metadata File Generation

/// Generate FFMETADATA1 file content
pub fn generate_ffmetadata(book: &BookMetadata, chapters: &[Chapter]) -> String {
    let mut metadata = String::from(";FFMETADATA1\n");
    
    // Add metadata fields
    if !book.title.is_empty() {
        metadata.push_str(&format!("title={}\n", escape_metadata_value(&book.title)));
    }
    if !book.author.is_empty() {
        metadata.push_str(&format!("artist={}\n", escape_metadata_value(&book.author)));
    }
    if let Some(ref series) = book.series {
        metadata.push_str(&format!("album={}\n", escape_metadata_value(series)));
    }
    if let Some(ref genre) = book.genre {
        metadata.push_str(&format!("genre={}\n", escape_metadata_value(genre)));
    }
    if let Some(ref year) = book.publish_year {
        metadata.push_str(&format!("date={}\n", escape_metadata_value(year)));
    }
    if let Some(ref publisher) = book.publisher {
        metadata.push_str(&format!("publisher={}\n", escape_metadata_value(publisher)));
    }
    if let Some(ref description) = book.description {
        metadata.push_str(&format!("comment={}\n", escape_metadata_value(description)));
    }
    
    // Add custom tags
    if let Some(ref narrator) = book.narrator {
        metadata.push_str(&format!("narrator={}\n", escape_metadata_value(narrator)));
    }
    if let Some(ref isbn) = book.isbn {
        metadata.push_str(&format!("isbn={}\n", escape_metadata_value(isbn)));
    }
    if let Some(ref asin) = book.asin {
        metadata.push_str(&format!("asin={}\n", escape_metadata_value(asin)));
    }
    if let Some(ref language) = book.language {
        metadata.push_str(&format!("language={}\n", escape_metadata_value(language)));
    }
    
    metadata.push('\n');
    
    // Add chapters
    metadata.push_str(&generate_chapters_ffmetadata(chapters));
    
    metadata
}

/// Generate chapters section for FFMETADATA1
pub fn generate_chapters_ffmetadata(chapters: &[Chapter]) -> String {
    let mut chapters_str = String::new();
    
    for chapter in chapters {
        chapters_str.push_str("[CHAPTER]\n");
        chapters_str.push_str("TIMEBASE=1/1000\n");
        chapters_str.push_str(&format!("START={}\n", chapter.start_time));
        chapters_str.push_str(&format!("END={}\n", chapter.start_time + chapter.duration));
        chapters_str.push_str(&format!("title={}\n", escape_metadata_value(&chapter.title)));
        chapters_str.push('\n');
    }
    
    chapters_str
}

/// Escape metadata values (handle special characters)
fn escape_metadata_value(value: &str) -> String {
    // FFMETADATA1 uses = for key-value pairs, so escape = and newlines
    value.replace('=', "\\=").replace('\n', "\\n")
}

/// Generate chapters from file list (one chapter per file)
pub fn generate_chapters_from_files(files: &[String]) -> Result<Vec<Chapter>> {
    let mut chapters = Vec::new();
    let mut cumulative_start = 0u64;
    
    for file in files {
        let params = probe_audio_file(file)?;
        let duration_ms = params.duration_ms;
        
        // Use filename (without extension) as chapter title
        let filename = Path::new(file)
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Unknown Chapter")
            .to_string();
        
        let chapter = Chapter::new(filename, cumulative_start, duration_ms);
        chapters.push(chapter);
        
        cumulative_start += duration_ms;
    }
    
    Ok(chapters)
}

/// Write metadata file to temporary location
pub fn write_metadata_file(metadata: &str, temp_dir: &Path) -> Result<PathBuf> {
    let mut file = fs::File::create(temp_dir.join("metadata.txt"))?;
    file.write_all(metadata.as_bytes())?;
    file.sync_all()?;
    Ok(temp_dir.join("metadata.txt"))
}

/// Escape filename for concat file format
pub fn escape_concat_filename(path: &str) -> String {
    // Escape single quotes: ' -> '\''
    path.replace('\'', "'\\''")
}

/// Create concat file for FFmpeg concat demuxer
pub fn create_concat_file(files: &[String], temp_dir: &Path) -> Result<PathBuf> {
    let mut concat_content = String::new();
    
    for file in files {
        let escaped = escape_concat_filename(file);
        concat_content.push_str(&format!("file '{}'\n", escaped));
    }
    
    let concat_path = temp_dir.join("concat_list.txt");
    let mut file = fs::File::create(&concat_path)?;
    file.write_all(concat_content.as_bytes())?;
    file.sync_all()?;
    
    Ok(concat_path)
}

/// Create filter complex script for long file lists
pub fn create_filter_complex_script(files: &[String], temp_dir: &Path, normalize_volume: bool) -> Result<PathBuf> {
    let mut script = String::new();
    
    // Build concat filter: [0:a][1:a][2:a]...concat=n=N:v=0:a=1[outa]
    let num_files = files.len();
    let inputs: Vec<String> = (0..num_files)
        .map(|i| format!("[{}:a]", i))
        .collect();
    
    script.push_str(&inputs.join(""));
    script.push_str(&format!("concat=n={}:v=0:a=1", num_files));
    
    if normalize_volume {
        script.push_str(",speechnorm=e=6.5:r=0.0001:l=1");
    }
    
    script.push_str("[outa]");
    
    let script_path = temp_dir.join("filter_script.txt");
    let mut file = fs::File::create(&script_path)?;
    file.write_all(script.as_bytes())?;
    file.sync_all()?;
    
    Ok(script_path)
}

/// Scale cover image if needed
pub async fn scale_cover_image(input: &str, max_size: u32, temp_dir: &Path) -> Result<PathBuf> {
    // Check if image needs scaling (would need image crate to get dimensions)
    // For now, just copy to temp dir if max_size is set
    let _input_path = Path::new(input);
    let output_path = temp_dir.join("cover_scaled.jpg");
    
    if max_size > 0 {
        // Use ffmpeg to scale
        let status = TokioCommand::new("ffmpeg")
            .arg("-i")
            .arg(input)
            .arg("-vf")
            .arg(&format!("scale={}:{}:force_original_aspect_ratio=decrease", max_size, max_size))
            .arg("-y") // Overwrite
            .arg(&output_path)
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .status()
            .await?;
        
        if status.success() {
            Ok(output_path)
        } else {
            // Fallback: copy original
            fs::copy(input, &output_path)?;
            Ok(output_path)
        }
    } else {
        // No scaling needed, copy original
        fs::copy(input, &output_path)?;
        Ok(output_path)
    }
}

// Phase 3: FFmpeg Command Construction

/// Validate FFmpeg is installed
pub fn validate_ffmpeg_installed() -> Result<()> {
    let output = StdCommand::new("ffmpeg")
        .arg("-version")
        .output()
        .map_err(|_| anyhow::anyhow!("ffmpeg not found in PATH. Please install FFmpeg."))?;
    
    if output.status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("ffmpeg command failed"))
    }
}

/// Build FFmpeg command for conversion
pub async fn build_ffmpeg_command(config: &ConversionConfig, temp_dir: &TempDir) -> Result<(TokioCommand, Option<ConcatMethod>)> {
    validate_ffmpeg_installed()?;
    
    let mut cmd = TokioCommand::new("ffmpeg");
    
    // Determine input files
    let input_type = detect_input_type(&config.input_path)?;
    
    // Store concat method for later use
    let concat_method = match &input_type {
        InputType::Directory(ref files) => {
            Some(should_use_concat_demuxer(files, config.processing_options.normalize_volume)?)
        }
        _ => None,
    };

    // Probe first file to get default parameters
    let first_file = match &input_type {
        InputType::SingleM4B(path) | InputType::SingleAudioFile(path) => Some(path.clone()),
        InputType::Directory(files) => files.first().cloned(),
    };
    let input_params = if let Some(path) = first_file {
        probe_audio_file(&path).ok()
    } else {
        None
    };
    
    match input_type {
        InputType::SingleM4B(ref path) | InputType::SingleAudioFile(ref path) => {
            println!("[DEBUG] Using single file input: {}", path);
            cmd.arg("-i").arg(path);
        }
        InputType::Directory(ref files) => {
            println!("[DEBUG] Using directory input with {} files", files.len());
            let method = concat_method.as_ref().unwrap();
            
            match method {
                ConcatMethod::Demuxer => {
                    let concat_file = create_concat_file(files, temp_dir.path())?;
                    println!("[DEBUG] Using concat demuxer with file: {:?}", concat_file);
                    cmd.arg("-f").arg("concat")
                        .arg("-safe").arg("0")
                        .arg("-i").arg(&concat_file);
                }
                ConcatMethod::FilterComplex => {
                    // Add all files as inputs
                    println!("[DEBUG] Using filter_complex with {} input files", files.len());
                    for file in files {
                        println!("[DEBUG] Adding input file: {}", file);
                        cmd.arg("-i").arg(file);
                    }
                    
                    // Check if we need filter script (for long file lists)
                    if files.len() > 50 {
                        let script_path = create_filter_complex_script(files, temp_dir.path(), config.processing_options.normalize_volume)?;
                        cmd.arg("-filter_complex_script").arg(&script_path);
                    } else {
                        // Build inline filter_complex
                        let inputs: Vec<String> = (0..files.len())
                            .map(|i| format!("[{}:a]", i))
                            .collect();
                        let mut filter = format!("{}concat=n={}:v=0:a=1", 
                            inputs.join(""), files.len());
                        
                        if config.processing_options.normalize_volume {
                            filter.push_str(",speechnorm=e=6.5:r=0.0001:l=1");
                        }
                        
                        filter.push_str("[outa]");
                        cmd.arg("-filter_complex").arg(&filter);
                    }
                }
            }
        }
    }
    
    // Add metadata file
    let metadata_content = generate_ffmetadata(&config.book_metadata, &config.chapters);
    let metadata_path = write_metadata_file(&metadata_content, temp_dir.path())?;
    cmd.arg("-i").arg(&metadata_path);
    
    // Add cover image
    if let Some(ref cover_path) = config.cover_image_path {
        // Scale if needed
        let cover_path = scale_cover_image(cover_path, config.processing_options.max_cover_size, temp_dir.path()).await?;
        cmd.arg("-i").arg(&cover_path);
    }
    
    // Set mapping
    let use_filter_complex = match &concat_method {
        Some(ConcatMethod::FilterComplex) => true,
        _ => false,
    };
    
    if use_filter_complex {
        cmd.arg("-map").arg("[outa]");
    } else {
        cmd.arg("-map").arg("0:a");
    }
    // Chapters are automatically embedded from the metadata file input
    // Cover image index: with filter_complex we have [0..N-1]=audio, N=metadata, N+1=cover
    if config.cover_image_path.is_some() {
        let cover_index = if use_filter_complex {
            match &input_type {
                InputType::Directory(files) => files.len() + 1, // audio files + metadata
                _ => 2,
            }
        } else {
            2 // single/concat input=0, metadata=1, cover=2
        };
        cmd.arg("-map").arg(format!("{}:v", cover_index));
    }
    
    // Audio codec and bitrate
    let ffmpeg_codec = match config.audio_codec.as_str() {
        "copy" => "copy",
        "opus" => "libopus",
        _ => "aac",
    };
    cmd.arg("-c:a").arg(ffmpeg_codec);
    
    if ffmpeg_codec != "copy" {
        if let Some(ref bitrate) = config.audio_bitrate {
            cmd.arg("-b:a").arg(bitrate);
        } else if let Some(ref params) = input_params {
            // Match input bitrate if available, otherwise default to a sensible value
            if let Some(br) = params.bitrate {
                cmd.arg("-b:a").arg(format!("{}b", br));
            } else {
                cmd.arg("-b:a").arg("128k");
            }
        } else {
            cmd.arg("-b:a").arg("128k"); // Final fallback
        }
        
        // Channels: use config, or match input, or default to stereo
        if let Some(channels) = config.audio_channels {
            cmd.arg("-ac").arg(channels.to_string());
        } else if let Some(ref params) = input_params {
            cmd.arg("-ac").arg(params.channels.to_string());
        } else {
            cmd.arg("-ac").arg("2");
        }
    }
    
    // Cover image options
    if config.cover_image_path.is_some() {
        cmd.arg("-c:v").arg("copy");
        cmd.arg("-disposition:v").arg("attached_pic");
        // Ensure cover image doesn't impact bitrate calculations in some players
        cmd.arg("-b:v").arg("0");
    }
    
    // Volume normalization (only if not using filter_complex)
    if config.processing_options.normalize_volume && !use_filter_complex {
        cmd.arg("-af").arg("speechnorm=e=6.5:r=0.0001:l=1");
    }
    
    // Add metadata flags (redundancy for compatibility)
    add_metadata_args(&mut cmd, &config.book_metadata);
    
    // Output path (use .tmp for atomic write)
    let output_path = if config.processing_options.atomic_write {
        get_atomic_temp_path(&config.output_path)
    } else {
        config.output_path.clone()
    };
    
    // Ensure output directory exists
    if let Some(parent) = std::path::Path::new(&output_path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create output directory: {}", parent.display()))?;
        }
    }
    
    println!("[DEBUG] Output path: {}", output_path);
    cmd.arg(&output_path);
    
    // Suppress output
    cmd.stdout(std::process::Stdio::null());
    cmd.stderr(std::process::Stdio::piped());
    
    Ok((cmd, concat_method))
}

/// Add metadata flags to command
fn add_metadata_args(cmd: &mut TokioCommand, book: &BookMetadata) {
    if !book.title.is_empty() {
        cmd.arg("-metadata").arg(&format!("title={}", book.title));
    }
    if !book.author.is_empty() {
        cmd.arg("-metadata").arg(&format!("artist={}", book.author));
    }
    if let Some(ref series) = book.series {
        cmd.arg("-metadata").arg(&format!("album={}", series));
    }
    if let Some(ref genre) = book.genre {
        cmd.arg("-metadata").arg(&format!("genre={}", genre));
    }
    if let Some(ref year) = book.publish_year {
        cmd.arg("-metadata").arg(&format!("date={}", year));
    }
    if let Some(ref publisher) = book.publisher {
        cmd.arg("-metadata").arg(&format!("publisher={}", publisher));
    }
    if let Some(ref description) = book.description {
        cmd.arg("-metadata").arg(&format!("comment={}", description));
    }
    if let Some(ref narrator) = book.narrator {
        cmd.arg("-metadata").arg(&format!("narrator={}", narrator));
    }
}

/// Generate a temporary path for atomic write that preserves the original extension
fn get_atomic_temp_path(output_path: &str) -> String {
    let path = Path::new(output_path);
    if let (Some(stem), Some(ext)) = (path.file_stem(), path.extension()) {
        let mut tmp_name = stem.to_os_string();
        tmp_name.push(".tmp.");
        tmp_name.push(ext);
        path.with_file_name(tmp_name).to_string_lossy().to_string()
    } else {
        format!("{}.tmp", output_path)
    }
}

// Phase 4: FFmpeg Execution

/// Execute FFmpeg conversion
pub async fn execute_ffmpeg(mut cmd: TokioCommand) -> Result<()> {
    // Capture stderr to see what went wrong
    cmd.stderr(std::process::Stdio::piped());
    
    let mut child = cmd.spawn()
        .context("Failed to spawn ffmpeg process")?;
    
    // Capture stderr output
    let stderr = child.stderr.take();
    let stderr_handle = if let Some(mut stderr) = stderr {
        Some(tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let reader = BufReader::new(&mut stderr);
            let mut lines = reader.lines();
            let mut output = Vec::new();
            while let Ok(Some(line)) = lines.next_line().await {
                eprintln!("[FFmpeg stderr] {}", line);
                output.push(line);
            }
            output.join("\n")
        }))
    } else {
        None
    };
    
    let status = child.wait().await
        .context("Failed to wait for ffmpeg process")?;
    
    let stderr_output = if let Some(handle) = stderr_handle {
        handle.await.unwrap_or_default()
    } else {
        String::new()
    };
    
    if status.success() {
        Ok(())
    } else {
        let error_msg = if !stderr_output.is_empty() {
            format!("FFmpeg conversion failed with exit code: {:?}\nStderr:\n{}", status.code(), stderr_output)
        } else {
            format!("FFmpeg conversion failed with exit code: {:?}", status.code())
        };
        Err(anyhow::anyhow!("{}", error_msg))
    }
}

/// Main conversion function
pub async fn convert_to_m4b(config: ConversionConfig) -> Result<String> {
    println!("[DEBUG] Starting conversion:");
    println!("[DEBUG]   Input: {}", config.input_path);
    println!("[DEBUG]   Output: {}", config.output_path);
    println!("[DEBUG]   Chapters: {}", config.chapters.len());
    println!("[DEBUG]   Cover: {:?}", config.cover_image_path);
    
    // Create temporary directory
    let temp_dir = TempDir::new()
        .context("Failed to create temporary directory")?;
    println!("[DEBUG] Temp directory: {:?}", temp_dir.path());
    
    // Build command
    let (cmd, _concat_method) = build_ffmpeg_command(&config, &temp_dir).await?;
    
    // Debug: Print the command being executed (approximate)
    println!("[DEBUG] FFmpeg command built (check stderr for full command)");
    
    // Execute conversion
    execute_ffmpeg(cmd).await?;
    
    // Atomic write: rename .tmp to final name
    if config.processing_options.atomic_write {
        let tmp_path = get_atomic_temp_path(&config.output_path);
        fs::rename(&tmp_path, &config.output_path)
            .with_context(|| format!("Failed to rename {} to {}", tmp_path, config.output_path))?;
    }
    
    // Validate output
    let output_path = Path::new(&config.output_path);
    if !output_path.exists() {
        return Err(anyhow::anyhow!("Output file was not created: {}", config.output_path));
    }
    
    let metadata = fs::metadata(&output_path)
        .context("Failed to get output file metadata")?;
    if metadata.len() == 0 {
        return Err(anyhow::anyhow!("Output file is empty: {}", config.output_path));
    }
    
    // Temp dir will be cleaned up automatically on drop
    Ok(config.output_path)
}
