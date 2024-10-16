mod web_server;

use async_process::Command;
use regex::Regex;
use std::env;
use std::thread;
use std::path::{Path, PathBuf};

use serenity::async_trait;
use serenity::builder::{CreateAttachment, CreateMessage};
use serenity::model::channel::{Attachment, Message};
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use tracing::{error, info, warn};

use anyhow::Result;
use lazy_static::lazy_static;

use tokio::runtime::Runtime;
use tokio::signal;
use std::sync::Arc;
use tokio::sync::Notify;
use tokio::fs::File;

struct Handler;

lazy_static! {
    static ref AVIF_PATTERN: Regex = Regex::new(r"https://.+\.ylilauta\.org/.+\.avif").unwrap();
    static ref MP4_PATTERN: Regex = Regex::new(r"https://.+\.ylilauta\.org/.+\.mp4").unwrap();
}

const DOWNLOAD_DIR: &str = "downloads";
const CONVERTED_DIR: &str = "converted";

async fn download_file(url: &str) -> Result<PathBuf> {
    let res = reqwest::get(url).await?;

    if res.status().is_success() {
        let file_name = Path::new(url)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("downloaded_file");

        let file_path = Path::new(DOWNLOAD_DIR).join(file_name);
        let mut dest = File::create(&file_path).await?;

        let content = res.bytes().await?;
        tokio::io::copy(&mut content.as_ref(), &mut dest).await?;
        info!(
            "File '{}' downloaded and saved successfully.",
            file_path.display()
        );
        Ok(file_path)
    } else {
        Err(anyhow::anyhow!(
            "Download failed with status: {}",
            res.status()
        ))
    }
}

async fn convert_avif_to_png(file_path: &Path) -> Result<PathBuf> {
    let avif_img = match image::open(&file_path) {
        Ok(img) => img,
        Err(e) => {
            tokio::fs::remove_file(&file_path).await?;
            return Err(anyhow::anyhow!("Failed to open AVIF image: {:?}", e));
        }
    };

    // Change the file extension to .png
    let converted_png_path = file_path.with_extension("png");

    if let Err(e) = avif_img.save_with_format(&converted_png_path, image::ImageFormat::Png) {
        return Err(anyhow::anyhow!("Failed to save PNG image: {:?}", e));
    }

    // clean up original file
    if let Err(e) = tokio::fs::remove_file(&file_path).await {
        error!("Failed to remove file '{}': {:?}", file_path.display(), e);
    }

    Ok(converted_png_path)
}

async fn send_file_to_channel(ctx: &Context, msg: &Message, file_path: &Path) -> Result<()> {
    let attachments = match CreateAttachment::path(&file_path).await {
        Ok(attachment) => vec![attachment],
        Err(e) => {
            error!("Failed to create attachment: {:?}", e);
            return Err(anyhow::anyhow!("Failed to create attachment: {:?}", e));
        }
    };

    if let Err(e) = msg
        .channel_id
        .send_files(&ctx.http, attachments, CreateMessage::new())
        .await
    {
        error!("Failed to send files: {:?}", e);
        return Err(anyhow::anyhow!("Failed to send files: {:?}", e));
    }

    info!("File '{}' sent to the channel", file_path.display());

    // clean up sent file
    if let Err(e) = tokio::fs::remove_file(&file_path).await {
        warn!("Failed to remove file '{}': {:?}", file_path.display(), e);
    }

    Ok(())
}

async fn handle_avif_conversion(ctx: &Context, msg: &Message, url: &str) -> Result<()> {
    let file_path = match download_file(url).await {
        Ok(path) => path,
        Err(e) => {
            error!("Failed to download file: {:?}", e);
            return Err(e);
        }
    };

    let converted_png_path = convert_avif_to_png(&file_path).await?;
    send_file_to_channel(ctx, msg, &converted_png_path).await
}

