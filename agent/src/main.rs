mod channels;
mod config;
mod db;
mod message_parser;
mod platform;
mod polling;
mod server;

use crate::channels::zalo_desktop::ZaloDesktopChannel;
use crate::config::Config;
use crate::db::Database;
use crate::polling::Poller;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_target(false)
        .init();

    // Load .env.local if exists
    let env_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .join(".env.local");
    if env_path.exists() {
        if let Ok(content) = std::fs::read_to_string(&env_path) {
            for line in content.lines() {
                let line = line.trim();
                if line.is_empty() || line.starts_with('#') {
                    continue;
                }
                if let Some((key, val)) = line.split_once('=') {
                    std::env::set_var(key.trim(), val.trim());
                }
            }
        }
    }

    let config = Config::load();

    tracing::info!("Haviz Agent v{}", env!("CARGO_PKG_VERSION"));
    tracing::info!("DB: {}", config.db_path.display());
    tracing::info!("Zalo reader: {}", config.zalo_reader_path.display());

    // Open database
    let db = Arc::new(
        Database::open(&config.db_path).expect("Failed to open database"),
    );

    // Create Zalo Desktop channel
    let channel = Box::new(ZaloDesktopChannel::new(
        config.zalo_reader_path.to_string_lossy().to_string(),
        config.my_name.clone(),
    ));

    // Start poller in background
    let db_poll = db.clone();
    tokio::spawn(async move {
        let mut poller = Poller::new(db_poll, channel, config.poll_interval_secs);
        poller.run().await;
    });

    // Start HTTP server
    let app = server::create_router(db);
    let addr = format!("0.0.0.0:{}", config.http_port);
    tracing::info!("HTTP server: http://localhost:{}", config.http_port);
    tracing::info!("Ready. Polling Zalo every {}s.", config.poll_interval_secs);

    let listener = tokio::net::TcpListener::bind(&addr)
        .await
        .expect("Failed to bind HTTP server");
    axum::serve(listener, app)
        .await
        .expect("HTTP server error");
}
