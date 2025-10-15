mod bot;
mod downloader;
mod utils;
mod models;

use dotenvy::dotenv;
use std::env;
use teloxide::prelude::*;

#[tokio::main]
async fn main() {
    dotenv().ok();
    env_logger::init();

    log::info!("Starting Social Media Downloader Bot...");

    let token = env::var("BOT_TOKEN")
        .expect("BOT_TOKEN must be set in .env file");

    let bot = Bot::new(token);

    bot::run(bot).await;
}