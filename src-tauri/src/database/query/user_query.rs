use rusqlite::{Connection, OptionalExtension, Result, params};
use crate::auth::user::{User, Role};
use std::str::FromStr;

pub fn insert_user(conn: &Connection, user: &User) -> Result<i64> {
    if let Some(id) = user.id {
        conn.execute(
            "INSERT INTO users (id, username, email, password_hash, role, farm_id) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                user.id,
                user.username,
                user.email,
                user.password_hash,
                user.role.to_string(),
                user.farm_id
            ],
        )?;
        Ok(id)
    }
    else {
        conn.execute(
            "INSERT INTO users (username, email, password_hash, role, farm_id) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                user.username,
                user.email,
                user.password_hash,
                user.role.to_string(),
                user.farm_id
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}


pub fn get_user_by_username(conn: &Connection, username: &str) -> Result<Option<User>, String> {
    conn.query_row(
        "SELECT id, username, email, password_hash, role, farm_id FROM users WHERE username = ?",
        params![username],
        |row| {
            let role_string : String = row.get(4)?;
            Ok(User{
                id: row.get(0)?,
                username: row.get(1)?,
                email: row.get(2)?,
                role: Role::from_str(&role_string).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
                password_hash: row.get(3)?,
                
                farm_id: row.get(5)?
            })
        }
    ).optional()
    .map_err(|e| e.to_string())
}


