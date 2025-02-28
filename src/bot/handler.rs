use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;
use tracing::info;

use crate::bot::commands::convert::ConvertCommand;

pub struct Handler;

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        // Ignore messages from self and other bots
        if msg.author.bot {
            return;
        }

        // Check for embeds
        if msg.embeds.is_empty() {
            // Try to handle with the convert command
            ConvertCommand::handle(&ctx, &msg).await;
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        info!("{} is connected!", ready.user.name);
    }
}
