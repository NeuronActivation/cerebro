use anyhow::Result;
use lazy_static::lazy_static;
use regex::Regex;
use serenity::model::channel::Message;
use serenity::prelude::*;
use tracing::{error, info};

use crate::bot::services::yliproxy::YliProxy;

lazy_static! {
    static ref MP4_PATTERN: Regex = Regex::new(r"https://.+\.ylilauta\.org/.+\.mp4").unwrap();
}

pub struct ConvertCommand;

impl ConvertCommand {
    pub async fn handle(ctx: &Context, msg: &Message) -> bool {
        // Check if message contains a Ylilauta video URL
        if let Some(captures) = MP4_PATTERN.captures(&msg.content) {
            if let Some(url) = captures.get(0) {
                info!("Found Ylilauta video URL: {}", url.as_str());
                let process_reaction = msg.react(&ctx.http, '⏳').await.unwrap();

                if let Err(e) = Self::process_video(ctx, msg, url.as_str()).await {
                    error!("Error processing video: {:?}", e);
                    msg.react(&ctx.http, '❌').await.ok();
                }

                if let Err(e) = process_reaction.delete(&ctx.http).await {
                    error!("Error removing reactions: {:?}", e);
                }

                return true;
            }
        }

        false
    }

    async fn process_video(ctx: &Context, msg: &Message, url: &str) -> Result<()> {
        // Download the video
        let file_path = YliProxy::download_file(url).await?;

        // Extract ID and convert video
        let id = YliProxy::extract_id_from_url(url);
        let output_file = YliProxy::convert_to_h264(&file_path, id).await?;

        // Get the file URL and send it
        let file_name = output_file.file_name().unwrap().to_str().unwrap();
        let file_url = YliProxy::get_file_url(file_name);
        msg.channel_id.say(&ctx.http, file_url).await?;

        Ok(())
    }
}
