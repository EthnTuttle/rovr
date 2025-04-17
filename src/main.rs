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
use reqwest;
use serde_json::json;
use config::Config;
use std::sync::Arc;
use tokio::sync::RwLock;

const YOUTUBE_URL_PATTERN: &str = r"(?:https?://)?(?:www\.)?(?:youtube\.com/watch\?v=|youtu\.be/)([a-zA-Z0-9_-]{11})";

#[derive(Clone)]
struct BotConfig {
    name: String,
    allowed_pubkeys: Vec<String>,
    nip05: String,
    relays: Vec<String>,
    format: String,
    quality: String,
}

fn load_config() -> Result<BotConfig> {
    let mut builder = Config::builder();
    
    // First try to load from application data directory
    if let Some(proj_dirs) = ProjectDirs::from("com", "rovr", "rovr") {
        let config_path = proj_dirs.data_dir().join("config.toml");
        if config_path.exists() {
            builder = builder.add_source(config::File::from(config_path));
        }
    }
    
    // Then try to load from current directory (for backward compatibility)
    builder = builder.add_source(config::File::with_name("config"));

    let config = builder.build()?;
    let settings = config.try_deserialize::<serde_json::Value>()?;
    
    Ok(BotConfig {
        name: settings["bot"]["name"].as_str().unwrap_or("YouTube Downloader Bot").to_string(),
        allowed_pubkeys: settings["bot"]["allowed_pubkeys"].as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        nip05: settings["bot"]["nip05"].as_str().unwrap_or("").to_string(),
        relays: settings["relays"]["urls"].as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter_map(|v| v.as_str().map(String::from))
            .collect(),
        format: settings["downloads"]["format"].as_str().unwrap_or("mp3").to_string(),
        quality: settings["downloads"]["quality"].as_str().unwrap_or("0").to_string(),
    })
}

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

async fn get_random_dog_image() -> Result<String> {
    let response = reqwest::get("https://dog.ceo/api/breeds/image/random")
        .await?
        .json::<serde_json::Value>()
        .await?;
    
    Ok(response["message"].as_str().unwrap().to_string())
}

async fn set_profile_metadata(client: &Client, keys: &Keys, name: &str, nip05: &str) -> Result<()> {
    let dog_image = get_random_dog_image().await?;
    
    let metadata = json!({
        "name": name,
        "about": "I download YouTube videos and convert them to MP3!",
        "picture": dog_image,
        "nip05": nip05
    });

    let metadata_event = EventBuilder::new(
        Kind::Metadata,
        metadata.to_string(),
        &[],
    ).to_event(keys)?;

    client.send_event(metadata_event).await?;
    info!("Set profile metadata with name: {} and random dog picture", name);
    
    Ok(())
}

fn get_downloads_dir() -> Result<PathBuf> {
    if let Some(proj_dirs) = ProjectDirs::from("com", "rovr", "rovr") {
        let downloads_dir = proj_dirs.data_dir().join("downloads");
        fs::create_dir_all(&downloads_dir)?;
        Ok(downloads_dir)
    } else {
        error!("Could not determine application data directory");
        Ok(PathBuf::from("downloads"))
    }
}

async fn update_subscription(client: &Client, keys: &Keys, allowed_pubkeys: &[String], last_pubkeys: &mut Vec<String>) -> Result<bool> {
    // Check if the pubkeys have changed
    if allowed_pubkeys == last_pubkeys {
        return Ok(false);
    }

    // Update last known pubkeys
    *last_pubkeys = allowed_pubkeys.to_vec();

    // Convert allowed pubkeys to XOnlyPublicKey
    let mut authors = Vec::new();
    for pubkey in allowed_pubkeys {
        if let Ok(pk) = XOnlyPublicKey::from_bech32(pubkey) {
            authors.push(pk);
        }
    }

    // Create a new subscription for DMs from the allowed pubkeys
    let subscription = Filter::new()
        .kinds(vec![Kind::EncryptedDirectMessage])
        .pubkey(keys.public_key())
        .authors(authors)
        .since(Timestamp::now());

    // Unsubscribe from old filters
    client.unsubscribe().await;
    
    // Subscribe to new filters
    client.subscribe(vec![subscription]).await;
    info!("Updated subscription with {} allowed pubkeys", allowed_pubkeys.len());
    
    Ok(true)
}

