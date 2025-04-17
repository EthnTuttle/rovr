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

# Stop and disable the service if it's running
if systemctl is-active --quiet rovr; then
    echo "Stopping rovr service..."
    systemctl stop rovr
    systemctl disable rovr
fi

# Remove the service file
if [ -f "/etc/systemd/system/rovr.service" ]; then
    echo "Removing systemd service..."
    rm /etc/systemd/system/rovr.service
    systemctl daemon-reload
fi

# Remove the binary
if [ -f "/usr/local/bin/rovr" ]; then
    echo "Removing binary..."
    rm /usr/local/bin/rovr
fi

# Ask about removing the data directory
if read_timeout "Do you want to remove the data directory (/home/$USER/.local/share/rovr)?"; then
    echo "Removing data directory..."
    rm -rf "/home/$USER/.local/share/rovr"
else
    echo "Keeping data directory."
fi

echo "Uninstallation complete!" 