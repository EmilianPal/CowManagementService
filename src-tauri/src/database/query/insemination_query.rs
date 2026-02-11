use rusqlite::{params, Connection, Result};
use crate::model::insemination::Insemination;

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

pub fn get_inseminations(conn: &Connection) -> Result<Vec<Insemination>> {
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

pub fn get_insemination(conn: &Connection, id: i64) -> Result<Insemination> {
    conn.query_row(
        "SELECT id, dam_id, sire_id, date 
         FROM inseminations 
         WHERE id = ?1",
        params![id],
        |row| {
        
            Ok(Insemination {
                id: row.get(0)?,
                dam_id: row.get(1)?,
                sire_id: row.get(2)?,
                date: row.get(3)?
            })
        },
    )
}

pub fn get_inseminations_by_dam(conn: &Connection, dam_id: i64) -> Result<Vec<Insemination>> {
    let mut stmt = conn.prepare(
        "SELECT id, dam_id, sire_id, date 
         FROM inseminations 
         WHERE dam_id = ?1",
    )?;
    let insemination_iter = stmt.query_map(params![dam_id], |row| {
        Ok(Insemination {
            id: row.get(0)?,
            dam_id: row.get(1)?,
            sire_id: row.get(2)?,
            date: row.get(3)?
        })
    })?;
    insemination_iter.collect()
}

pub fn get_inseminations_by_sire(conn: &Connection, sire_id: i64) -> Result<Vec<Insemination>> {
    let mut stmt = conn.prepare(
        "SELECT id, dam_id, sire_id, date 
         FROM inseminations 
         WHERE sire_id = ?1",
    )?;
    let insemination_iter = stmt.query_map(params![sire_id], |row| {
        Ok(Insemination {
            id: row.get(0)?,
            dam_id: row.get(1)?,
            sire_id: row.get(2)?,
            date: row.get(3)?
        })
    })?;
    insemination_iter.collect()
}



pub fn get_insemination_by_dam_and_date(conn: &Connection, dam_id: i64, date: &str) -> Result<Insemination> {
    conn.query_row(
        "
        SELECT id, dam_id, sire_id, date 
        FROM inseminations 
        WHERE dam_id = ?1
        AND date = ?2",
        params![dam_id, date],
        |row| {
            Ok(Insemination {
                id: row.get(0)?,
                dam_id: row.get(1)?,
                sire_id: row.get(2)?,
                date: row.get(3)?
            })
        },
    )
}

pub fn delete_insemination_by_dam(conn: &Connection, dam_id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM inseminations WHERE dam_id = ?", params![dam_id])? != 0)
}

pub fn remove_sire_from_inseminations(conn: &Connection, sire_id: i64) -> Result<bool> {
    Ok(conn.execute("UPDATE inseminations SET sire_id = NULL WHERE sire_id = ?", params![sire_id])? != 0)
}



