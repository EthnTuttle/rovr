#!/bin/bash

# Exit on error
set -e

echo "Installing Rovr - YouTube Downloader Bot for Nostr"

# Check for required commands
echo "Checking for required commands..."
for cmd in rustc cargo python3 pip; do
    if ! command -v $cmd &> /dev/null; then
        echo "Error: $cmd is required but not installed."
        exit 1
    fi
done

# Check for FFmpeg
if ! command -v ffmpeg &> /dev/null; then
    echo "Error: FFmpeg is required but not installed."
    echo "Please install FFmpeg using your package manager:"
    echo "  Ubuntu/Debian: sudo apt install ffmpeg"
    echo "  Fedora: sudo dnf install ffmpeg"
    echo "  macOS: brew install ffmpeg"
    exit 1
fi

# Create and activate Python virtual environment
echo "Setting up Python virtual environment..."
python3 -m venv venv
source venv/bin/activate

# Install Python dependencies
echo "Installing Python dependencies..."
pip install yt-dlp

# Build the Rust project
echo "Building the Rust project..."
cargo build --release

# Create default config file if it doesn't exist
if [ ! -f config.toml ]; then
    echo "Creating default config file..."
    cat > config.toml << EOL
[bot]
name = "YouTube Downloader Bot"
allowed_pubkeys = []
nip05 = ""

[relays]
urls = [
    "wss://relay.damus.io",
    "wss://nostr.wine",
    "wss://relay.nostr.band"
]

[downloads]
format = "mp3"
quality = "0"
EOL
fi

echo "Installation complete!"
echo "Please edit config.toml to add your allowed pubkeys and other settings."
echo "To run the bot, use: cargo run" 