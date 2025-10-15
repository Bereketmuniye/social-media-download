pub mod handlers;
use handlers::*;

use teloxide::prelude::*;
use std::env;
use crate::downloader::Downloader;

pub async fn run(bot: Bot) {
    let max_file_size = env::var("MAX_FILE_SIZE")
        .unwrap_or_else(|_| "2000000000".to_string())
        .parse()
        .unwrap_or(2_000_000_000);

    let download_path = env::var("DOWNLOAD_PATH")
        .unwrap_or_else(|_| "./downloads".to_string());

    let downloader = Downloader::new(download_path, max_file_size);

    let handler = dptree::entry()
        .branch(Update::filter_message().filter_command::<Command>().endpoint(handle_commands))
        .branch(Update::filter_message().filter(text_contains("http")).endpoint(handle_download_request));

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![downloader])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn text_contains(text: &str) -> bool {
    text.contains("youtube.com") ||
    text.contains("youtu.be") ||
    text.contains("tiktok.com") ||
    text.contains("instagram.com") ||
    text.contains("twitter.com") ||
    text.contains("x.com")
}