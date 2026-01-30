#!/bin/bash
# Install Windows FFmpeg (ffprobe/ffmpeg) into the Lectern Wine prefix so the
# Windows build can generate chapters and convert. Uses gyan.dev essentials build.
set -e
FFMPEG_7Z_URL="https://www.gyan.dev/ffmpeg/builds/ffmpeg-release-essentials.7z"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WINEPREFIX="${WINEPREFIX:-$HOME/Documents/lectern-wineprefix}"
FFMPEG_DEST="$WINEPREFIX/drive_c/ffmpeg"
PATH="$HOME/.local/opt/wine-staging/bin:$PATH"

echo "Wine prefix: $WINEPREFIX"
echo "FFmpeg destination: $FFMPEG_DEST"

# Download 7z if not present
ARCHIVE="$SCRIPT_DIR/ffmpeg-release-essentials.7z"
if [[ ! -f "$ARCHIVE" ]]; then
  echo "Downloading $FFMPEG_7Z_URL ..."
  curl -L -o "$ARCHIVE" "$FFMPEG_7Z_URL"
fi

# Extract (need 7z: p7zip-full or p7zip-plugins)
EXTRACT_DIR="$SCRIPT_DIR/ffmpeg-extract"
rm -rf "$EXTRACT_DIR"
mkdir -p "$EXTRACT_DIR"
if command -v 7z &>/dev/null; then
  7z x -o"$EXTRACT_DIR" "$ARCHIVE" -y
elif command -v 7za &>/dev/null; then
  7za x -o"$EXTRACT_DIR" "$ARCHIVE" -y
else
  echo "Error: 7z or 7za not found. Install p7zip-full or p7zip-plugins." >&2
  exit 1
fi

# Find bin folder (archive has one top-level dir like ffmpeg-7.x-essentials_build)
BIN_SRC=$(find "$EXTRACT_DIR" -type d -name bin -path '*/bin' | head -1)
if [[ -z "$BIN_SRC" ]] || [[ ! -f "$BIN_SRC/ffprobe.exe" ]]; then
  echo "Error: could not find ffprobe.exe in extracted archive." >&2
  ls -laR "$EXTRACT_DIR" | head -80
  exit 1
fi

mkdir -p "$FFMPEG_DEST/bin"
cp -a "$BIN_SRC"/* "$FFMPEG_DEST/bin/"
echo "Copied FFmpeg binaries to $FFMPEG_DEST/bin"
rm -rf "$EXTRACT_DIR"

# Symlink into system32 so Windows apps find ffprobe/ffmpeg/ffplay without relying on PATH refresh
SYSTEM32="$WINEPREFIX/drive_c/windows/system32"
mkdir -p "$SYSTEM32"
ln -sf "$FFMPEG_DEST/bin/ffprobe.exe" "$SYSTEM32/ffprobe.exe"
ln -sf "$FFMPEG_DEST/bin/ffmpeg.exe" "$SYSTEM32/ffmpeg.exe"
if [[ -f "$FFMPEG_DEST/bin/ffplay.exe" ]]; then
  ln -sf "$FFMPEG_DEST/bin/ffplay.exe" "$SYSTEM32/ffplay.exe"
  echo "Symlinked ffprobe.exe, ffmpeg.exe, and ffplay.exe into C:\\windows\\system32 (chapters + conversion + playback)."
else
  echo "Symlinked ffprobe.exe and ffmpeg.exe (ffplay.exe not in build; chapter playback under Wine needs ffmpeg full build)."
fi

# Add C:\ffmpeg\bin to Wine user PATH for persistence; then sync so registry is visible to new processes
CURRENT=$(wine reg query 'HKCU\Environment' /v Path 2>/dev/null | awk '/REG_EXPAND_SZ/ { for(i=4;i<=NF;i++) printf "%s%s", $i, (i<NF?OFS:ORS); }' | sed 's/^[[:space:]]*//;s/[[:space:]]*$//')
if [[ -n "$CURRENT" ]]; then
  NEW_PATH="C:\\ffmpeg\\bin;$CURRENT"
else
  NEW_PATH="C:\\ffmpeg\\bin"
fi
wine reg add 'HKCU\Environment' /v Path /t REG_EXPAND_SZ /d "$NEW_PATH" /f >/dev/null 2>&1
wine wineboot -u 2>/dev/null || true
echo "Updated Wine user PATH and synced (wineboot -u). Lectern can use ffprobe/ffmpeg now."
