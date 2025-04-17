#!/bin/bash

# Exit on error
set -e

# Check if running as root
if [ "$EUID" -ne 0 ]; then
    echo "Please run as root"
    exit 1
fi

# Get the current user
USER=$(logname)

# Check if the binary exists in the target directory
BINARY_PATH="target/release/rovr"
if [ ! -f "$BINARY_PATH" ]; then
    echo "Error: Binary not found at $BINARY_PATH"
    echo "Please build the release version first: cargo build --release"
    exit 1
fi

# Create necessary directories
echo "Creating directories..."
mkdir -p /usr/local/bin
mkdir -p "/home/$USER/.local/share/rovr"

# Install the binary
echo "Installing binary to /usr/local/bin..."
cp "$BINARY_PATH" /usr/local/bin/rovr
chmod +x /usr/local/bin/rovr

# Check if the service file exists
if [ ! -f "rovr.service" ]; then
    echo "Error: rovr.service file not found"
    exit 1
fi

# Create a temporary service file with the actual values
sed "s|%USER%|$USER|g" rovr.service > /tmp/rovr.service

# Install the service
echo "Installing systemd service..."
cp /tmp/rovr.service /etc/systemd/system/rovr.service
systemctl daemon-reload

echo "Service installed successfully!"

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

# Ask if user wants to start the service
if read_timeout "Do you want to start the service now?"; then
    systemctl start rovr
    echo "Service started. Checking status..."
    systemctl status rovr --no-pager
else
    echo "Skipping service start."
fi

# Ask if user wants to enable the service on boot
if read_timeout "Do you want to enable the service to start on boot?"; then
    systemctl enable rovr
    echo "Service enabled to start on boot."
else
    echo "Skipping service enable."
fi

echo -e "\nCommon systemd commands:"
echo "  Start service: sudo systemctl start rovr"
echo "  Stop service: sudo systemctl stop rovr"
echo "  Restart service: sudo systemctl restart rovr"
echo "  Enable on boot: sudo systemctl enable rovr"
echo "  Disable on boot: sudo systemctl disable rovr"
echo "  View logs: sudo journalctl -u rovr -f"
echo "  Check status: sudo systemctl status rovr" 