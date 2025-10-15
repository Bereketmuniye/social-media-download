use crate::models::{DownloadRequest, DownloadResult, YtDlpVideoInfo};
use crate::utils::sanitize_filename;
use anyhow::Result;
use std::path::PathBuf;
use tokio::process::Command;
use tokio::fs;
use serde_json;

pub struct Downloader {
    download_path: String,
    max_file_size: u64,
}

impl Downloader {
    pub fn new(download_path: String, max_file_size: u64) -> Self {
        // Create download directory if it doesn't exist
        let _ = std::fs::create_dir_all(&download_path);
        
        Self {
            download_path,
            max_file_size,
        }
    }

    pub async fn download_video(&self, request: &DownloadRequest) -> Result<DownloadResult> {
        log::info!("Starting download for URL: {}", request.url);

        // Get video info first
        let video_info = self.get_video_info(&request.url).await?;
        
        // Choose best format
        let format = self.choose_best_format(&video_info)?;
        
        // Download the video
        let output_path = self.download_with_ytdlp(&request.url, &video_info.title, &format).await?;
        
        // Check file size
        let metadata = fs::metadata(&output_path).await?;
        if metadata.len() > self.max_file_size {
            fs::remove_file(&output_path).await?;
            return Err(anyhow::anyhow!(
                "File too large: {} > {}", 
                metadata.len(), 
                self.max_file_size
            ));
        }

        Ok(DownloadResult {
            file_path: output_path.clone(),
            file_name: PathBuf::from(&output_path)
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string(),
            file_size: metadata.len(),
            duration: video_info.duration,
            thumbnail: video_info.thumbnail,
        })
    }

    async fn get_video_info(&self, url: &str) -> Result<YtDlpVideoInfo> {
        let output = Command::new("yt-dlp")
            .arg("--dump-json")
            .arg("--no-warnings")
            .arg(url)
            .output()
            .await?;

        if !output.status.success() {
            let error_msg = String::from_utf8_lossy(&output.stderr);
            return Err(anyhow::anyhow!("yt-dlp failed: {}", error_msg));
        }

        let info: YtDlpVideoInfo = serde_json::from_slice(&output.stdout)?;
        Ok(info)
    }

    fn choose_best_format(&self, video_info: &YtDlpVideoInfo) -> Result<String> {
        // Prefer mp4 format with reasonable size
        for format in &video_info.formats {
            if format.ext == "mp4" {
                if let Some(size) = format.filesize {
                    if size <= self.max_file_size && size > 0 {
                        return Ok(format.format_id.clone());
                    }
                }
            }
        }

        // Fallback to best format that fits
        for format in &video_info.formats {
            if let Some(size) = format.filesize {
                if size <= self.max_file_size && size > 0 {
                    return Ok(format.format_id.clone());
                }
            }
        }

        Err(anyhow::anyhow!("No suitable format found within size limit"))
    }

    async fn download_with_ytdlp(
        &self, 
        url: &str, 
        title: &str, 
        format: &str
    ) -> Result<String> {
        let sanitized_title = sanitize_filename(title);
        let output_template = format!("{}/%(title)s.%(ext)s", self.download_path);
        
        let status = Command::new("yt-dlp")
            .arg("-f")
            .arg(format)
            .arg("-o")
            .arg(&output_template)
            .arg("--no-warnings")
            .arg("--no-progress")
            .arg(url)
            .status()
            .await?;

        if !status.success() {
            return Err(anyhow::anyhow!("yt-dlp download failed"));
        }

        // Find the downloaded file - simplified approach
        let files = fs::read_dir(&self.download_path).await?;
        let mut latest_file = None;
        let mut latest_time = std::time::SystemTime::UNIX_EPOCH;

        let mut entries = files;
        while let Some(entry) = entries.next_entry().await? {
            if let Ok(metadata) = entry.metadata().await {
                if let Ok(modified) = metadata.modified() {
                    if modified > latest_time {
                        latest_time = modified;
                        latest_file = Some(entry.path());
                    }
                }
            }
        }

        if let Some(file_path) = latest_file {
            Ok(file_path.to_string_lossy().to_string())
        } else {
            Err(anyhow::anyhow!("Could not find downloaded file"))
        }
    }

    pub async fn cleanup_file(&self, file_path: &str) -> Result<()> {
        if PathBuf::from(file_path).exists() {
            fs::remove_file(file_path).await?;
            log::info!("Cleaned up file: {}", file_path);
        }
        Ok(())
    }
}