use chrono::{NaiveDate, Local};
use rusqlite::{params, Connection, Result};
use crate::model::cow::{Cow, Sex, Breed, Category};
use std::str::FromStr;
use rusqlite::ToSql;
use crate::utils::cow_filter::CowFilter;






pub fn insert_cow(conn: &Connection, cow: &Cow, farm_id: i64) -> Result<i64> {
    if farm_id != cow.farm_id {
        return Ok(-1); 
    }
    if let Some(id) = cow.id {
        conn.execute(
            "INSERT INTO cows (id, farm_id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                cow.farm_id,
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
            "INSERT INTO cows (farm_id, ear_tag, sex, breed, category, birth_date, entry_date, exit_date, birth_id) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                cow.farm_id,
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

pub fn update_cow(conn: &Connection, cow: &Cow, farm_id: i64) -> Result<bool> {
    Ok(conn.execute(
        "UPDATE cows SET farm_id = ?1, ear_tag = ?2, sex = ?3, breed = ?4, category = ?5, birth_date = ?6, entry_date = ?7, exit_date = ?8, birth_id = ?9 WHERE id = ?10 AND farm_id = ?11",
        params![
            cow.farm_id,
            cow.ear_tag,
            cow.sex.to_string(),
            cow.breed.to_string(),
            cow.category.to_string(),
            cow.birth_date,
            cow.entry_date,
            cow.exit_date,
            cow.birth_id,
            cow.id,
            farm_id
        ],
    )? != 0)
}

pub fn delete_cow(conn: &Connection, id: i64, farm_id: i64) -> Result<bool> {
    Ok(conn.execute("DELETE FROM cows WHERE id = ? AND farm_id = ?", params![id, farm_id])? != 0)
}

const COW_SELECT: &str = "SELECT cows.id, cows.farm_id, cows.ear_tag, cows.sex, cows.breed,
    cows.category, cows.birth_date, cows.entry_date, cows.exit_date, cows.birth_id,
    (SELECT COUNT(*) FROM births WHERE mother_id = cows.id) AS birth_count,
    CASE WHEN cows.sex = 'Female'
        THEN (SELECT COUNT(*) FROM inseminations WHERE dam_id = cows.id)
        ELSE (SELECT COUNT(*) FROM inseminations WHERE sire_id = cows.id)
    END AS insemination_count
    FROM cows";

fn cow_from_row(row: &rusqlite::Row<'_>) -> Result<Cow> {
    let sex: String = row.get(3)?;
    let breed: String = row.get(4)?;
    let category: String = row.get(5)?;
    Ok(Cow {
        id: row.get(0)?,
        farm_id: row.get(1)?,
        ear_tag: row.get(2)?,
        sex: Sex::from_str(&sex).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
        breed: Breed::from_str(&breed).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
        category: Category::from_str(&category).map_err(|_| rusqlite::Error::ExecuteReturnedResults)?,
        birth_date: row.get(6)?,
        entry_date: row.get(7)?,
        exit_date: row.get(8)?,
        birth_id: row.get(9)?,
        birth_count: row.get(10)?,
        insemination_count: row.get(11)?,
    })
}

pub fn get_cows(conn: &Connection, farm_id: i64) -> Result<Vec<Cow>> {
    let mut stmt = conn.prepare(&format!("{COW_SELECT} WHERE cows.farm_id = ?1 ORDER BY cows.ear_tag"))?;
    let rows = stmt.query_map([farm_id], cow_from_row)?.collect();
    rows
}

pub fn get_cow(conn: &Connection, id: i64, farm_id: i64) -> Result<Cow> {
    conn.query_row(&format!("{COW_SELECT} WHERE cows.id = ?1 AND cows.farm_id = ?2"),
        params![id, farm_id], cow_from_row)
}

pub fn get_cow_by_eartag(conn: &Connection, ear_tag: &str, farm_id: i64) -> Result<Cow> {
    conn.query_row(&format!("{COW_SELECT} WHERE cows.ear_tag = ?1 AND cows.farm_id = ?2"),
        params![ear_tag, farm_id], cow_from_row)
}

pub fn get_unassigned_calves_on_date(conn: &Connection, date: &NaiveDate, farm_id: i64) -> Result<Vec<Cow>> {
    let mut stmt = conn.prepare(&format!("{COW_SELECT} WHERE cows.birth_date = ?1 AND cows.birth_id IS NULL AND cows.farm_id = ?2"))?;
    let rows = stmt.query_map(params![date, farm_id], cow_from_row)?.collect();
    rows
}

pub fn get_cows_in_the_plantation(conn: &Connection, date: &NaiveDate, farm_id: i64) -> Result<Vec<Cow>> {
    let mut stmt = conn.prepare(&format!("{COW_SELECT} WHERE cows.entry_date <= ?1 AND (cows.exit_date > ?1 OR cows.exit_date IS NULL) AND cows.farm_id = ?2"))?;
    let rows = stmt.query_map(params![date, farm_id], cow_from_row)?.collect();
    rows
}

pub fn get_cows_born_on_a_given_birth(conn: &Connection, birth_id: i64, farm_id: i64) -> Result<Vec<Cow>> {
    let mut stmt = conn.prepare(&format!("{COW_SELECT} WHERE cows.birth_id = ?1 AND cows.farm_id = ?2"))?;
    let rows = stmt.query_map(params![birth_id, farm_id], cow_from_row)?.collect();
    rows
}

pub fn remove_birth_from_cows(conn: &Connection, birth_id: i64, farm_id: i64) -> Result<bool> {
    Ok(conn.execute("UPDATE cows SET birth_id = NULL WHERE birth_id = ? AND farm_id = ?", params![birth_id, farm_id])? != 0)
}

pub fn get_cows_filtered(conn: &Connection, filter: CowFilter, farm_id: i64) -> Result<Vec<Cow>, String> {
    for year in [filter.entered_in_year, filter.exited_in_year, filter.born_in_year].into_iter().flatten() {
        if !(1..=9999).contains(&year) {
            return Err("Anul trebuie să fie cuprins între 1 și 9999.".into());
        }
    }
    if filter.minimum_age_months.is_some_and(|n| n < 0) || filter.maximum_age_months.is_some_and(|n| n < 0) {
        return Err("Vârsta în luni nu poate fi negativă.".into());
    }
    if let (Some(min), Some(max)) = (filter.minimum_age_months, filter.maximum_age_months) {
        if min >= max { return Err("Valoarea pentru „Vârsta peste” trebuie să fie mai mică decât cea pentru „Vârsta sub”.".into()); }
    }
    let mut query = format!("{COW_SELECT} WHERE cows.farm_id = ?");
    let mut params: Vec<Box<dyn ToSql>> = Vec::new();
    params.push(Box::new(farm_id) as Box<dyn ToSql>);

    let ref_date = filter.date.unwrap_or_else(|| Local::now().date_naive());

    if filter.show_only_entered {
        query.push_str(" AND entry_date <= ? AND (exit_date IS NULL OR exit_date > ?)");
        params.push(Box::new(ref_date) as Box<dyn ToSql>);
        params.push(Box::new(ref_date) as Box<dyn ToSql>);
    }

    if filter.show_only_exited {
        query.push_str(" AND exit_date IS NOT NULL AND exit_date <= ?");
        params.push(Box::new(ref_date));
    }
    if let Some(text) = filter.ear_tag_contains.as_ref().filter(|s| !s.trim().is_empty()) {
        query.push_str(" AND instr(upper(ear_tag), upper(?)) > 0");
        params.push(Box::new(text.trim().to_owned()));
    }
    if let Some(date) = filter.entered_on {
        query.push_str(" AND entry_date = ?");
        params.push(Box::new(date));
    }
    for (year, column) in [(filter.entered_in_year, "entry_date"), (filter.exited_in_year, "exit_date")] {
        if let Some(year) = year {
            query.push_str(&format!(" AND {column} >= ? AND {column} <= ?"));
            params.push(Box::new(format!("{year:04}-01-01")));
            params.push(Box::new(format!("{year:04}-12-31")));
        }
    }
    if let Some(date) = filter.exited_on {
        query.push_str(" AND exit_date = ?");
        params.push(Box::new(date));
    }

    if let Some(digits) = filter.last_4_digits_eartag {
        query.push_str(" AND ear_tag LIKE ?");
        params.push(Box::new(format!("%{}", digits)) as Box<dyn ToSql>);
    }

    if let Some(s) = filter.sex {
        query.push_str(" AND sex = ?");
        params.push(Box::new(s.to_string()) as Box<dyn ToSql>);
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
        query.push_str(" AND birth_date >= ? AND birth_date <= ?");
        params.push(Box::new(format!("{year:04}-01-01")));
        params.push(Box::new(format!("{year:04}-12-31")));
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
        let count_sql = "CASE WHEN cows.sex = 'Female'
            THEN (SELECT COUNT(*) FROM inseminations WHERE dam_id = cows.id)
            ELSE (SELECT COUNT(*) FROM inseminations WHERE sire_id = cows.id)
            END";

        if let Some(lt) = filter.inseminations_less_than {
            query.push_str(&format!(" AND ({}) < ?", count_sql));
            params.push(Box::new(lt));
        }
        if let Some(mt) = filter.inseminations_more_than {
            query.push_str(&format!(" AND ({}) > ?", count_sql));
            params.push(Box::new(mt));
        }
    }

    // Completed calendar months: the current month counts once the birth day is reached.
    let age_sql = "((CAST(strftime('%Y', ?) AS INTEGER) - CAST(strftime('%Y', birth_date) AS INTEGER)) * 12
        + CAST(strftime('%m', ?) AS INTEGER) - CAST(strftime('%m', birth_date) AS INTEGER)
        - (CAST(strftime('%d', ?) AS INTEGER) < CAST(strftime('%d', birth_date) AS INTEGER)))";
    for (limit, operator) in [(filter.maximum_age_months, "<"), (filter.minimum_age_months, ">=")] {
        if let Some(months) = limit {
            query.push_str(&format!(" AND birth_date <= ? AND {} {} ?", age_sql, operator));
            for _ in 0..4 { params.push(Box::new(ref_date)); }
            params.push(Box::new(months));
        }
    }

    query.push_str(" ORDER BY ear_tag");

    let mut stmt = conn.prepare(&query).map_err(|e| e.to_string())?;
    let param_refs: Vec<&dyn ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let cow_iter = stmt.query_map(&param_refs[..], cow_from_row).map_err(|e| e.to_string())?;

    let results: Result<Vec<Cow>, _> = cow_iter.collect();
    results.map_err(|e| e.to_string())
}
