# Rovr

A Rust-based Nostr bot that downloads and converts YouTube videos to audio format.

## Prerequisites

- Rust (latest stable version)
- FFmpeg (version 7.1.1 or compatible)
- Python 3.x (for yt-dlp)
- Virtual environment (recommended)

## Installation

1. Clone the repository:
```bash
git clone https://github.com/EthnTuttle/rovr.git
cd rovr
```

2. Install Python dependencies:
```bash
python3 -m venv venv
source venv/bin/activate
pip install yt-dlp
```

3. Build the project:
```bash
cargo build
```

## Configuration

The application can be configured using the `config.toml` file:

```toml
[bot]
name = "YouTube Downloader Bot"  # Bot's display name
allowed_pubkey = "npub..."       # Nostr public key allowed to use the bot
nip05 = "your@nip05.id"         # NIP-05 identifier

[relays]
urls = [                         # List of Nostr relays to connect to
    "wss://relay.damus.io",
    "wss://nostr.wine",
    "wss://relay.nostr.band"
]

[downloads]
format = "aac"                   # Output audio format (aac, mp3, etc.)
quality = "0"                    # Audio quality (0 = best)
```

## Usage

1. Start the bot:
```bash
cargo run
```

2. The bot will display its public key (npub) and a QR code. Save this information.

3. Send a direct message to the bot on Nostr with a YouTube URL. The bot will:
   - Download the video
   - Convert it to the configured audio format
   - Save it to the downloads directory
   - Send you a confirmation message

4. Find your downloaded files in:
   - Linux: `~/.local/share/rovr/downloads/`
   - macOS: `~/Library/Application Support/rovr/downloads/`
   - Windows: `%APPDATA%\rovr\downloads\`

## Features

- Secure direct messaging using NIP-04 encryption
- Automatic YouTube URL detection
- Configurable audio format and quality
- Multiple relay support
- Persistent key storage
- QR code generation for easy bot identification

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 