#!/bin/bash

# Run the application
if [ "$1" = "--cli" ]; then
    cargo run -- --cli
else
    cargo run
fi