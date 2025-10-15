mod bot;
mod downloader;
mod utils;
mod models;

use dotenvy::dotenv;
use std::env;
use teloxide::prelude::*;
use warp::Filter;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    log::info!("Starting Social Media Downloader Bot...");

    // Start health check server in background
    tokio::spawn(start_health_server());

    let token = env::var("BOT_TOKEN")
        .expect("BOT_TOKEN must be set in .env file");

    let bot = Bot::new(token);

    bot::run(bot).await;
}

async fn start_health_server() {
    let health_route = warp::path("health")
        .map(|| warp::reply::json(&serde_json::json!({"status": "ok"})));

    log::info!("Health check server running on port 8000");
    warp::serve(health_route)
        .run(([0, 0, 0, 0], 8000))
        .await;
}