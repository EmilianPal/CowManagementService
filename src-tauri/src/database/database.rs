use rusqlite::{Connection, Result};
use tauri::AppHandle;
use std::fs;
use tauri::Manager;

pub fn init_db(app_handle: &AppHandle) -> Result<Connection, String> {
    let app_dir = app_handle.path().app_data_dir()
        .expect("The app data directory should exist.");
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    let db_path = app_dir.join("management.db");

    let conn = Connection::open(db_path).map_err(|e| e.to_string())?;

    conn.execute("PRAGMA foreign_keys = ON", []).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS births (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mother_id INTEGER NOT NULL,
            date TEXT NOT NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS cows (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            ear_tag TEXT NOT NULL UNIQUE,
            sex TEXT NOT NULL,
            breed TEXT NOT NULL,
            category TEXT NOT NULL,
            birth_date TEXT NOT NULL,
            entry_date TEXT NOT NULL,
            exit_date TEXT,
            birth_id INTEGER,
            FOREIGN KEY (birth_id) REFERENCES births (id) ON DELETE SET NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS inseminations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            dam_id INTEGER NOT NULL,
            sire_id INTEGER,
            date TEXT NOT NULL,
            FOREIGN KEY (dam_id) REFERENCES cows (id) ON DELETE CASCADE,
            FOREIGN KEY (sire_id) REFERENCES cows (id) ON DELETE SET NULL
        )",
        [],
    ).map_err(|e| e.to_string())?;

    Ok(conn)
}