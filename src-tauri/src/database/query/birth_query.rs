use rusqlite::{params, Connection, Result};
use crate::model::{birth::Birth};

pub fn insert_birth(conn: &Connection, birth: &Birth, farm_id: i64) -> Result<i64> {
    if let Some(id) = birth.id {
        conn.execute(
            "INSERT INTO births (id, mother_id, date, farm_id) 
             VALUES (?1, ?2, ?3, ?4)",
            params![
                id,
                birth.mother_id,
                birth.date,
                farm_id
            ],
        )?;
        Ok(id)
    } else {
        conn.execute(
            "INSERT INTO births (mother_id, date, farm_id) 
             VALUES (?1, ?2, ?3)",
            params![
                birth.mother_id,
                birth.date,
                farm_id
            ],
        )?;
        Ok(conn.last_insert_rowid())
    }
}

pub fn update_birth(conn: &Connection, birth: &Birth, farm_id: i64) -> Result<bool> {
    Ok(conn.execute(
        "UPDATE births SET mother_id = ?1, date = ?2, farm_id = ?3 WHERE id = ?4 AND farm_id = ?5",
        params![
            birth.mother_id,
            birth.date,
            farm_id,
            birth.id,
            farm_id
        ],
    )? != 0)
}

pub fn delete_birth(conn: &Connection, id: i64, farm_id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM births WHERE id = ? AND farm_id = ?", params![id, farm_id])? != 0)
}

pub fn get_births(conn: &Connection, farm_id: i64) -> Result<Vec<Birth>> {
    let mut stmt = conn.prepare(
        "SELECT id, mother_id, date, farm_id FROM births WHERE farm_id = ?"
    )?;

    let birth_iter = stmt.query_map(params![farm_id], |row| {
        Ok(Birth {
            id: row.get(0)?,
            mother_id: row.get(1)?,
            date: row.get(2)?,
            farm_id: row.get(3)?,
        })
    })?;
    birth_iter.collect()
}

pub fn get_birth(conn: &Connection, id: i64, farm_id: i64) -> Result<Birth> {
    conn.query_row(
        "SELECT id, mother_id, date 
         FROM births 
         WHERE id = ?1 AND farm_id = ?2",
        params![id, farm_id],
        |row| {
            Ok(Birth {
                id: row.get(0)?,
                mother_id: row.get(1)?,
                date: row.get(2)?,
                farm_id: row.get(3)?,
            })
        },
    )
}

pub fn get_birth_by_mother_and_date(conn: &Connection, mother_id: i64, date: &str, farm_id: i64) -> Result<Birth> {
    conn.query_row(
        "SELECT id, mother_id, date, farm_id
         FROM births
         WHERE mother_id = ?1
         AND date = ?2
         AND farm_id = ?3",
        params![mother_id, date, farm_id],
        |row| {
            Ok(Birth {
                id: row.get(0)?,
                mother_id: row.get(1)?,
                date: row.get(2)?,
                farm_id: row.get(3)?,
            })
        },
    )
}

pub fn get_births_by_mother(conn: &Connection, mother_id: i64, farm_id: i64) -> Result<Vec<Birth>> {
    let mut stmt = conn.prepare(
        "SELECT id, mother_id, date, farm_id
         FROM births 
         WHERE mother_id = ?1 AND farm_id = ?2",
    )?;
    let birth_iter = stmt.query_map(params![mother_id, farm_id], |row| {
        Ok(Birth {
            id: row.get(0)?,
            mother_id: row.get(1)?,
            date: row.get(2)?,
            farm_id: row.get(3)?
        })
    })?;
    birth_iter.collect()
}

pub fn delete_births_by_mother(conn: &Connection, mother_id: i64, farm_id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM births WHERE mother_id = ? AND farm_id = ?", params![mother_id, farm_id])? != 0)
}