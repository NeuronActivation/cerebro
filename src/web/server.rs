use actix_files::Files;
use actix_web::{App, HttpServer};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::Notify;
use tracing::info;

use crate::config::settings::CONFIG;

pub async fn run_file_server(shutdown_signal: Arc<Notify>) -> std::io::Result<()> {
    let addr = format!("{}:{}", CONFIG.webserver_host, CONFIG.webserver_port)
        .parse::<SocketAddr>()
        .expect("Failed to parse host and port into SocketAddr");

    let converted_dir = CONFIG.converted_path();

    let server =
        HttpServer::new(move || App::new().service(Files::new("/", converted_dir.clone())))
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
