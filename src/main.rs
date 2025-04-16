use anyhow::Result;
use nostr::prelude::*;
use nostr_sdk::prelude::*;
use regex::Regex;
use std::process::Command;
use std::fs;
use std::path::PathBuf;
use tokio::time::{sleep, Duration};
use qrcode::QrCode;
use qrcode::render::unicode;
use log::{info, error};
use directories::ProjectDirs;

const YOUTUBE_URL_PATTERN: &str = r"(?:https?://)?(?:www\.)?(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})";
const ALLOWED_PUBKEY: &str = "npub160t5zfxalddaccdc7xx30sentwa5lrr3rq4rtm38x99ynf8t0vwsvzyjc9";

fn get_keys() -> Result<Keys> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "rovr", "rovr") {
        let data_dir = proj_dirs.data_dir();
        fs::create_dir_all(data_dir)?;
        
        let keys_path = data_dir.join("keys.json");
        
        if keys_path.exists() {
            info!("Loading existing keys from {}", keys_path.display());
            let keys_str = fs::read_to_string(&keys_path)?;
            let keys = Keys::from_sk_str(&keys_str)?;
            Ok(keys)
        } else {
            info!("Generating new keys and saving to {}", keys_path.display());
            let keys = Keys::generate();
            let sk = keys.secret_key()?.to_bech32()?;
            fs::write(keys_path, sk)?;
            Ok(keys)
        }
    } else {
        error!("Could not determine application data directory");
        Ok(Keys::generate())
    }
}

fn print_qr_code(text: &str) {
    let code = QrCode::new(text.as_bytes()).unwrap();
    let qr_string = code
        .render::<unicode::Dense1x2>()
        .dark_color(unicode::Dense1x2::Light)
        .light_color(unicode::Dense1x2::Dark)
        .build();
    println!("\nQR Code for npub:\n{}", qr_string);
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with INFO level by default
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting YouTube DM bot...");

    // Load or generate keys
    let keys = get_keys()?;
    let bot_npub = keys.public_key().to_bech32()?;
    println!("\n=== BOT IDENTITY ===");
    println!("npub: {}", bot_npub);
    println!("===================\n");
    info!("Bot public key: {}", bot_npub);
    print_qr_code(&bot_npub);

    // Create a new client
    let client = Client::new(&keys);
    info!("Created new Nostr client");

    // Add relays
    info!("Connecting to relays...");
    client.add_relay("wss://relay.damus.io", None).await?;
    client.add_relay("wss://nostr.wine", None).await?;
    client.add_relay("wss://relay.nostr.band", None).await?;
    info!("Added all relays");

    // Connect to relays
    client.connect().await;
    info!("Connected to all relays");

    // Convert the allowed pubkey to XOnlyPublicKey
    let allowed_pubkey = XOnlyPublicKey::from_bech32(ALLOWED_PUBKEY)?;
    info!("Converted allowed pubkey: {}", ALLOWED_PUBKEY);

    // Create a subscription for DMs from the specific pubkey
    let subscription = Filter::new()
        .kinds(vec![Kind::EncryptedDirectMessage]) // NIP-04 encrypted DMs
        .pubkey(keys.public_key())
        .authors(vec![allowed_pubkey])
        .since(Timestamp::now());

    // Subscribe to DMs
    client.subscribe(vec![subscription]).await;
    info!("Subscribed to DMs from {}", ALLOWED_PUBKEY);

    // Create regex for YouTube URLs
    let youtube_regex = Regex::new(YOUTUBE_URL_PATTERN).unwrap();
    info!("Initialized YouTube URL regex");

    info!("Bot is running and listening for DMs from {}", ALLOWED_PUBKEY);

    // Listen for events
    let mut notifications = client.notifications();
    while let Ok(notification) = notifications.recv().await {
        if let RelayPoolNotification::Event(_url, event) = notification {
            if event.kind == Kind::EncryptedDirectMessage {
                // Decrypt the message
                let decrypted_content = nip04::decrypt(
                    &keys.secret_key()?,
                    &event.pubkey,
                    &event.content,
                )?;
                info!("Received DM from {}: {}", event.pubkey.to_bech32()?, decrypted_content);
                
                // Check if the message contains a YouTube URL
                if let Some(captures) = youtube_regex.captures(&decrypted_content) {
                    if let Some(video_id) = captures.get(1) {
                        info!("Found YouTube video ID: {}", video_id.as_str());
                        let youtube_url = format!("https://youtube.com/watch?v={}", video_id.as_str());
                        
                        // Download and convert to MP3
                        let output = Command::new("yt-dlp")
                            .args([
                                "-x", // Extract audio
                                "--audio-format", "mp3", // Convert to MP3
                                "--audio-quality", "0", // Best quality
                                "-o", "downloads/%(title)s.%(ext)s", // Output format
                                &youtube_url,
                            ])
                            .output()?;

                        if output.status.success() {
                            info!("Successfully downloaded and converted video");
                            // Send success message with YouTube link
                            let response = format!("Successfully downloaded and converted the video to MP3!\n\nYouTube: {}", youtube_url);
                            let encrypted_content = nip04::encrypt(
                                &keys.secret_key()?,
                                &event.pubkey,
                                response,
                            )?;
                            let response_event = EventBuilder::new(
                                Kind::EncryptedDirectMessage,
                                encrypted_content,
                                &[Tag::parse(vec!["p", &event.pubkey.to_string()]).unwrap()],
                            ).to_event(&keys)?;
                            
                            client.send_event(response_event).await?;
                            info!("Sent success response to user");
                        } else {
                            error!("Failed to download video. Error: {:?}", String::from_utf8_lossy(&output.stderr));
                            // Send error message with YouTube link
                            let error_msg = format!("Failed to download and convert the video.\n\nYouTube: {}\n\nPlease try again later.", youtube_url);
                            let encrypted_content = nip04::encrypt(
                                &keys.secret_key()?,
                                &event.pubkey,
                                error_msg,
                            )?;
                            let error_event = EventBuilder::new(
                                Kind::EncryptedDirectMessage,
                                encrypted_content,
                                &[Tag::parse(vec!["p", &event.pubkey.to_string()]).unwrap()],
                            ).to_event(&keys)?;
                            
                            client.send_event(error_event).await?;
                            info!("Sent error response to user");
                        }
                    }
                } else {
                    info!("Message did not contain a YouTube URL");
                    // Send message back to user about invalid YouTube URL
                    let error_msg = "Sorry, I couldn't find a valid YouTube URL in your message. Please send a valid YouTube link.";
                    let encrypted_content = nip04::encrypt(
                        &keys.secret_key()?,
                        &event.pubkey,
                        error_msg,
                    )?;
                    let error_event = EventBuilder::new(
                        Kind::EncryptedDirectMessage,
                        encrypted_content,
                        &[Tag::parse(vec!["p", &event.pubkey.to_string()]).unwrap()],
                    ).to_event(&keys)?;
                    
                    client.send_event(error_event).await?;
                    info!("Sent invalid URL response to user");
                }
            }
        }
    }

    Ok(())
}
