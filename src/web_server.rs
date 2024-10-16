use actix_web::{App, HttpServer};
use actix_files::Files;
use std::path::PathBuf;
use tokio::sync::Notify;
use std::sync::Arc;

pub async fn run_file_server(converted_dir: PathBuf, shutdown_signal: Arc<Notify>) -> std::io::Result<()> {
    let server = HttpServer::new(move || {
        App::new()
            .service(Files::new("/files", converted_dir.clone()).show_files_listing())
    })
    .bind("0.0.0.0:8089")?;

    let server_handle = server.run();

    tokio::select! {
        _ = server_handle => {},
        _ = shutdown_signal.notified() => {
            // Graceful shutdown
            tracing::info!("Shutting down file server");
        }
    }

    Ok(())
}


