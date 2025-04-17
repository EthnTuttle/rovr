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

# Get the application data directory (matching ProjectDirs::from("com", "rovr", "rovr"))
if [ -z "$XDG_DATA_HOME" ]; then
    DATA_DIR="$HOME/.local/share/rovr"
else
    DATA_DIR="$XDG_DATA_HOME/rovr"
fi

# Create the application data directory and its parent directories
echo "Creating application data directory..."
mkdir -p "$DATA_DIR"

# Function to read input with default value and validation
read_with_default() {
    local prompt="$1"
    local default="$2"
    local validation="$3"  # Optional validation function name
    local input
    
    while true; do
        echo -n "$prompt [$default]: "
        read input
        input="${input:-$default}"
        
        if [ -n "$validation" ] && ! $validation "$input"; then
            echo "Invalid input. Please try again."
            continue
        fi
        
        echo "$input"
        break
    done
}

# Function to read multiple values with validation
read_multiple() {
    local prompt="$1"
    local validation="$2"  # Optional validation function name
    local values=()
    local input
    
    echo "$prompt (press Enter twice to finish):"
    while true; do
        read input
        if [ -z "$input" ]; then
            break
        fi
        
        if [ -n "$validation" ] && ! $validation "$input"; then
            echo "Invalid input. Please try again."
            continue
        fi
        
        values+=("$input")
    done
    echo "${values[@]}"
}

# Validation functions
validate_npub() {
    [[ "$1" =~ ^npub[0-9a-zA-Z]{58}$ ]]
}

validate_relay() {
    [[ "$1" =~ ^wss?://.+$ ]]
}

validate_format() {
    [[ "$1" =~ ^(mp3|aac|m4a|opus|vorbis|flac)$ ]]
}

validate_quality() {
    [[ "$1" =~ ^[0-9]$ ]]
}

# Gather configuration information with validation
echo -e "\n=== Bot Configuration ==="
BOT_NAME=$(read_with_default "Enter bot name" "YouTube Downloader Bot")
echo "Enter allowed public keys (npub format):"
DEFAULT_PUBKEYS=("npub160t5zfxalddaccdc7xx30sentwa5lrr3rq4rtm38x99ynf8t0vwsvzyjc9")
ALLOWED_PUBKEYS=($(read_multiple "Enter a public key" validate_npub))

# If no pubkeys were entered, use defaults
if [ ${#ALLOWED_PUBKEYS[@]} -eq 0 ]; then
    ALLOWED_PUBKEYS=("${DEFAULT_PUBKEYS[@]}")
fi

NIP05=$(read_with_default "Enter NIP-05 identifier (optional)" "")

echo -e "\n=== Relay Configuration ==="
echo "Enter relay URLs (press Enter twice to finish):"
DEFAULT_RELAYS=("wss://relay.damus.io" "wss://nostr.wine" "wss://relay.nostr.band")
RELAY_URLS=($(read_multiple "Enter a relay URL" validate_relay))

# If no relays were entered, use defaults
if [ ${#RELAY_URLS[@]} -eq 0 ]; then
    RELAY_URLS=("${DEFAULT_RELAYS[@]}")
fi

echo -e "\n=== Download Configuration ==="
FORMAT=$(read_with_default "Enter audio format (mp3, aac, m4a, opus, vorbis, flac)" "mp3" validate_format)
QUALITY=$(read_with_default "Enter audio quality (0-9, 0 = best)" "0" validate_quality)

# Create the config file in the application data directory
echo "Creating configuration file..."
cat > "$DATA_DIR/config.toml" << EOL
[bot]
name = "$BOT_NAME"
allowed_pubkeys = [
    $(printf '"%s",\n' "${ALLOWED_PUBKEYS[@]}" | sed '$s/,$//')
]
nip05 = "$NIP05"

[relays]
urls = [
    $(printf '"%s",\n' "${RELAY_URLS[@]}" | sed '$s/,$//')
]

[downloads]
format = "$FORMAT"
quality = "$QUALITY"
EOL

# Create downloads directory
echo "Creating downloads directory..."
mkdir -p "$DATA_DIR/downloads"

echo "Installation complete!"
echo "Configuration saved to: $DATA_DIR/config.toml"
echo "Downloads will be saved to: $DATA_DIR/downloads"
echo "To run the bot, use: cargo run" 