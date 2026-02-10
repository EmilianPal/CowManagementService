use rusqlite::{params, Connection, Result};
use crate::model::Insemination;

pub fn insert_insemination(conn: &Connection, insemination: &Insemination) -> Result<i64> {
    conn.execute(
        "INSERT INTO inseminations (dam_id, sire_id, date)
         VALUES (?1, ?2, ?3)",
        params![
            insemination.dam_id,
            insemination.sire_id,
            insemination.date
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_insemination(conn: &Connection, insemination: &Insemination) -> Result<bool> {
    Ok(conn.execute(
        "UPDATE inseminations SET dam_id = ?1, sire_id = ?2, date = ?3 WHERE id = ?4",
        params![
            insemination.dam_id,
            insemination.sire_id,
            insemination.date,
            insemination.id
        ],
    )? != 0)
}

pub fn delete_insemination(conn: &Connection, id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM inseminations WHERE id = ?", params![id])? != 0)
}

pub fn get_insemination(conn: &Connection) -> Result<Vec<Insemination>> {
    let mut stmt = conn.prepare(
        "SELECT id, dam_id, sire_id, date FROM inseminations"
    )?;

    let insemination_iter = stmt.query_map([], |row| {
        Ok(Insemination {
            id: row.get(0)?,
            dam_id: row.get(1)?,
            sire_id: row.get(2)?,
            date: row.get(3)?,
        })
    })?;
    insemination_iter.collect()
}