async fn handle_attachment(ctx: &Context, msg: &Message, attachment: &Attachment) -> Result<()> {
    let content = attachment.download().await?;
    let file_path = Path::new(DOWNLOAD_DIR).join(&attachment.filename);
    let mut file = File::create(&file_path).await?;

    tokio::io::copy(&mut content.as_ref(), &mut file).await?;

    // Check if the file is an AVIF image properly (not just by extension)
    if image::guess_format(&content).unwrap() == image::ImageFormat::Avif {
        let png_file_path = convert_avif_to_png(&file_path).await?;
        send_file_to_channel(ctx, msg, &png_file_path).await?;
    }

    Ok(())
}

async fn handle_mp4_conversion(ctx: &Context, msg: &Message, url: &str) -> Result<()> {
    let file_path = download_file(url).await?;

    // Extract the ID from the Ylilauta URL
    let id = url.split('/').last().unwrap().split('.').next().unwrap();
    let file_name = format!("{}.mp4", id);
    let output_file = Path::new(CONVERTED_DIR).join(&file_name);

    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            file_path.to_str().unwrap(),
            "-c:v",
            "libx264",
            "-preset",
            "veryfast",
            "-crf",
            "23",
            "-threads",
            "4",
            "-c:a",
            "copy",
            output_file.to_str().unwrap(),
        ])
        .output()
        .await
        .expect("failed to execute process");

    if output.status.success() {
        let file_url = format!("http://localhost:8080/files/{}", file_name);
        msg.channel_id.say(&ctx.http, file_url).await?;
    } else {
        return Err(anyhow::anyhow!(
            "Failed to convert MP4 file: {:?}",
            output.stderr
        ));
    }

    Ok(())
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from self and other bots
        if msg.author.bot {
            return;
        }

        // Handle AVIF images from Ylilauta.org
        if let Some(captures) = AVIF_PATTERN.captures(&msg.content) {
            info!("Ylilauta AVIF conversion");
            if let Some(url) = captures.get(0) {
                if let Err(e) = handle_avif_conversion(&ctx, &msg, url.as_str()).await {
                    error!("Error handling AVIF conversion: {:?}", e);
                }
            }
        }

        // Handle MP4 files if no embeds are found
        if msg.embeds.is_empty() {
            if let Some(captures) = MP4_PATTERN.captures(&msg.content) {
                info!("No embeds found in the message. Downloading");
                if let Some(url) = captures.get(0) {
                    if let Err(e) = handle_mp4_conversion(&ctx, &msg, url.as_str()).await {
                        error!("Error handling MP4 conversion: {:?}", e);
                    }
                }
            }
        }

        // Handle attachments
        for attachment in &msg.attachments {
            if attachment.filename.ends_with(".avif") {
                info!("Attachment: {}, starting conversion", attachment.filename);
                if let Err(e) = handle_attachment(&ctx, &msg, attachment).await {
                    error!("Error handling AVIF attachment: {:?}", e);
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}


#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    info!("Logging initialized, starting the bot and file server");

    let token = env::var("DISCORD_TOKEN").expect("Discord token not set in the environment");
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    tokio::fs::create_dir_all(DOWNLOAD_DIR)
        .await
        .expect("Failed to create download directory");

    tokio::fs::create_dir_all(CONVERTED_DIR)
        .await
        .expect("Failed to create converted directory");

    let shutdown = Arc::new(Notify::new());
    let shutdown_signal = shutdown.clone();

    // Start the file server in a separate task
    let file_server_handle = tokio::spawn(async move {
        let converted_dir = std::path::PathBuf::from(CONVERTED_DIR);
        if let Err(e) = web_server::run_file_server(converted_dir, shutdown_signal).await {
            error!("File server error: {:?}", e);
        }
    });

    let mut client = Client::builder(&token, intents)
        .event_handler(Handler)
        .await
        .expect("Error creating client");

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        info!("Ctrl+C received, shutting down");
        shard_manager.shutdown_all().await;
        shutdown.notify_waiters();
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    // Wait for the file server to finish
    file_server_handle.await.unwrap();

    info!("Shutdown complete");
}
