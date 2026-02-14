use chrono::{NaiveDate, Local};
use rusqlite::{params, Connection, Result};
use crate::model::cow::{Cow, Sex, Breed, Category};
use std::str::FromStr;
use rusqlite::ToSql;
use crate::utils::cow_filter::CowFilter;






pub fn insert_cow(conn: &Connection, cow: &Cow) -> Result<i64> {
    if let Some(id) = cow.id {
        conn.execute(
            "INSERT INTO cows (id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                id,
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
        Ok(id)
    } else {
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
        "Select id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
         FROM cows 
         WHERE ear_tag = ?1",
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
        "Select id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
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
        "Select id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
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

pub fn get_cows_born_on_a_given_birth(conn: &Connection, birth_id: i64) -> Result<Vec<Cow>>{
    let mut stmt = conn.prepare(
        "Select id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id 
        From cows
        Where birth_id = ?1")?;
        let cow_iter = stmt.query_map(params![birth_id], |row| {
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

pub fn remove_birth_from_cows(conn: &Connection, birth_id: i64) -> Result<bool> {
    Ok(conn.execute("UPDATE cows SET birth_id = NULL WHERE birth_id = ?", params![birth_id])? != 0)
}

pub fn get_cows_filtered(conn: &Connection, filter: CowFilter) -> Result<Vec<Cow>, String> {
    let mut query = "SELECT id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id FROM cows WHERE 1=1".to_string();
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();

    let ref_date = filter.date.unwrap_or_else(|| Local::now().date_naive());

    if filter.show_only_entered {
        query.push_str(" AND entry_date <= ? AND (exit_date IS NULL OR exit_date > ?)");
        params.push(Box::new(ref_date));
        params.push(Box::new(ref_date));
    }

    if let Some(digits) = filter.last_4_digits_eartag {
        query.push_str(" AND ear_tag LIKE ?");
        params.push(Box::new(format!("%{}", digits)));
    }

    if let Some(s) = filter.sex {
        query.push_str(" AND sex = ?");
        params.push(Box::new(s.to_string()));
    }

    if let Some(b) = filter.breed {
        query.push_str(" AND breed = ?");
        params.push(Box::new(b.to_string()));
    }

    if let Some(cat) = filter.category {
        query.push_str(" AND category = ?");
        params.push(Box::new(cat.to_string()));
    }

    if let Some(year) = filter.born_in_year {
        query.push_str(" AND CAST(strftime('%Y', birth_date) AS INTEGER) = ?");
        params.push(Box::new(year));
    }

    if let Some(date) = filter.born_on {
        query.push_str(" AND birth_date = ?");
        params.push(Box::new(date));
    }

    if filter.births_less_than.is_some() || filter.births_more_than.is_some() {
        query.push_str(" AND sex = 'Female'"); 
        
        if let Some(lt) = filter.births_less_than {
            query.push_str(" AND (SELECT COUNT(*) FROM births WHERE mother_id = cows.id) < ?");
            params.push(Box::new(lt));
        }
        if let Some(mt) = filter.births_more_than {
            query.push_str(" AND (SELECT COUNT(*) FROM births WHERE mother_id = cows.id) > ?");
            params.push(Box::new(mt));
        }
    }

    if filter.inseminations_less_than.is_some() || filter.inseminations_more_than.is_some() {
        let count_sql = "
            SELECT COUNT(*) FROM inseminations 
            WHERE (sex = 'Female' AND dam_id = cows.id) 
               OR (sex = 'Male' AND sire_id = cows.id)
        ";

        if let Some(lt) = filter.inseminations_less_than {
            query.push_str(&format!(" AND ({}) < ?", count_sql));
            params.push(Box::new(lt));
        }
        if let Some(mt) = filter.inseminations_more_than {
            query.push_str(&format!(" AND ({}) > ?", count_sql));
            params.push(Box::new(mt));
        }
    }

    if let Some(min_m) = filter.minimum_age_months {
        query.push_str(" AND birth_date <= date(?, ?)");
        params.push(Box::new(ref_date));
        params.push(Box::new(format!("-{} months", min_m)));
    }
    if let Some(max_m) = filter.maximum_age_months {
        query.push_str(" AND birth_date >= date(?, ?)");
        params.push(Box::new(ref_date));
        params.push(Box::new(format!("-{} months", max_m)));
    }

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let cow_iter = stmt.query_map(&param_refs[..], |row| {
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
    }).map_err(|e| e.to_string())?;

    let results: Result<Vec<Cow>, _> = cow_iter.collect();
    results.map_err(|e| e.to_string())
}