#![allow(dead_code)]

use clap::Parser;
use tracing_subscriber::EnvFilter;

mod cli;
mod config;
mod error;
mod handlers;
mod models;
mod request;
mod response;
mod server;
mod storage;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")))
        .init();

    let cli = cli::Cli::parse();
    let config = cli.to_config();

    std::fs::create_dir_all(&config.store_root).ok();
    std::fs::create_dir_all(&config.log_dir).ok();

    server::start_server(config).await;
}
