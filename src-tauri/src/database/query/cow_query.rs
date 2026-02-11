use chrono::NaiveDate;
use rusqlite::{params, Connection, Result};
use crate::model::cow::{Cow, Sex, Breed, Category};
use std::str::FromStr;


pub fn insert_cow(conn: &Connection, cow: &Cow) -> Result<i64> {
    conn.execute(
        "INSERT INTO cows (ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id) 
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            cow.ear_tag,
            cow.sex.to_string(),
            cow.breed.to_string(),
            cow.category.to_string(),
            cow.birth_date,
            cow.entry_date,
            cow.exit_date,
            cow.birth_id
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update_cow(conn: &Connection, cow: &Cow) -> Result<bool> {
    Ok(conn.execute(
        "UPDATE cows SET ear_tag = ?1, sex = ?2, breed = ?3, category = ?4, birth_date = ?5, entry_date = ?6, exit_date = ?7, birth_id = ?8 WHERE id = ?9",
        params![
            cow.ear_tag,
            cow.sex.to_string(),
            cow.breed.to_string(),
            cow.category.to_string(),
            cow.birth_date,
            cow.entry_date,
            cow.exit_date,
            cow.birth_id,
            cow.id
        ],
    )? != 0)
}

pub fn delete_cow(conn: &Connection, id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM cows WHERE id = ?", params![id])? != 0)
}

pub fn get_cows(conn: &Connection) -> Result<Vec<Cow>> {
    let mut stmt = conn.prepare(
        "SELECT id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id FROM cows"
    )?;

    let cow_iter = stmt.query_map([], |row| {
        let sex_str: String = row.get(2)?;
        let breed_str: String = row.get(3)?;
        let cat_str: String = row.get(4)?;

        Ok(Cow {
            id: row.get(0)?,
            ear_tag: row.get(1)?,
            sex: Sex::from_str(&sex_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            breed: Breed::from_str(&breed_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            category: Category::from_str(&cat_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            
            birth_date: row.get(5)?,
            entry_date: row.get(6)?,
            exit_date: row.get(7)?,
            birth_id: row.get(8)?,
        })
    })?;
    cow_iter.collect()
}

pub fn get_cow(conn: &Connection, id: i64) -> Result<Cow> {
    conn.query_row(
        "SELECT id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
         FROM cows 
         WHERE id = ?1",
        params![id],
        |row| {
            let sex_str: String = row.get(2)?;
            let breed_str: String = row.get(3)?;
            let cat_str: String = row.get(4)?;

            Ok(Cow {
                id: row.get(0)?,
                ear_tag: row.get(1)?,
                sex: Sex::from_str(&sex_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
                breed: Breed::from_str(&breed_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
                category: Category::from_str(&cat_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
                
                birth_date: row.get(5)?,
                entry_date: row.get(6)?,
                exit_date: row.get(7)?,
                birth_id: row.get(8)?,
            })
        },
    )
}

pub fn get_cow_by_eartag(conn: &Connection, ear_tag: &str) -> Result<Cow> {
    conn.query_row(
        "Select id, eartag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
         FROM cows 
         WHERE eartag = ?1",
        params![ear_tag],
        |row| {
            let sex_str: String = row.get(2)?;
            let breed_str: String = row.get(3)?;
            let cat_str: String = row.get(4)?;
            Ok(Cow {
                id: row.get(0)?,
                ear_tag: row.get(1)?,
                sex: Sex::from_str(&sex_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
                breed: Breed::from_str(&breed_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
                category: Category::from_str(&cat_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?, 
                birth_date: row.get(5)?,
                entry_date: row.get(6)?,
                exit_date: row.get(7)?,
                birth_id: row.get(8)?,
            })
        },
    )
}

pub fn get_unassigned_calves_on_date(conn: &Connection, date: &NaiveDate) -> Result<Vec<Cow>>{
    let mut stmt = conn.prepare(
        "Select id, eartag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
        From cows
        Where birth_date = ?1
        And birth_id is null")?;
    let cow_iter = stmt.query_map(params![date], |row| {
        let sex_str: String = row.get(2)?;
        let breed_str: String = row.get(3)?;
        let cat_str: String = row.get(4)?;
        Ok(Cow {
            id: row.get(0)?,
            ear_tag: row.get(1)?,
            sex: Sex::from_str(&sex_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            breed: Breed::from_str(&breed_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            category: Category::from_str(&cat_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            birth_date: row.get(5)?,
            entry_date: row.get(6)?,
            exit_date: row.get(7)?,
            birth_id: row.get(8)?,
        })
    })?;
    cow_iter.collect()
}

pub fn get_cows_in_the_plantation(conn: &Connection, date: &NaiveDate) -> Result<Vec<Cow>>{
    let mut stmt = conn.prepare(
        "Select id, eartag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
        From cows
        Where entry_date <= ?1
        And (exit_date > ?1 OR exit_date is null)")?;
    let cow_iter = stmt.query_map(params![date], |row| {
        let sex_str: String = row.get(2)?;
        let breed_str: String = row.get(3)?;
        let cat_str: String = row.get(4)?;
        Ok(Cow {
            id: row.get(0)?,
            ear_tag: row.get(1)?,
            sex: Sex::from_str(&sex_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            breed: Breed::from_str(&breed_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            category: Category::from_str(&cat_str).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
            birth_date: row.get(5)?,
            entry_date: row.get(6)?,
            exit_date: row.get(7)?,
            birth_id: row.get(8)?,
        })
    })?;
    cow_iter.collect()
}