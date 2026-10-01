use super::session::AppState;
use serde::Serialize;

#[derive(Serialize)]
pub struct SavedAccount {
    pub id: i64,
    pub username: String,
    pub farm_name: String,
}

fn remember(conn: &rusqlite::Connection, user_id: i64) -> Result<(), String> {
    conn.execute("INSERT OR IGNORE INTO remembered_accounts (user_id) SELECT id FROM users WHERE id = ?1", [user_id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn activate(conn: &rusqlite::Connection, user_id: i64) -> Result<(), String> {
    let changed = conn.execute(
        "UPDATE app_settings SET active_user_id = ?1 WHERE id = 1 AND EXISTS (
            SELECT 1 FROM remembered_accounts r JOIN users u ON u.id = r.user_id WHERE u.id = ?1
        )", [user_id],
    ).map_err(|e| e.to_string())?;
    if changed != 1 { return Err("Autentific\u{0103}-te mai \u{00ee}nt\u{00e2}i cu parola acestui cont.".into()); }
    Ok(())
}

fn list(conn: &rusqlite::Connection) -> Result<Vec<SavedAccount>, String> {
    let mut statement = conn.prepare(
        "SELECT u.id, u.username, f.name FROM remembered_accounts r
         JOIN users u ON u.id = r.user_id JOIN farms f ON f.id = u.farm_id
         ORDER BY u.username COLLATE NOCASE",
    ).map_err(|e| e.to_string())?;
    let rows = statement.query_map([], |r| Ok(SavedAccount {
        id: r.get(0)?, username: r.get(1)?, farm_name: r.get(2)?,
    })).map_err(|e| e.to_string())?;
    rows.collect::<Result<Vec<_>, _>>().map_err(|e| e.to_string())
}

pub fn end_session(conn: &rusqlite::Connection, keep_remembered: bool) -> Result<(), String> {
    let tx = conn.unchecked_transaction().map_err(|e| e.to_string())?;
    if !keep_remembered {
        tx.execute("DELETE FROM remembered_accounts WHERE user_id = (SELECT active_user_id FROM app_settings WHERE id = 1)", [])
            .map_err(|e| e.to_string())?;
    }
    tx.execute("UPDATE app_settings SET active_user_id = NULL WHERE id = 1", []).map_err(|e| e.to_string())?;
    tx.commit().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn saved_accounts(state: tauri::State<'_, AppState>) -> Result<Vec<SavedAccount>, String> {
    let conn = state.db_pool.get().map_err(|e| e.to_string())?;
    list(&conn)
}

// Persist only account IDs, and only after an authenticated session exists.

#[tauri::command]
pub fn remember_account(state: tauri::State<'_, AppState>) -> Result<(), String> {
    let guard = state.session.lock().map_err(|e| e.to_string())?;
    let user_id = guard.as_ref().ok_or("No session found")?.user_id;
    let conn = state.db_pool.get().map_err(|e| e.to_string())?;
    remember(&conn, user_id)
}

#[tauri::command]
pub fn switch_account(state: tauri::State<'_, AppState>, user_id: i64) -> Result<crate::controller::controller::SessionInfo, String> {
    {
        let mut guard = state.session.lock().map_err(|e| e.to_string())?;
        let conn = state.db_pool.get().map_err(|e| e.to_string())?;
        activate(&conn, user_id)?;
        *guard = None;
    }
    crate::controller::controller::get_session(state)?.ok_or("No session found".into())
}

#[tauri::command]
pub fn forget_account(state: tauri::State<'_, AppState>, user_id: i64) -> Result<(), String> {
    let conn = state.db_pool.get().map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM remembered_accounts WHERE user_id = ?1", [user_id]).map_err(|e| e.to_string())?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accounts_survive_reopening_until_explicit_logout() {
        let path = std::env::temp_dir().join(format!("remembered-{}-{}.sqlite", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            crate::database::database::create_tables(&conn).unwrap();
            conn.execute("INSERT INTO farms (id, name) VALUES (1, 'Farm one'), (2, 'Farm two')", []).unwrap();
            conn.execute("INSERT INTO users (id, username, email, password_hash, role, farm_id) VALUES (1, 'one', 'one@test', 'hash1', 'Admin', 1), (2, 'two', 'two@test', 'hash2', 'Viewer', 2)", []).unwrap();
            assert!(activate(&conn, 2).is_err());
            remember(&conn, 1).unwrap();
            remember(&conn, 2).unwrap();
            remember(&conn, 2).unwrap();
            activate(&conn, 1).unwrap();
            end_session(&conn, true).unwrap();
            assert_eq!(list(&conn).unwrap().len(), 2);
        }
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            crate::database::database::create_tables(&conn).unwrap();
            assert_eq!(list(&conn).unwrap().len(), 2);
            activate(&conn, 2).unwrap();
            let farm: i64 = conn.query_row("SELECT farm_id FROM users WHERE id = (SELECT active_user_id FROM app_settings WHERE id = 1)", [], |r| r.get(0)).unwrap();
            assert_eq!(farm, 2);
            end_session(&conn, false).unwrap();
            assert!(activate(&conn, 2).is_err());
            assert_eq!(list(&conn).unwrap().len(), 1);
            activate(&conn, 1).unwrap();
            assert!(activate(&conn, 99).is_err());
            let active: i64 = conn.query_row("SELECT active_user_id FROM app_settings WHERE id = 1", [], |r| r.get(0)).unwrap();
            assert_eq!(active, 1);
        }
        {
            let conn = rusqlite::Connection::open(&path).unwrap();
            crate::database::database::create_tables(&conn).unwrap();
            assert!(activate(&conn, 2).is_err());
            conn.execute("DELETE FROM users WHERE id = 1", []).unwrap();
            assert!(list(&conn).unwrap().is_empty());
        }
        std::fs::remove_file(path).unwrap();
    }
}