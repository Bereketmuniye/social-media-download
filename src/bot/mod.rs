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

    let handler = Update::filter_message()
        .branch(
            dptree::entry()
                .filter_command::<BotCommand>()
                .endpoint(handle_commands)
        )
        .branch(
            dptree::filter(|msg: Message| {
                msg.text().map(|text| is_social_media_url(text)).unwrap_or(false)
            })
            .endpoint(handle_download_request)
        );

    Dispatcher::builder(bot, handler)
        .dependencies(dptree::deps![downloader])
        .enable_ctrlc_handler()
        .build()
        .dispatch()
        .await;
}

fn is_social_media_url(text: &str) -> bool {
    text.contains("youtube.com") ||
    text.contains("youtu.be") ||
    text.contains("tiktok.com") ||
    text.contains("instagram.com") ||
    text.contains("twitter.com") ||
    text.contains("x.com")
}