# Default recipe
default:
    @just --list

# Development tasks
dev:
    cargo run

build:
    cargo build

release:
    cargo build --release

test:
    cargo test

clean:
    cargo clean
    rm -rf downloads/
    rm -rf venv/
    rm -rf ffmpeg*/

# Installation tasks
install:
    chmod +x install.sh
    ./install.sh

install-service:
    chmod +x install-service.sh
    sudo ./install-service.sh

uninstall-service:
    chmod +x uninstall-service.sh
    sudo ./uninstall-service.sh

update:
    chmod +x update.sh
    sudo ./update.sh

# Nix tasks
nix-shell:
    nix-shell

# Service management tasks
start:
    sudo systemctl start rovr

stop:
    sudo systemctl stop rovr

restart:
    sudo systemctl restart rovr

status:
    sudo systemctl status rovr

logs:
    sudo journalctl -u rovr -f

# Configuration tasks
edit-config:
    nano config.toml

create-config:
    echo "Creating default config.toml..."
    echo '[bot]' > config.toml
    echo 'name = "YouTube Downloader Bot"' >> config.toml
    echo 'allowed_pubkeys = []' >> config.toml
    echo 'nip05 = ""' >> config.toml
    echo '' >> config.toml
    echo '[relays]' >> config.toml
    echo 'urls = [' >> config.toml
    echo '    "wss://relay.damus.io",' >> config.toml
    echo '    "wss://nostr.wine",' >> config.toml
    echo '    "wss://relay.nostr.band"' >> config.toml
    echo ']' >> config.toml
    echo '' >> config.toml
    echo '[downloads]' >> config.toml
    echo 'format = "mp3"' >> config.toml
    echo 'quality = "0"' >> config.toml 