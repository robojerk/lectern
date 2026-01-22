#!/bin/bash

# Set Qt5 environment variables for Homebrew
export LDFLAGS="-L/home/linuxbrew/.linuxbrew/opt/qt@5/lib"
export CPPFLAGS="-I/home/linuxbrew/.linuxbrew/opt/qt@5/include"
export PKG_CONFIG_PATH="/home/linuxbrew/.linuxbrew/opt/qt@5/lib/pkgconfig"

# Set Qt platform
export QT_QPA_PLATFORM=xcb

# Run the application
cargo run