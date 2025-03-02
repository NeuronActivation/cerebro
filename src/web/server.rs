use actix_files::Files;
use actix_web::{web, App, HttpServer};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::{Mutex, Notify};
use tracing::info;

use crate::config::CONFIG;
use crate::web::handlers::{index, initialize_cache};
use crate::web::models::ThumbnailCache;
use crate::web::thumbnails::ensure_thumbs_dir;

pub async fn run_file_server(shutdown_signal: Arc<Notify>) -> std::io::Result<()> {
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port)
        .parse::<SocketAddr>()
        .expect("Failed to parse host and port into SocketAddr");

    // Create thumbs directory if it doesn't exist
    let thumbs_dir = ensure_thumbs_dir()?;

    // Create the thumbnail cache
    let thumbnail_cache = web::Data::new(Mutex::new(ThumbnailCache::new()));

    // Initialize the cache on startup
    let cache_clone = thumbnail_cache.clone();
    tokio::spawn(async move {
        initialize_cache(cache_clone).await;
    });

    let server = HttpServer::new(move || {
        let converted_path = PathBuf::from(&CONFIG.converted_dir);
        App::new()
            .app_data(thumbnail_cache.clone())
            .service(index)
            .service(Files::new("/thumbs", thumbs_dir.clone()))
            .service(
                Files::new("/", converted_path)
                    .index_file("") // No index file, we handle it with our custom handler
                    .use_last_modified(true),
            )
    })
    .bind(addr)?;

    info!("Starting file server on: {addr}");

    let server_handle = server.run();

    tokio::select! {
        result = server_handle => result,
        _ = shutdown_signal.notified() => {
            info!("Shutdown signal received, stopping web server");
            Ok(())
        }
    }
}
