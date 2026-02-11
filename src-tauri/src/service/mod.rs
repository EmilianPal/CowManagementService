pub mod service;

use rusqlite::{Connection, Transaction};
use crate::model::cow::{Cow, Sex, Breed, Category};
use crate::model::birth::Birth;
use crate::model::insemination::Insemination;
use crate::database;

pub struct Service;

impl Service {
    pub fn insert_cow(conn: &Connection, cow: &Cow) -> Result<i64, rusqlite::Error> {
        database::query::cow_query::insert_cow(conn, cow)
    }

    pub fn delete_cow(conn: &Connection, id: i64) -> Result<bool, rusqlite::Error> {
        database::query::cow_query::delete_cow(conn, id)
    }

    pub fn update_cow(conn: &Connection, cow: &Cow) ->Result<bool, rusqlite::Error> {
        database::query::cow_query::update_cow(conn, cow)
    }

    
}