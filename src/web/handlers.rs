use actix_web::{get, web, HttpResponse};
use tokio::sync::Mutex;
use tracing::error;

use crate::web::models::ThumbnailCache;
use crate::web::thumbnails::{get_video_list, process_missing_thumbnails};

// Handler for the index page
#[get("/")]
pub async fn index(cache: web::Data<Mutex<ThumbnailCache>>) -> HttpResponse {
    let mut cache = cache.lock().await;

    // Refresh video list if needed
    if cache.needs_refresh() {
        match get_video_list().await {
            Ok(videos) => {
                cache.videos.clear();
                for video in videos {
                    cache.videos.insert(video.id.clone(), video);
                }
                cache.last_refresh = std::time::SystemTime::now();
                cache.initialized = true;

                // Process thumbnails for the initial load
                if !cache.initialized {
                    tokio::spawn(process_missing_thumbnails(cache.videos.clone()));
                } else {
                    // Process in the background for subsequent refreshes
                    tokio::spawn(process_missing_thumbnails(cache.videos.clone()));
                }
            }
            Err(e) => error!("Failed to refresh video list: {}", e),
        }
    }

    // Create a sorted list of videos (newest first)
    let mut videos: Vec<_> = cache.videos.values().cloned().collect();
    videos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    // Create HTML for the index page
    let mut html = String::from(
        r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>YliProxy</title>
        <style>
            body { font-family: Arial, sans-serif; margin: 0; padding: 20px; background-color: #f5f5f5; }
            .video-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 20px; }
            .video-item { background-color: white; border-radius: 8px; overflow: hidden; box-shadow: 0 2px 4px rgba(0,0,0,0.1); }
            .thumbnail { width: 100%; height: 180px; object-fit: cover; background-color: #ccc; }
            .video-info { padding: 10px; }
            h1 { color: #333; }
            a { text-decoration: none; color: inherit; }
            .video-title { margin: 5px 0; color: #333; }
        </style>
    </head>
    <body>
        <h1>YliProxy</h1>
        <div class="video-grid">
    "#,
    );

    for video in videos {
        html.push_str(&format!(r#"
            <div class="video-item">
                <a href="/{filename}">
                    <img class="thumbnail" src="/thumbs/{id}.jpg" alt="{filename}" onerror="this.style.backgroundColor='#ccc';">
                    <div class="video-info">
                        <h3 class="video-title">{filename}</h3>
                    </div>
                </a>
            </div>
        "#, id = video.id, filename = video.filename));
    }

    html.push_str(
        r#"
        </div>
    </body>
    </html>
    "#,
    );

    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(html)
}

// Initialize the cache with data on server startup
pub async fn initialize_cache(cache: web::Data<Mutex<ThumbnailCache>>) {
    tracing::info!("Initializing video cache on startup");

    let mut cache_lock = cache.lock().await;

    match get_video_list().await {
        Ok(videos) => {
            for video in videos {
                cache_lock.videos.insert(video.id.clone(), video);
            }
            cache_lock.last_refresh = std::time::SystemTime::now();
            cache_lock.initialized = true;

            // Process thumbnails in the background
            let videos_clone = cache_lock.videos.clone();
            tokio::spawn(async move {
                process_missing_thumbnails(videos_clone).await;
            });

            tracing::info!("Cache initialized with {} videos", cache_lock.videos.len());
        }
        Err(e) => error!("Failed to initialize video cache: {}", e),
    }
}
