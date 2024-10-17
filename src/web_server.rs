use actix_files::Files;
use actix_web::{App, HttpServer};
use std::sync::Arc;
use std::{net::SocketAddr, path::PathBuf};
use tokio::sync::Notify;
use tracing::info;

pub async fn run_file_server(
    addr: SocketAddr,
    converted_dir: PathBuf,
    shutdown_signal: Arc<Notify>,
) -> std::io::Result<()> {
    let server =
        HttpServer::new(move || App::new().service(Files::new("/", converted_dir.clone())))
            .bind(addr)?;

    info!("Starting file server on: {addr}");

    server.run().await?;
    shutdown_signal.notified();

    Ok(())
}
