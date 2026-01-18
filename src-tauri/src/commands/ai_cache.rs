use crate::error::AppError;
use rusqlite::{params, Connection, OptionalExtension};
use std::fs;
use std::path::PathBuf;
use tauri::{AppHandle, Manager};

const DB_FILENAME: &str = "ai_cache.sqlite3";

fn normalize_prompt(prompt: &str) -> String {
    fn is_trim_char(c: char) -> bool {
        c.is_whitespace()
            || c.is_control()
            || matches!(
                c,
                '\u{200b}' | '\u{200c}' | '\u{200d}' | '\u{2060}' | '\u{feff}'
            )
    }

    prompt.trim_matches(is_trim_char).to_string()
}

fn db_path(app: &AppHandle) -> Result<PathBuf, AppError> {
    let dir = app.path().app_data_dir()?;
    fs::create_dir_all(&dir)?;
    Ok(dir.join(DB_FILENAME))
}

fn init_db(conn: &Connection) -> Result<(), AppError> {
    conn.execute_batch(
        r#"
        CREATE TABLE IF NOT EXISTS ai_cache (
          prompt TEXT PRIMARY KEY,
          response   TEXT NOT NULL,
          created_at INTEGER NOT NULL,
          updated_at INTEGER NOT NULL
        );
        "#,
    )?;
    Ok(())
}

/// Get cached AI response for the given prompt (entry.result).
#[tauri::command]
pub async fn ai_cache_get(app: AppHandle, prompt: String) -> Result<Option<String>, AppError> {
    let prompt = normalize_prompt(&prompt);
    if prompt.is_empty() {
        return Ok(None);
    }

    let path = db_path(&app)?;
    let result = tokio::task::spawn_blocking(move || -> Result<Option<String>, AppError> {
        let conn = Connection::open(path)?;
        init_db(&conn)?;

        let mut stmt = conn.prepare("SELECT response FROM ai_cache WHERE prompt = ?1")?;
        let cached = stmt
            .query_row([prompt], |row| row.get::<_, String>(0))
            .optional()?;
        Ok(cached)
    })
    .await??;

    Ok(result)
}

/// Store (or overwrite) AI response for the given prompt (entry.result).
#[tauri::command]
pub async fn ai_cache_set(
    app: AppHandle,
    prompt: String,
    response: String,
) -> Result<(), AppError> {
    let prompt = normalize_prompt(&prompt);
    if prompt.is_empty() {
        return Ok(());
    }

    let path = db_path(&app)?;
    tokio::task::spawn_blocking(move || -> Result<(), AppError> {
        let conn = Connection::open(path)?;
        init_db(&conn)?;

        conn.execute(
            r#"
            INSERT INTO ai_cache (prompt, response, created_at, updated_at)
            VALUES (?1, ?2, CAST(strftime('%s','now') AS INTEGER), CAST(strftime('%s','now') AS INTEGER))
            ON CONFLICT(prompt) DO UPDATE SET
              response   = excluded.response,
              updated_at = excluded.updated_at
            "#,
            params![prompt, response],
        )?;

        Ok(())
    })
    .await??;

    Ok(())
}
