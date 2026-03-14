use tauri::State;

use crate::webhook::WebhookServerHandle;

pub type WebhookState = std::sync::Mutex<Option<WebhookServerHandle>>;

#[tauri::command]
pub fn get_webhook_status(state: State<'_, WebhookState>) -> Result<String, String> {
    let guard = state.lock().map_err(|e| e.to_string())?;
    match guard.as_ref() {
        Some(handle) if handle.is_running() => Ok("running".to_string()),
        Some(_) => Ok("stopped".to_string()),
        None => Ok("error".to_string()),
    }
}
