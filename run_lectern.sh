#!/bin/bash

# Set Qt5 environment variables for Homebrew
export LDFLAGS="-L/home/linuxbrew/.linuxbrew/opt/qt@5/lib"
export CPPFLAGS="-I/home/linuxbrew/.linuxbrew/opt/qt@5/include"
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/opt/qt@5/lib/pkgconfig"

# Set Qt platform and plugin path
export QT_QPA_PLATFORM="wayland;xcb"
export QT_QPA_PLATFORM_PLUGIN_PATH="/home/linuxbrew/.linuxbrew/Cellar/qt@5/5.15.18/plugins/platforms"
export QT_PLUGIN_PATH="/home/linuxbrew/.linuxbrew/Cellar/qt@5/5.15.18/plugins"

# Run the application
if [ "$1" = "--cli" ]; then
    cargo run -- --cli
else
    cargo run
fi