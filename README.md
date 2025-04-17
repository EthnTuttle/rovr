# Rovr

⚠️ **VIBE CODED WARNING** ⚠️

This project is vibe-coded, which means:
- It works on my machine (and hopefully yours)
- The code is more art than science
- Documentation may be vibes-based
- Updates may break things in unexpected ways
- You're welcome to contribute, but expect vibes

If you're looking for enterprise-grade, production-ready code, this might not be the project for you. If you're cool with vibes, welcome aboard! 🎵

---

A Rust-based Nostr bot that downloads and converts YouTube videos to MP3 format. The bot listens for direct messages containing YouTube URLs from authorized users and automatically downloads and converts them in parallel.

## Installation

### Option 1: Using Nix (Recommended)

Nix provides a complete, reproducible development environment with all dependencies. This is the recommended way to install Rovr as it ensures consistent behavior across different systems.

1. Install Nix using the Determinate Systems installer:
```bash
# On Linux/macOS
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install

# On Windows (WSL2)
wsl --install
curl --proto '=https' --tlsv1.2 -sSf -L https://install.determinate.systems/nix | sh -s -- install
```

2. Clone the repository and enter the development environment:
```bash
git clone https://github.com/EthnTuttle/rovr.git
cd rovr
nix-shell
```

3. Build and run:
```bash
cargo build
cargo run
```

### Option 2: Using the Install Script

If you prefer not to use Nix, you can use the provided install script. This is simpler but may require manual dependency management.

1. Clone the repository:
```bash
git clone https://github.com/EthnTuttle/rovr.git
cd rovr
```

2. Run the installation script:
```bash
chmod +x install.sh
./install.sh
```

The script will check for dependencies, set up the environment, and create a default config file.

### Running as a System Service (Linux)

To run Rovr as a system service on Linux:

1. Set up Python virtual environment and install yt-dlp:
```bash
python3 -m venv venv
source venv/bin/activate
pip install yt-dlp
```

2. Build the release version:
```bash
cargo build --release
```

3. Install the systemd service:
```bash
chmod +x install-service.sh
sudo ./install-service.sh
```

The script will:
- Install the binary to `/usr/local/bin`
- Set up the systemd service
- Ask if you want to start the service
- Ask if you want to enable the service on boot

#### Important Path Configuration

The bot requires yt-dlp to be available in a known location. We handle this in two places:

1. In the systemd service file (`rovr.service`):
```ini
[Service]
User=ethan  # Change this to your username
WorkingDirectory=/home/ethan/code/rovr  # Change to your project directory
Environment="PATH=/home/ethan/code/rovr/venv/bin:/usr/local/bin:/usr/bin:/bin"
```

2. In the Rust code (`src/main.rs`):
```rust
let yt_dlp_path = PathBuf::from("/home/ethan/code/rovr/venv/bin/yt-dlp");
```

This setup ensures that:
- The service runs in your project directory where the virtual environment is located
- The PATH includes the virtual environment's bin directory first
- The code knows exactly where to find yt-dlp
- Dependencies are isolated and version-controlled in your project's virtual environment

Make sure to update both paths to match your system's directory structure and username.

Common systemd commands:
- Start service: `sudo systemctl start rovr`
- Stop service: `sudo systemctl stop rovr`
- Restart service: `sudo systemctl restart rovr`
- Enable on boot: `sudo systemctl enable rovr`
- Disable on boot: `sudo systemctl disable rovr`
- View logs: `sudo journalctl -u rovr -f`
- Check status: `sudo systemctl status rovr`

### Uninstalling the Service

To uninstall the system service:

1. Run the uninstall script:
```bash
chmod +x uninstall-service.sh
sudo ./uninstall-service.sh
```

The script will:
- Stop and disable the service
- Remove the binary from `/usr/local/bin`
- Remove the systemd service file
- Ask if you want to remove the data directory

## Configuration

The application is configured using the `config.toml` file:

```toml
[bot]
name = "YouTube Downloader Bot"  # Bot's display name
allowed_pubkeys = [              # List of Nostr public keys allowed to use the bot
    "npub1...",
    "npub2..."
]
nip05 = "your@nip05.id"         # NIP-05 identifier (optional)

[relays]
urls = [                        # List of Nostr relays to connect to
    "wss://relay.damus.io",
    "wss://nostr.wine",
    "wss://relay.nostr.band"
]

[downloads]
format = "mp3"                  # Output audio format (mp3, aac, etc.)
quality = "0"                   # Audio quality (0 = best)
```

### Dynamic Configuration

The bot checks the `config.toml` file every 5 seconds for changes to the `allowed_pubkeys` list. This allows you to add or remove authorized users without restarting the bot.

## Usage

1. Start the bot:
```bash
cargo run
```

2. The bot will display its public key (npub) and a QR code. Save this information.

3. Send a direct message to the bot on Nostr with a YouTube URL. The bot will:
   - Verify your pubkey is in the allowed list
   - Download the video
   - Convert it to MP3 format
   - Save it to the downloads directory
   - Send you a confirmation message

4. Find your downloaded files in:
   - Linux: `~/.local/share/rovr/downloads/`
   - macOS: `~/Library/Application Support/rovr/downloads/`
   - Windows: `%APPDATA%\rovr\downloads\`

## Features

- Secure direct messaging using NIP-04 encryption
- Dynamic authorized user list (updates every 5 seconds)
- Automatic YouTube URL detection
- High-quality MP3 conversion
- Concurrent downloads (multiple videos processed simultaneously)
- Multiple relay support
- Persistent key storage
- QR code generation for easy bot identification
- Cross-platform support

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 