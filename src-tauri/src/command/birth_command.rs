use rusqlite::Connection;
use crate::model::birth::{self, Birth};
use crate::database::query::{cow_query, birth_query};
use super::Command;

#[derivable(Debug)]
pub struct AddBirthCommand {
    pub birth: Birth,
    pub return_value: i64,
}

impl AddBirthCommand {
    pub fn new(birth: Birth) -> Self {
        Self {
            birth,
            return_value: 0,
        }
    }
}


impl Command for AddBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = birth_query::insert_birth(conn, &self.birth)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        birth_query::delete_birth(conn, self.return_value)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteBirthCommand {
    pub birth: Birth,
    pub return_value: bool,
    pub affected_cows: Vec<Cow>
}

impl DeleteBirthCommand {
    fn new(birth: Birth) -> Self {
        Self {
            birth,
            return_value: false,
            affected_cows: Vec::new(),
        }
    }
}


impl Command for DeleteBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        self.affected_cows = cow_query::get_cows_born_on_a_given_birth(&tx, self.birth.id.unwrap())
            .map_err(|e| e.to_string())?;
        cow_query::remove_birth_from_cows(&tx, self.birth.id.unwrap())
            .map_err(|e| e.to_string())?;
        self.return_value = birth_query::delete_birth(&tx, birth.id.unwrap())
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        birth_query::insert_birth(&tx, &self.birth)
            .map_err(|e| e.to_string())?;
        for cow in &self.affected_cows {
            cow_query::update_cow(&tx, cow)
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateBirthCommand {
    pub old_birth: Birth,
    pub new_birth: Birth,
    pub return_value: bool,
}

impl UpdateBirthCommand {
    pub fn new(old_birth: Birth, new_birth: Birth) -> Self {
        Self {
            old_birth,
            new_birth,
            return_value: false,
        }
    }
}


impl Command for UpdateBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = birth_query::update_birth(conn, &self.new_birth)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        birth_query::update_birth(conn, &self.old_birth)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}