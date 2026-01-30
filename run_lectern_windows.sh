#!/bin/bash

# The wineprefix will need a sans-serif font installed and available in the font path.
# The wineprefix will need ffplay, ffprobe and ffmpeg installed.

# Add wine staging to the path
PATH="$HOME/.local/opt/wine-staging/bin:$PATH"
# Run the Windows build under Wine. Build first with:
#   PATH="/home/linuxbrew/.linuxbrew/bin:$PATH" cargo build --release --target x86_64-pc-windows-gnu
cd "$(dirname "$0")"
EXE=./target/x86_64-pc-windows-gnu/release/lectern.exe
if [[ ! -f $EXE ]]; then
  echo "Not found: $EXE. Build for Windows first." >&2
  exit 1
fi
# FFmpeg/ffprobe for chapters and convert: run ./setup_ffmpeg_wine.sh once (downloads Windows build into prefix).
WINEPREFIX=${WINEPREFIX:-$HOME/Documents/lectern-wineprefix}
# So Wine sees ffmpeg/bin immediately without relying on registry refresh
[[ -d "$WINEPREFIX/drive_c/ffmpeg/bin" ]] && PATH="$PATH:$WINEPREFIX/drive_c/ffmpeg/bin"
export WINEPREFIX
wine "$EXE"