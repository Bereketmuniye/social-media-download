use std::path::Path;
use std::process::Stdio;
use tokio::process::Command;
use anyhow::Result;

pub fn sanitize_filename(filename: &str) -> String {
    filename
        .chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' || c == '.' { c } else { '_' })
        .collect()
}

pub async fn check_ytdlp_installed() -> Result<bool> {
    let output = Command::new("yt-dlp")
        .arg("--version")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .await;

    Ok(output.is_ok())
}

pub fn get_file_extension(url: &str) -> String {
    if url.contains("youtube.com") || url.contains("youtu.be") {
        "mp4".to_string()
    } else if url.contains("tiktok.com") {
        "mp4".to_string()
    } else if url.contains("instagram.com") {
        "mp4".to_string()
    } else if url.contains("twitter.com") || url.contains("x.com") {
        "mp4".to_string()
    } else {
        "mp4".to_string()
    }
}

pub async fn get_video_info(url: &str) -> Result<String> {
    let output = Command::new("yt-dlp")
        .arg("--get-title")
        .arg(url)
        .output()
        .await?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
    } else {
        Err(anyhow::anyhow!("Failed to get video info"))
    }
}