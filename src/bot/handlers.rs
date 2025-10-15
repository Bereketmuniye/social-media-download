use teloxide::prelude::*;
use teloxide::types::InputFile;
use crate::downloader::Downloader;
use crate::models::DownloadRequest;
use anyhow::Result;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "Supported commands:")]
pub enum BotCommand {
    #[command(description = "Start the bot")]
    Start,
    #[command(description = "Show help message")]
    Help,
    #[command(description = "Check if bot is alive")]
    Status,
}

pub async fn handle_commands(
    bot: Bot,
    msg: Message,
    cmd: BotCommand,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        BotCommand::Start => {
            let welcome_text = r#"
🤖 **Social Media Downloader Bot**

Send me a link from:
• YouTube
• TikTok  
• Instagram
• Twitter/X

I'll download the video and send it to you!

**Commands:**
/start - Start the bot
/help - Show this help
/status - Check bot status

**Note:** Videos are automatically deleted after download.
            "#;

            bot.send_message(msg.chat.id, welcome_text)
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        BotCommand::Help => {
            let help_text = r#"
**How to use:**
1. Send any supported social media URL
2. Wait for processing
3. Receive the video file

**Supported platforms:**
- YouTube
- TikTok
- Instagram
- Twitter/X

**Max file size:** 2GB
            "#;

            bot.send_message(msg.chat.id, help_text)
                .parse_mode(teloxide::types::ParseMode::Markdown)
                .await?;
        }
        BotCommand::Status => {
            bot.send_message(msg.chat.id, "✅ Bot is running and ready to download!")
                .await?;
        }
    }

    Ok(())
}

pub async fn handle_download_request(
    bot: Bot,
    msg: Message,
    downloader: Downloader,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let chat_id = msg.chat.id;
    let message_id = msg.id;
    let url = msg.text().unwrap_or("").to_string();

    log::info!("Received download request from {}: {}", chat_id, url);

    // Validate URL
    if !is_valid_url(&url) {
        bot.send_message(chat_id, "❌ Please provide a valid URL from supported platforms (YouTube, TikTok, Instagram, Twitter/X)")
            .reply_to_message_id(message_id)
            .await?;
        return Ok(());
    }

    // Send initial processing message
    let processing_msg = bot.send_message(chat_id, "⏳ Processing your request...")
        .reply_to_message_id(message_id)
        .await?;

    let request = DownloadRequest {
        url: url.clone(),
        chat_id: chat_id.0,
        message_id: message_id.0,
    };

    match downloader.download_video(&request).await {
        Ok(result) => {
            // Update message to downloading
            bot.edit_message_text(chat_id, processing_msg.id, "📥 Downloading video...")
                .await?;

            // Send the video file
            let file = InputFile::file(&result.file_path);
            
            match bot.send_video(chat_id, file)
                .caption(format!("📹 {}", result.file_name))
                .reply_to_message_id(message_id)
                .await
            {
                Ok(_) => {
                    // Update to completed
                    bot.edit_message_text(chat_id, processing_msg.id, "✅ Download completed!")
                        .await?;
                }
                Err(e) => {
                    log::error!("Failed to send video: {}", e);
                    bot.edit_message_text(chat_id, processing_msg.id, "❌ Failed to send video file")
                        .await?;
                }
            }

            // Cleanup downloaded file
            if let Err(e) = downloader.cleanup_file(&result.file_path).await {
                log::warn!("Failed to cleanup file {}: {}", result.file_path, e);
            }
        }
        Err(e) => {
            log::error!("Download failed: {}", e);
            let error_msg = if e.to_string().contains("File too large") {
                "❌ File too large. Maximum size is 2GB.".to_string()
            } else if e.to_string().contains("No suitable format") {
                "❌ No downloadable video found or video is too large.".to_string()
            } else {
                format!("❌ Download failed: {}", e)
            };
            
            bot.edit_message_text(chat_id, processing_msg.id, error_msg)
                .await?;
        }
    }

    Ok(())
}

fn is_valid_url(url: &str) -> bool {
    url.starts_with("http") && (
        url.contains("youtube.com") ||
        url.contains("youtu.be") ||
        url.contains("tiktok.com") ||
        url.contains("instagram.com") ||
        url.contains("twitter.com") ||
        url.contains("x.com")
    )
}