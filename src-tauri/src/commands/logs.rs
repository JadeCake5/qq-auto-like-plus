use std::io::{BufRead, BufReader};

use tauri::Manager;

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogEntry {
    pub timestamp: String,
    pub level: String,
    pub message: String,
}

/// 读取日志文件中本次启动的条目（最多最近 500 行）
#[tauri::command]
pub fn get_startup_logs(app: tauri::AppHandle) -> Result<Vec<LogEntry>, String> {
    let log_dir = app.path().app_log_dir().map_err(|e| e.to_string())?;

    // tauri-plugin-log 的日志文件名格式
    let log_file = log_dir.join("QQ Auto Like Plus.log");
    if !log_file.exists() {
        return Ok(vec![]);
    }

    let file = std::fs::File::open(&log_file).map_err(|e| e.to_string())?;
    let reader = BufReader::new(file);

    // 先读所有行到内存，取最后 500 行
    let all_lines: Vec<String> = reader.lines().flatten().collect();
    let start = if all_lines.len() > 500 {
        all_lines.len() - 500
    } else {
        0
    };

    let mut entries = Vec::new();
    for line in &all_lines[start..] {
        if let Some(entry) = parse_log_line(line) {
            entries.push(entry);
        }
    }

    Ok(entries)
}

/// 解析日志行：[2026-03-15][04:11:48][module][INFO] message
fn parse_log_line(line: &str) -> Option<LogEntry> {
    if !line.starts_with('[') {
        return None;
    }

    // 提取 [日期][时间]
    let mut rest = line;
    let date = extract_bracket(&mut rest)?;
    let time = extract_bracket(&mut rest)?;
    let _module = extract_bracket(&mut rest);
    let level_raw = extract_bracket(&mut rest)?;

    let level = match level_raw.to_uppercase().as_str() {
        "INFO" => "info",
        "WARN" | "WARNING" => "warn",
        "ERROR" => "error",
        _ => return None, // 跳过 TRACE/DEBUG
    };

    let message = rest.trim().to_string();
    if message.is_empty() {
        return None;
    }

    Some(LogEntry {
        timestamp: format!("{} {}", date, time),
        level: level.to_string(),
        message,
    })
}

fn extract_bracket(s: &mut &str) -> Option<String> {
    if !s.starts_with('[') {
        return None;
    }
    let end = s.find(']')?;
    let content = s[1..end].to_string();
    *s = &s[end + 1..];
    Some(content)
}
