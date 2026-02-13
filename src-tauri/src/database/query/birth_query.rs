use rusqlite::{params, Connection, Result};
use crate::model::birth::Birth;

pub fn insert_birth(conn: &Connection, birth: &Birth) -> Result<i64> {
    if let Some(id) = birth.id {
        conn.execute(
            "INSERT INTO births (id, mother_id, date) 
             VALUES (?1, ?2, ?3)",
            params![
                id,
                birth.mother_id,
                birth.date
            ],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO births (mother_id, date) 
             VALUES (?1, ?2)",
            params![
                birth.mother_id,
                birth.date
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

pub fn update_birth(conn: &Connection, birth: &Birth) -> Result<bool> {
    Ok(conn.execute(
        "UPDATE births SET mother_id = ?1, date = ?2 WHERE id = ?3",
        params![
            birth.mother_id,
            birth.date,
            birth.id
        ],
    )? != 0)
}

pub fn delete_birth(conn: &Connection, id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM births WHERE id = ?", params![id])? != 0)
}

pub fn get_births(conn: &Connection) -> Result<Vec<Birth>> {
    let mut stmt = conn.prepare(
        "SELECT id, mother_id, date FROM births"
    )?;

    let birth_iter = stmt.query_map([], |row| {
        Ok(Birth {
            id: row.get(0)?,
            mother_id: row.get(1)?,
            date: row.get(2)?,
        })
    })?;
    birth_iter.collect()
}

pub fn get_birth(conn: &Connection, id: i64) -> Result<Birth> {
    conn.query_row(
        "SELECT id, mother_id, date 
         FROM births 
         WHERE id = ?1",
        params![id],
        |row| {
            Ok(Birth {
                id: row.get(0)?,
                mother_id: row.get(1)?,
                date: row.get(2)?
            })
        },
    )
}

pub fn get_birth_by_mother_and_date(conn: &Connection, mother_id: i64, date: &str) -> Result<Birth> {
    conn.query_row(
        "SELECT id, mother_id, date 
         FROM births 
         WHERE mother_id = ?1
         AND date = ?2",
        params![mother_id, date],
        |row| {
            Ok(Birth {
                id: row.get(0)?,
                mother_id: row.get(1)?,
                date: row.get(2)?
            })
        },
    )
}

pub fn get_births_by_mother(conn: &Connection, mother_id: i64) -> Result<Vec<Birth>> {
    let mut stmt = conn.prepare(
        "SELECT id, mother_id, date 
         FROM births 
         WHERE mother_id = ?1",
    )?;
    let birth_iter = stmt.query_map(params![mother_id], |row| {
        Ok(Birth {
            id: row.get(0)?,
            mother_id: row.get(1)?,
            date: row.get(2)?
        })
    })?;
    birth_iter.collect()
}

pub fn delete_births_by_mother(conn: &Connection, mother_id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM births WHERE mother_id = ?", params![mother_id])? != 0)
}