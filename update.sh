#!/bin/bash

# Exit on error
set -e

echo "Updating Rovr - YouTube Downloader Bot for Nostr"

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Get the current user
USER=$(logname)

# Function to read input with timeout and default to No
read_timeout() {
    local prompt="$1"
    local timeout=10
    local default="n"
    
    echo -n "$prompt (y/N, timeout ${timeout}s) "
    read -t $timeout -n 1 -r answer || answer=$default
    echo
    [[ $answer =~ ^[Yy]$ ]]
}

# Check if git is installed
if ! command -v git &> /dev/null; then
    echo "Error: git is required but not installed."
    exit 1
fi

# Check if we're in a git repository
if [ ! -d .git ]; then
    echo "Error: Not a git repository. Please run this script from the Rovr directory."
    exit 1
fi

# Stop the service if it's running
if systemctl is-active --quiet rovr; then
    echo "Stopping rovr service..."
    systemctl stop rovr
fi

# Backup the current configuration
echo "Backing up configuration..."
if [ -f "config.toml" ]; then
    cp config.toml config.toml.bak
fi

# Get the application data directory
if [ -z "$XDG_DATA_HOME" ]; then
    DATA_DIR="$HOME/.local/share/rovr"
else
    DATA_DIR="$XDG_DATA_HOME/rovr"
fi

# Backup the application data configuration
if [ -f "$DATA_DIR/config.toml" ]; then
    cp "$DATA_DIR/config.toml" "$DATA_DIR/config.toml.bak"
fi

# Pull the latest changes
echo "Pulling latest changes..."
git pull

# Update Python dependencies
echo "Updating Python dependencies..."
if [ -d "venv" ]; then
    source venv/bin/activate
    pip install --upgrade yt-dlp
fi

# Build the project
echo "Building the project..."
cargo build --release

# Restore configuration
echo "Restoring configuration..."
if [ -f "config.toml.bak" ]; then
    mv config.toml.bak config.toml
fi

if [ -f "$DATA_DIR/config.toml.bak" ]; then
    mv "$DATA_DIR/config.toml.bak" "$DATA_DIR/config.toml"
fi

# Update the binary if installed as a service
if [ -f "/usr/local/bin/rovr" ]; then
    echo "Updating system binary..."
    cp target/release/rovr /usr/local/bin/rovr
    chmod +x /usr/local/bin/rovr
fi

# Restart the service if it was running
if systemctl is-enabled --quiet rovr; then
    if read_timeout "Do you want to start the service now?"; then
        echo "Starting rovr service..."
        systemctl start rovr
        echo "Service started. Checking status..."
        systemctl status rovr --no-pager
    else
        echo "Skipping service start."
        echo "You can start the service manually with: sudo systemctl start rovr"
    fi
fi

echo "Update complete!"
echo "To check the service status, use: sudo systemctl status rovr"
echo "To view logs, use: sudo journalctl -u rovr -f" 