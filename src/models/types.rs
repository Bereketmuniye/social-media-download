use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct DownloadRequest {
    pub url: String,
    pub chat_id: i64,
    pub message_id: i32,
}

#[derive(Debug, Clone)]
pub struct DownloadResult {
    pub file_path: String,
    pub file_name: String,
    pub file_size: u64,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct YtDlpFormat {
    pub format_id: String,
    pub ext: String,
    pub filesize: Option<u64>,
    pub url: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct YtDlpVideoInfo {
    pub title: String,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub formats: Vec<YtDlpFormat>,
}