async fn handle_download(
    client: &Client,
    keys: &Keys,
    event: &Event,
    video_id: &str,
    format: String,
    quality: String,
    downloads_dir: PathBuf,
) -> Result<()> {
    let youtube_url = format!("https://youtube.com/watch?v={}", video_id);
    info!("Starting download for video ID: {}", video_id);
    
    // Use yt-dlp from virtual environment
    let yt_dlp_path = PathBuf::from("/home/ethan/code/rovr/venv/bin/yt-dlp");
    
    // Download and convert to audio
    let output = Command::new(yt_dlp_path)
        .args([
            "-x", // Extract audio
            "--audio-format", &format,
            "--audio-quality", &quality,
            "-o", // Output format
            &format!("{}/%(title)s.%(ext)s", downloads_dir.display()),
            &youtube_url,
        ])
        .output()?;

    if output.status.success() {
        info!("Successfully downloaded and converted video: {}", video_id);
        // Send success message with YouTube link and download path
        let response = format!(
            "Successfully downloaded and converted the video to {}!\n\nYouTube: {}\n\nFile saved to: {}",
            format.to_uppercase(),
            youtube_url,
            downloads_dir.display()
        );
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
        info!("Sent success response to user for video: {}", video_id);
    } else {
        error!("Failed to download video {}. Error: {:?}", video_id, String::from_utf8_lossy(&output.stderr));
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
        info!("Sent error response to user for video: {}", video_id);
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logger with INFO level by default
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    info!("Starting YouTube DM bot...");

    // Load configuration
    let config = load_config()?;
    info!("Loaded configuration");

    // Load or generate keys
    let keys = get_keys()?;
    let bot_npub = keys.public_key().to_bech32()?;
    println!("\n=== BOT IDENTITY ===");
    println!("Name: {}", config.name);
    println!("npub: {}", bot_npub);
    println!("===================\n");
    info!("Bot public key: {}", bot_npub);
    print_qr_code(&bot_npub);

    // Create downloads directory
    let downloads_dir = get_downloads_dir()?;
    info!("Using downloads directory: {}", downloads_dir.display());

    // Create a new client
    let client = Client::new(&keys);
    info!("Created new Nostr client");

    // Add relays
    info!("Connecting to relays...");
    for relay in &config.relays {
        client.add_relay(relay.clone(), None).await?;
        info!("Added relay: {}", relay);
    }
    info!("Added all relays");

    // Connect to relays
    client.connect().await;
    info!("Connected to all relays");

    // Set profile metadata
    set_profile_metadata(&client, &keys, &config.name, &config.nip05).await?;

    // Initial subscription setup
    let mut last_pubkeys = Vec::new();
    update_subscription(&client, &keys, &config.allowed_pubkeys, &mut last_pubkeys).await?;

    // Create regex for YouTube URLs
    let youtube_regex = Regex::new(YOUTUBE_URL_PATTERN).unwrap();
    info!("Initialized YouTube URL regex");

    info!("Bot is running and listening for DMs from allowed pubkeys");

    // Spawn a task to periodically update the subscription
    let client_clone = client.clone();
    let keys_clone = keys.clone();
    let config_arc = Arc::new(RwLock::new(config.clone()));
    let last_pubkeys_arc = Arc::new(RwLock::new(Vec::new()));
    
    tokio::spawn(async move {
        loop {
            sleep(Duration::from_secs(5)).await;
            let config = config_arc.read().await;
            let pubkeys = config.allowed_pubkeys.clone();
            let mut last_pubkeys = last_pubkeys_arc.write().await;
            if let Err(e) = update_subscription(&client_clone, &keys_clone, &pubkeys, &mut last_pubkeys).await {
                error!("Failed to update subscription: {}", e);
            }
        }
    });

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
                        let video_id_str = video_id.as_str().to_string();
                        info!("Spawning download task for video: {}", video_id_str);
                        
                        let format = config.format.clone();
                        let quality = config.quality.clone();
                        let downloads_dir = downloads_dir.clone();
                        let client = client.clone();
                        let keys = keys.clone();
                        let event = event.clone();
                        
                        // Spawn a new task for the download
                        tokio::spawn(async move {
                            if let Err(e) = handle_download(
                                &client,
                                &keys,
                                &event,
                                &video_id_str,
                                format,
                                quality,
                                downloads_dir,
                            ).await {
                                error!("Error handling download for video {}: {}", video_id_str, e);
                            }
                        });
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
