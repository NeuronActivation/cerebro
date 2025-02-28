use anyhow::Result;
use async_process::Command;
use std::path::{Path, PathBuf};
use tokio::fs::File;
use tracing::info;

use crate::config::settings::CONFIG;

pub struct YliProxy;

impl YliProxy {
    pub async fn convert_to_h264(input_path: &Path, id: &str) -> Result<PathBuf> {
        let file_name = format!("{}.mp4", id);
        let output_file = Path::new(&CONFIG.converted_dir).join(&file_name);

        let output = Command::new("ffmpeg")
            .args([
                "-y",
                "-i",
                input_path.to_str().unwrap(),
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
            .await?;

        if output.status.success() {
            info!("Successfully converted video to H264: {}", file_name);
            Ok(output_file)
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            Err(anyhow::anyhow!("Failed to convert video: {}", error))
        }
    }

    pub async fn download_file(url: &str) -> Result<PathBuf> {
        let res = reqwest::get(url).await?;

        if res.status().is_success() {
            let file_name = Path::new(url)
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("downloaded_file");

            let file_path = Path::new(&CONFIG.download_dir).join(file_name);
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

    pub fn get_file_url(file_name: &str) -> String {
        format!("{}/{}", CONFIG.server_url, file_name)
    }

    pub fn extract_id_from_url(url: &str) -> &str {
        url.split('/').last().unwrap().split('.').next().unwrap()
    }
}
