#!/bin/bash

# Force X11/XWayland backend by unsetting WAYLAND_DISPLAY
# This allows drag-and-drop to work on Wayland systems via XWayland
unset WAYLAND_DISPLAY

# Ensure DISPLAY is set (usually :0 or :1)
# If not set, XWayland will use the default
export DISPLAY="${DISPLAY:-:0}"

echo "Running Lectern with X11/XWayland backend"
echo "WAYLAND_DISPLAY: ${WAYLAND_DISPLAY:-(unset)}"
echo "DISPLAY: $DISPLAY"

# Run the application
cargo run
