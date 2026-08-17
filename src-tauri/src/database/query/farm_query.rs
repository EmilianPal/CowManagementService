use crate::model::farm::Farm;
use rusqlite::{params, Connection, Result};



pub fn insert_farm(conn: &Connection, farm: &Farm) -> Result<i64> {
    if let Some(id) = farm.id {
        conn.execute(
        "INSERT INTO farm (id, name) Values (?1, ?2)",
        params![farm.id, farm.name],
        )?;
        Ok(id)
    }
    else{
        conn.execute(
        "INSERT INTO farm (name) Values (?1)",
        params![farm.name],
    )?;
    Ok(conn.last_insert_rowid())
    }
    
}