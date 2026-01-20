use crate::commands::shortcut::ShortcutHandlerGuard;
use crate::error::AppError;
use crate::platform;
use log::debug;
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::sleep;

/// Get selected text.
#[tauri::command]
pub async fn get_selection(_app: AppHandle) -> Result<String, AppError> {
    // suspend shortcut handling to avoid interference
    let _guard = ShortcutHandlerGuard::suspend();

    Ok(platform::get_selection().unwrap_or_default())
}

/// Get selected text for mouse-driven hooks.
///
/// Only uses platform native APIs (no key simulation / clipboard fallback).
pub async fn get_selection_for_mouse(_app: AppHandle) -> Result<String, AppError> {
    // suspend shortcut handling to avoid interference
    let _guard = ShortcutHandlerGuard::suspend();

    for delay_ms in [0u64, 30, 80, 150] {
        if delay_ms > 0 {
            sleep(Duration::from_millis(delay_ms)).await;
        }

        if let Ok(text) = platform::get_selection() {
            let text = text.trim().to_string();
            if !text.is_empty() {
                return Ok(text);
            }
        }
    }

    debug!("mouse selection: native empty");
    Ok(String::new())
}
