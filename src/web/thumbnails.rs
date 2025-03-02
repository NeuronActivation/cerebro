use async_process::Command;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::fs;
use tracing::{error, info};

use crate::config::CONFIG;
use crate::web::models::VideoInfo;

// Process missing thumbnails in the background
pub async fn process_missing_thumbnails(videos: HashMap<String, VideoInfo>) {
    let thumbs_dir = Path::new(&CONFIG.converted_dir).join("thumbs");

    // Create thumbs directory if it doesn't exist
    if !thumbs_dir.exists() {
        if let Err(e) = fs::create_dir_all(&thumbs_dir).await {
            error!("Failed to create thumbs directory: {}", e);
            return;
        }
    }

    // Process each video that needs a thumbnail
    for (id, video) in videos {
        let thumb_path = thumbs_dir.join(format!("{}.jpg", id));

        // Skip if thumbnail already exists
        if fs::metadata(&thumb_path).await.is_ok() {
            continue;
        }

        // Generate thumbnail in the background
        let video_path = Path::new(&CONFIG.converted_dir).join(&video.filename);

        info!("Generating thumbnail for video: {}", id);

        if let Err(e) = generate_thumbnail(&video_path, &thumb_path).await {
            error!("Failed to generate thumbnail for {}: {}", id, e);
        }
    }
}

// Function to generate a thumbnail from a video
pub async fn generate_thumbnail(
    video_path: &Path,
    thumb_path: &Path,
) -> Result<(), std::io::Error> {
    let output = Command::new("ffmpeg")
        .args([
            "-y",
            "-i",
            video_path.to_str().unwrap(),
            "-ss",
            "00:00:01",
            "-vframes",
            "1",
            "-vf",
            "scale=320:-1", // Scale to 320px width, maintain aspect ratio
            "-q:v",
            "2",                          // High quality
            thumb_path.to_str().unwrap(), // Output thumbnail path
        ])
        .output()
        .await?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Failed to generate thumbnail: {}", error),
        ));
    }

    Ok(())
}

// Function to get the list of videos
pub async fn get_video_list() -> Result<Vec<VideoInfo>, std::io::Error> {
    let mut videos = Vec::new();
    let dir_path = Path::new(&CONFIG.converted_dir);

    let mut entries = fs::read_dir(dir_path).await?;

    while let Some(entry) = entries.next_entry().await? {
        let path = entry.path();

        // Skip directories and non-mp4 files
        if path.is_dir() || path.extension().and_then(|e| e.to_str()) != Some("mp4") {
            continue;
        }

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        let id = path
            .file_stem()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();

        // Create thumbnail path first
        let thumbnail = format!("thumbs/{}.jpg", id);

        // Get file metadata for creation time
        let metadata = fs::metadata(&path).await?;
        let created_at = metadata.created().unwrap_or(SystemTime::now());

        videos.push(VideoInfo {
            id,
            filename,
            thumbnail,
            created_at,
        });
    }

    Ok(videos)
}

// Ensure thumbnails directory exists
pub fn ensure_thumbs_dir() -> std::io::Result<PathBuf> {
    let thumbs_dir = Path::new(&CONFIG.converted_dir).join("thumbs");
    if !thumbs_dir.exists() {
        std::fs::create_dir_all(&thumbs_dir)?;
    }
    Ok(thumbs_dir)
}
