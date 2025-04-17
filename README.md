# Rovr

A Rust-based Nostr bot that downloads and converts YouTube videos to MP3 format. The bot listens for direct messages containing YouTube URLs from authorized users and automatically downloads and converts them in parallel.

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
source venv/bin/activate  # On Windows: .\venv\Scripts\activate
pip install yt-dlp
```

3. Build the project:
```bash
cargo build
```

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

## Performance

- Downloads are processed concurrently in separate tasks
- The bot remains responsive while processing multiple downloads
- Failed downloads don't affect other ongoing conversions
- Memory-efficient task management using Tokio runtime

## Security

- The bot only responds to messages from pubkeys listed in `allowed_pubkeys`
- All direct messages are encrypted using NIP-04
- Keys are stored securely in the user's application data directory
- No API keys or sensitive information are stored in the code
- Downloads are isolated to prevent interference

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details. 