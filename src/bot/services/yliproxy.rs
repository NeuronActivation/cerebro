use anyhow::Result;
use async_process::Command;
use lazy_static::lazy_static;
use regex::Regex;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::info;

use crate::config::CONFIG;

lazy_static! {
    static ref ID_PATTERN: Regex = Regex::new(r"/([^/]+)\.mp4$").unwrap();
}

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
            let mut dest = fs::File::create(&file_path).await?;

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

    pub fn extract_id_from_url(url: &str) -> Result<String> {
        ID_PATTERN
            .captures(url)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
            .ok_or_else(|| anyhow::anyhow!("Failed to extract ID from URL: {}", url))
    }

    pub async fn get_existing_file_url(id: &str) -> Option<String> {
        let file_name = format!("{}.mp4", id);
        let output_path = Path::new(&CONFIG.converted_dir).join(&file_name);

        if fs::metadata(&output_path).await.is_ok() {
            Some(Self::get_file_url(&file_name))
        } else {
            None
        }
    }

    pub fn get_file_url(file_name: &str) -> String {
        format!("{}/{}", CONFIG.public_url, file_name)
    }
}
