use std::path::Path;

use futures_util::StreamExt;
use tauri::Emitter;
use tokio::io::AsyncWriteExt;

use super::{DownloadProgress, ExtractProgress};

const NAPCAT_DOWNLOAD_URL: &str =
    "https://github.com/NapNeko/NapCatQQ/releases/latest/download/NapCat.Shell.Windows.Node.zip";

pub async fn download_napcat_zip(
    app_handle: &tauri::AppHandle,
    app_data_dir: &Path,
) -> Result<std::path::PathBuf, crate::errors::AppError> {
    let zip_path = app_data_dir.join("napcat_download.zip");
    let response = reqwest::get(NAPCAT_DOWNLOAD_URL).await?;
    let total_bytes = response.content_length().unwrap_or(0);

    let mut file = tokio::fs::File::create(&zip_path)
        .await
        .map_err(crate::errors::AppError::Io)?;

    let mut downloaded: u64 = 0;
    let start_time = std::time::Instant::now();
    let mut stream = response.bytes_stream();

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)
            .await
            .map_err(crate::errors::AppError::Io)?;
        downloaded += chunk.len() as u64;

        let elapsed = start_time.elapsed().as_secs_f64();
        let speed_bps = if elapsed > 0.0 {
            (downloaded as f64 / elapsed) as u64
        } else {
            0
        };
        let eta_seconds = if speed_bps > 0 && total_bytes > downloaded {
            (total_bytes - downloaded) / speed_bps
        } else {
            0
        };

        let progress = DownloadProgress {
            percentage: if total_bytes > 0 {
                downloaded as f64 / total_bytes as f64 * 100.0
            } else {
                0.0
            },
            downloaded_bytes: downloaded,
            total_bytes,
            speed_bps,
            eta_seconds,
        };
        let _ = app_handle.emit("napcat:download-progress", &progress);
    }
    file.flush()
        .await
        .map_err(crate::errors::AppError::Io)?;

    tracing::info!("NapCat 下载完成: {} bytes", downloaded);
    Ok(zip_path)
}

pub fn extract_napcat_zip(
    app_handle: &tauri::AppHandle,
    zip_path: &Path,
    app_data_dir: &Path,
) -> Result<(), crate::errors::AppError> {
    let target_dir = app_data_dir.join("napcat");
    std::fs::create_dir_all(&target_dir)?;

    let file = std::fs::File::open(zip_path)?;
    let mut archive =
        zip::ZipArchive::new(file).map_err(|e| crate::errors::AppError::Extract(e.to_string()))?;

    let total_files = archive.len() as u32;

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| crate::errors::AppError::Extract(e.to_string()))?;

        let out_path = target_dir.join(entry.mangled_name());

        if entry.is_dir() {
            std::fs::create_dir_all(&out_path)?;
        } else {
            if let Some(parent) = out_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut outfile = std::fs::File::create(&out_path)?;
            std::io::copy(&mut entry, &mut outfile)?;
        }

        let progress = ExtractProgress {
            current_file: i as u32 + 1,
            total_files,
            percentage: (i as f64 + 1.0) / total_files as f64 * 100.0,
        };
        let _ = app_handle.emit("napcat:extract-progress", &progress);
    }

    let _ = std::fs::remove_file(zip_path);
    tracing::info!("NapCat 解压完成: {:?}", target_dir);
    Ok(())
}

pub fn import_napcat_zip(
    app_handle: &tauri::AppHandle,
    local_zip_path: &Path,
    app_data_dir: &Path,
) -> Result<(), crate::errors::AppError> {
    if !local_zip_path.exists() {
        return Err(crate::errors::AppError::Extract(format!(
            "文件不存在: {:?}",
            local_zip_path
        )));
    }
    extract_napcat_zip(app_handle, local_zip_path, app_data_dir)
}
