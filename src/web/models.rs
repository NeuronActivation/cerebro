use serde::Serialize;
use std::collections::HashMap;
use std::time::{Duration, SystemTime};

#[derive(Serialize, Clone)]
pub struct VideoInfo {
    pub id: String,
    pub filename: String,
    pub thumbnail: String,
    pub created_at: SystemTime,
}

// Cache for thumbnails to avoid checking the filesystem too often
pub struct ThumbnailCache {
    pub videos: HashMap<String, VideoInfo>,
    pub last_refresh: SystemTime,
    pub initialized: bool,
}

impl ThumbnailCache {
    pub fn new() -> Self {
        Self {
            videos: HashMap::new(),
            last_refresh: SystemTime::UNIX_EPOCH, // Set to epoch to force initial refresh
            initialized: false,
        }
    }

    pub fn needs_refresh(&self) -> bool {
        // Always refresh if not initialized
        if !self.initialized {
            return true;
        }

        SystemTime::now()
            .duration_since(self.last_refresh)
            .map(|duration| duration > Duration::from_secs(30))
            .unwrap_or(true)
    }
}
