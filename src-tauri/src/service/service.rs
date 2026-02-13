use rusqlite::{Connection, Transaction};
use crate::model::cow::{Cow, Sex, Breed, Category};
use crate::model::birth::Birth;
use crate::model::insemination::Insemination;
use crate::database::query::{birth_query, cow_query, insemination_query};

pub struct Service;

impl Service {
    pub fn insert_cow(conn: &Connection, cow: &Cow) -> Result<i64, rusqlite::Error> {
        cow_query::insert_cow(conn, cow)
    }

    pub fn delete_cow(conn: &Connection, id: i64) -> Result<bool, rusqlite::Error> {
        cow_query::delete_cow(conn, id)
    }

    pub fn update_cow(conn: &mut Connection, cow: &Cow) -> Result<bool, rusqlite::Error> {
        let tx = conn.transaction()?;
        let original_cow = cow_query::get_cow(&tx, cow.id.unwrap())?;
        if original_cow.sex != cow.sex {
            if original_cow.sex == Sex::Male {
                insemination_query::remove_sire_from_inseminations(&tx, original_cow.id.unwrap())?;
            } else if original_cow.sex == Sex::Female {
                birth_query::delete_births_by_mother(&tx, original_cow.id.unwrap())?;
                insemination_query::delete_insemination_by_dam(&tx, original_cow.id.unwrap())?;
            }
        }
        let success = cow_query::update_cow(&tx, cow)?;
        tx.commit()?;
        Ok(success)
    }

    pub fn get_cows(conn: &Connection) -> Result<Vec<Cow>, rusqlite::Error> {
        cow_query::get_cows(conn)
    }

    pub fn get_cow(conn: &Connection, id: i64) -> Result<Cow, rusqlite::Error> {
        cow_query::get_cow(conn, id)
    }

    
}