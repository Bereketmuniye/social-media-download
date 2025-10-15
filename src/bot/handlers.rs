use teloxide::prelude::*;
use teloxide::types::InputFile;
use crate::downloader::Downloader;
use crate::models::DownloadRequest;
use anyhow::Result;

#[derive(BotCommand, Clone)]
#[command(rename = "lowercase", description = "Supported commands:")]
pub enum Command {
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
    cmd: Command,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    match cmd {
        Command::Start => {
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
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
        Command::Help => {
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
                .parse_mode(teloxide::types::ParseMode::MarkdownV2)
                .await?;
        }
        Command::Status => {
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

    log::info!("Received download request: {}", url);

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
            
            bot.send_video(chat_id, file)
                .caption(format!("📹 {}", result.file_name))
                .reply_to_message_id(message_id)
                .await?;

            // Update to completed
            bot.edit_message_text(chat_id, processing_msg.id, "✅ Download completed!")
                .await?;

            // Cleanup downloaded file
            let _ = downloader.cleanup_file(&result.file_path).await;
        }
        Err(e) => {
            let error_msg = format!("❌ Download failed: {}", e);
            bot.edit_message_text(chat_id, processing_msg.id, error_msg)
                .await?;
        }
    }

    Ok(())
}