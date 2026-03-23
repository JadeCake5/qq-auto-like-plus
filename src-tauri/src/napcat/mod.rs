use serde::{Deserialize, Serialize};

pub mod config;
pub mod downloader;
pub mod process;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum NapCatStatus {
    NotInstalled,
    Downloading,
    Extracting,
    Ready,
    Starting,
    WaitingForLogin,
    Running,
    Error(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub percentage: f64,
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bps: u64,
    pub eta_seconds: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtractProgress {
    pub current_file: u32,
    pub total_files: u32,
    pub percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LoginInfo {
    pub qq_number: String,
    pub nickname: String,
}

pub fn check_napcat_status(app_data_dir: &std::path::Path) -> NapCatStatus {
    let napcat_dir = app_data_dir.join("napcat");
    if napcat_dir.exists() && napcat_dir.is_dir() {
        if std::fs::read_dir(&napcat_dir)
            .map(|mut d| d.next().is_some())
            .unwrap_or(false)
        {
            NapCatStatus::Ready
        } else {
            NapCatStatus::NotInstalled
        }
    } else {
        NapCatStatus::NotInstalled
    }
}

/// 递归复制目录
pub fn copy_dir_recursive(src: &std::path::Path, dst: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dst)?;
    for entry in std::fs::read_dir(src)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let dest_path = dst.join(entry.file_name());
        if ty.is_dir() {
            copy_dir_recursive(&entry.path(), &dest_path)?;
        } else {
            std::fs::copy(entry.path(), dest_path)?;
        }
    }
    Ok(())
}
