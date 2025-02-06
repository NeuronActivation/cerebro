mod web_server;

use std::env;
use std::net::{IpAddr, SocketAddr};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use tracing::{error, info};

use anyhow::Result;
use async_process::Command;
use lazy_static::lazy_static;
use regex::Regex;

use tokio::{fs::File, signal, sync::Notify};

struct Handler;

lazy_static! {
    static ref SERVER_URL: String = env::var("WEBSERVER_URL").expect("WEBSERVER_URL not set");
    static ref DATA_PATH: String = env::var("DATA_PATH").unwrap_or("./".to_string());
    static ref DOWNLOAD_DIR: String = format!("{}/downloads", *DATA_PATH);
    static ref CONVERTED_DIR: String = format!("{}/converted", *DATA_PATH);
    static ref MP4_PATTERN: Regex = Regex::new(r"https://.+\.ylilauta\.org/.+\.mp4").unwrap();
}

async fn download_file(url: &str) -> Result<PathBuf> {
    let res = reqwest::get(url).await?;

    if res.status().is_success() {
        let file_name = Path::new(url)
            .file_name()
            .and_then(|s| s.to_str())
            .unwrap_or("downloaded_file");

        let file_path = Path::new(&*DOWNLOAD_DIR).join(file_name);
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

async fn handle_mp4_conversion(ctx: &Context, msg: &Message, url: &str) -> Result<()> {
    let file_path = download_file(url).await?;

    // Extract the ID from the Ylilauta URL
    let id = url.split('/').last().unwrap().split('.').next().unwrap();
    let file_name = format!("{}.mp4", id);
    let output_file = Path::new(&*CONVERTED_DIR).join(&file_name);

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
        let file_url = format!("{}/{}", *SERVER_URL, file_name);
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

        // Handle MP4 files if no embeds are found
        if msg.embeds.is_empty() {
            if let Some(captures) = MP4_PATTERN.captures(&msg.content) {
                info!("No embeds found in the message. Downloading");
                if let Some(url) = captures.get(0) {
                    let process_reaction = msg.react(&ctx.http, '⏳').await.unwrap();

                    if let Err(e) = handle_mp4_conversion(&ctx, &msg, url.as_str()).await {
                        error!("Error handling MP4 conversion: {:?}", e);
                        msg.react(&ctx.http, '❌').await.ok();
                    }
                    if let Err(e) = process_reaction.delete(&ctx.http).await {
                        error!("Error removing reactions: {:?}", e);
                    }
                    return;
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

    let host: IpAddr = env::var("WEBSERVER_HOST")
        .unwrap_or("0.0.0.0".to_string())
        .parse()
        .expect("WEBSERVER_PORT must be a valid u16");

    let port: u16 = env::var("WEBSERVER_PORT")
        .unwrap_or("8080".to_string())
        .parse()
        .expect("WEBSERVER_PORT must be a valid u16");

    let server_addr: SocketAddr = format!("{}:{}", host, port)
        .parse()
        .expect("Failed to parse host and port into SocketAddr");

    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    tokio::fs::create_dir_all(&*DOWNLOAD_DIR)
        .await
        .expect("Failed to create download directory");
    tokio::fs::create_dir_all(&*CONVERTED_DIR)
        .await
        .expect("Failed to create converted directory");

    let shutdown = Arc::new(Notify::new());
    let shutdown_signal = shutdown.clone();

    // Start the file server in a separate task
    let file_server_handle = tokio::spawn(async move {
        let converted_dir = PathBuf::from(&*CONVERTED_DIR);
        if let Err(e) =
            web_server::run_file_server(server_addr, converted_dir, shutdown_signal).await
        {
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
        shard_manager.shutdown_all().await;
        info!("Ctrl+C received, shutting down");
        shutdown.notify_waiters();
    });

    if let Err(why) = client.start().await {
        error!("Client error: {:?}", why);
    }

    // Wait for the file server to finish
    file_server_handle.await.unwrap();

    info!("Shutdown complete");
}
