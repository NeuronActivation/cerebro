use serenity::prelude::*;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

use crate::bot::handler::Handler;
use crate::config::settings::CONFIG;

pub async fn start_bot(shutdown_signal: Arc<Notify>) -> Result<(), Box<dyn std::error::Error>> {
    let intents = GatewayIntents::GUILD_MESSAGES
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::MESSAGE_CONTENT;

    let mut client = Client::builder(&CONFIG.discord_token, intents)
        .event_handler(Handler)
        .await?;

    let shard_manager = client.shard_manager.clone();

    tokio::spawn(async move {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for Ctrl+C");
        info!("Ctrl+C received, shutting down bot");
        shard_manager.shutdown_all().await;
        shutdown_signal.notify_waiters();
    });

    info!("Starting Discord bot");
    client.start().await?;

    Ok(())
}
