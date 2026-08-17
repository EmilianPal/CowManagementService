use rusqlite::{Connection, Result};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use tauri::AppHandle;
use std::fs;
use tauri::Manager;

pub fn init_db(app_handle: &AppHandle) -> Result<Pool<SqliteConnectionManager>, String> {
    let app_dir = app_handle.path().app_data_dir()
        .expect("The app data directory should exist.");
    
    fs::create_dir_all(&app_dir).map_err(|e| e.to_string())?;
    let db_path = app_dir.join("cow_management_service.db");

    let manager = SqliteConnectionManager::file(db_path)
        .with_init(|c| c.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA foreign_keys = ON;"
        ));

    let pool = Pool::new(manager).map_err(|e| e.to_string())?;

    let conn = pool.get().map_err(|e| e.to_string())?;
    create_tables(&conn).map_err(|e| e.to_string())?;

    Ok(pool)
}

pub fn create_tables(conn: &Connection) -> Result<()> {
    conn.execute("PRAGMA foreign_keys = ON", [])?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS farms (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE
        )", 
        []
    )?;

        conn.execute("CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            role TEXT NOT NULL,
            farm_id INTEGER,
            FOREIGN KEY (farm_id) REFERENCES farms (id) ON DELETE CASCADE
        )",
         [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            active_user_id INTEGER NULL,
            FOREIGN KEY (active_user_id) REFERENCES users (id) ON DELETE SET NULL
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS births (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            mother_id INTEGER NOT NULL,
            date TEXT NOT NULL,
            farm_id INTEGER NOT NULL,
            FOREIGN KEY (mother_id) REFERENCES cows (id) ON DELETE CASCADE,
            FOREIGN KEY (farm_id) REFERENCES farms (id) ON DELETE CASCADE,
            UNIQUE (mother_id, date)
        )",
        [],
    )?;

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
            farm_id INTEGER NOT NULL,
            FOREIGN KEY (birth_id) REFERENCES births (id) ON DELETE SET NULL,
            FOREIGN KEY (farm_id) REFERENCES farms (id) ON DELETE CASCADE
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS inseminations (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            dam_id INTEGER NOT NULL,
            sire_id INTEGER,
            date TEXT NOT NULL,
            farm_id INTEGER NOT NULL,
            FOREIGN KEY (dam_id) REFERENCES cows (id) ON DELETE CASCADE,
            FOREIGN KEY (sire_id) REFERENCES cows (id) ON DELETE SET NULL,
            FOREIGN KEY (farm_id) REFERENCES farms (id) ON DELETE CASCADE,
            UNIQUE (dam_id, date)
        )",
        [],
    )?;

    conn.execute(
        "INSERT OR IGNORE INTO app_settings (id, active_user_id) VALUES (1, NULL);",
        [],
    )?;

    Ok(())
}
