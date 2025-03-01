mod bot;
mod config;
mod web;

use std::sync::Arc;
use tokio::sync::Notify;
use tracing::{error, info};

use crate::bot::client::start_bot;
use crate::config::settings::CONFIG;
use crate::web::server::run_file_server;

#[tokio::main]
async fn main() {
    // Initialize logging
    tracing_subscriber::fmt::init();
    info!("Logging initialized, starting the application");

    // Create required directories
    tokio::fs::create_dir_all(&CONFIG.download_dir)
        .await
        .expect("Failed to create download directory");
    tokio::fs::create_dir_all(&CONFIG.converted_dir)
        .await
        .expect("Failed to create converted directory");

    // Create shutdown signal
    let shutdown = Arc::new(Notify::new());

    // Start web server
    let web_shutdown = shutdown.clone();
    let web_server_handle = tokio::spawn(async move {
        if let Err(e) = run_file_server(web_shutdown).await {
            error!("File server error: {:?}", e);
        }
    });

    // Start Discord bot
    let bot_shutdown = shutdown.clone();
    let bot_handle = tokio::spawn(async move {
        if let Err(e) = start_bot(bot_shutdown).await {
            error!("Bot error: {:?}", e);
        }
    });

    // Wait for both tasks to complete
    let _ = tokio::join!(web_server_handle, bot_handle);

    info!("Shutdown complete");
}
