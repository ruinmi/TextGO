use crate::commands::clipboard::{clear_clipboard, get_clipboard_text, with_clipboard_backup};
use crate::commands::shortcut::ShortcutHandlerGuard;
use crate::error::AppError;
use crate::platform;
use crate::ENIGO;
use enigo::{Direction, Key, Keyboard};
use log::{debug, warn};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tauri::AppHandle;
use tokio::time::sleep;

// maximum wait time in milliseconds for clipboard to update
static MAX_WAIT_TIME: AtomicU64 = AtomicU64::new(1000);

#[derive(Clone, Copy)]
enum CopyShortcut {
    Standard,
    Terminal,
}

/// Get selected text.
#[tauri::command]
pub async fn get_selection(app: AppHandle) -> Result<String, AppError> {
    get_selection_with_fallback(app, true).await
}

/// Get selected text, optionally falling back to clipboard.
///
/// `allow_clipboard_fallback = false` is useful for mouse-driven selection hooks,
/// where synthesizing `Cmd/Ctrl+C` can interfere with the user's selection
/// (e.g. VSCode integrated terminal).
pub async fn get_selection_with_fallback(
    app: AppHandle,
    allow_clipboard_fallback: bool,
) -> Result<String, AppError> {
    // suspend shortcut handling to avoid interference
    let _guard = ShortcutHandlerGuard::suspend();

    // try using platform native API to get selected text first
    if let Ok(text) = platform::get_selection() {
        if !text.is_empty() {
            return Ok(text);
        }
    }

    if !allow_clipboard_fallback {
        return Ok(String::new());
    }

    // if native API fails, fall back to clipboard method
    warn!("Failed to get selection natively, fallback to clipboard method");
    get_selection_fallback_with_shortcut(app, CopyShortcut::Standard).await
}

/// Get selected text through clipboard.
async fn get_selection_fallback_with_shortcut(
    app: AppHandle,
    shortcut: CopyShortcut,
) -> Result<String, AppError> {
    // use backup-operation-restore mode
    with_clipboard_backup(|| async move {
        // clear clipboard
        clear_clipboard()?;

        // send copy shortcut
        // https://github.com/enigo-rs/enigo/issues/153
        let _ = app.run_on_main_thread(move || {
            let _ = send_copy_keys(shortcut);
        });

        // wait for clipboard content to change in a loop
        let max_wait_time = Duration::from_millis(MAX_WAIT_TIME.load(Ordering::Relaxed));
        let check_interval = Duration::from_millis(5); // check interval 5ms
        let max_attempts = max_wait_time.as_millis() / check_interval.as_millis();

        let mut selected_text = String::new();

        for _attempt in 0..max_attempts {
            sleep(check_interval).await;

            // read current clipboard text
            if let Ok(current_text) = get_clipboard_text() {
                if !current_text.is_empty() {
                    // if clipboard content changed, copy operation completed
                    selected_text = current_text;
                    break;
                }
            }
        }

        if selected_text.is_empty() {
            warn!(
                "Clipboard did not change within {} ms, possibly no text selected",
                max_wait_time.as_millis()
            );
        } else {
            // adjust max wait time for next time
            MAX_WAIT_TIME
                .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |current| {
                    if current > 200 {
                        Some((current - 100).max(200))
                    } else {
                        Some(current)
                    }
                })
                .ok();
        }

        Ok(selected_text)
    })
    .await
}

/// Get selected text for mouse-driven hooks.
///
/// - retries native selection once after a short delay (some apps update selection late)
/// - uses clipboard fallback when needed, with a terminal-safe shortcut on Windows
pub async fn get_selection_for_mouse(app: AppHandle) -> Result<String, AppError> {
    // suspend shortcut handling to avoid interference
    let _guard = ShortcutHandlerGuard::suspend();

    // 1) native selection, then a short retry
    if let Ok(text) = platform::get_selection() {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    sleep(Duration::from_millis(30)).await;
    if let Ok(text) = platform::get_selection() {
        if !text.trim().is_empty() {
            return Ok(text);
        }
    }

    // 2) clipboard fallback with platform-specific shortcut
    #[cfg(target_os = "windows")]
    let shortcut = if platform::is_probably_terminal_focused().unwrap_or(false) {
        debug!("mouse selection: using terminal copy shortcut");
        CopyShortcut::Terminal
    } else {
        debug!("mouse selection: using standard copy shortcut");
        CopyShortcut::Standard
    };

    #[cfg(not(target_os = "windows"))]
    let shortcut = CopyShortcut::Standard;

    get_selection_fallback_with_shortcut(app, shortcut).await
}

/// Send copy shortcut key.
fn send_copy_keys(shortcut: CopyShortcut) -> Result<(), AppError> {
    let mut enigo_guard = ENIGO.lock()?;
    let enigo = enigo_guard.as_mut()?;

    // release modifier keys to avoid interference
    enigo.key(Key::Meta, Direction::Release)?;
    enigo.key(Key::Control, Direction::Release)?;
    enigo.key(Key::Alt, Direction::Release)?;
    enigo.key(Key::Shift, Direction::Release)?;

    // send Cmd+C or Ctrl+C
    #[cfg(target_os = "macos")]
    let modifier = Key::Meta;
    #[cfg(not(target_os = "macos"))]
    let modifier = Key::Control;

    match shortcut {
        CopyShortcut::Standard => {
            enigo.key(modifier, Direction::Press)?;
            enigo.key(Key::Unicode('c'), Direction::Click)?;
            enigo.key(modifier, Direction::Release)?;
        }
        CopyShortcut::Terminal => {
            #[cfg(target_os = "windows")]
            {
                enigo.key(Key::Control, Direction::Press)?;
                enigo.key(Key::Shift, Direction::Press)?;
                enigo.key(Key::Unicode('c'), Direction::Click)?;
                enigo.key(Key::Shift, Direction::Release)?;
                enigo.key(Key::Control, Direction::Release)?;
            }
            #[cfg(not(target_os = "windows"))]
            {
                enigo.key(modifier, Direction::Press)?;
                enigo.key(Key::Unicode('c'), Direction::Click)?;
                enigo.key(modifier, Direction::Release)?;
            }
        }
    }

    Ok(())
}
