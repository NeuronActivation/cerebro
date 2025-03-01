use actix_files::Files;
use actix_web::{App, HttpServer};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

use crate::config::settings::CONFIG;

pub async fn run_file_server(shutdown_signal: Arc<Notify>) -> std::io::Result<()> {
    let addr = format!("{}:{}", CONFIG.host, CONFIG.port)
        .parse::<SocketAddr>()
        .expect("Failed to parse host and port into SocketAddr");

    let server = HttpServer::new(move || {
        let path = PathBuf::from(&CONFIG.converted_dir);
        App::new().service(Files::new("/", path))
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
