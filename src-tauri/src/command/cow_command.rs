use rusqlite::Connection;
use crate::model::cow::{Cow, Sex};
use crate::model::birth::Birth;
use crate::model::insemination::Insemination;
use crate::database::query::{cow_query, birth_query, insemination_query};
use crate::command::command::Command;


#[derive(Debug)]
pub struct AddCowCommand {
    pub cow: Cow,
    pub return_value: i64,
}

impl  AddCowCommand {
    pub fn new(cow: Cow) -> Self {
        Self {
            cow,
            return_value: 0,
        }
    }
}

impl Command for AddCowCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = cow_query::insert_cow(conn, &self.cow)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        cow_query::delete_cow(conn, self.return_value)
            .map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct DeleteCowCommand {
    pub cow: Cow,
    pub return_value: bool,
    pub deleted_births: Vec<Birth>,
    pub deleted_inseminations: Vec<Insemination>,
}

impl DeleteCowCommand {
    pub fn new(cow: Cow) -> Self {
        Self {
            cow,
            return_value: false,
            deleted_births: Vec::new(),
            deleted_inseminations: Vec::new(),
        }
    }
}


impl Command for DeleteCowCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        if self.cow.sex == Sex::Male {
            self.deleted_inseminations = insemination_query::get_inseminations_by_sire(&tx, self.cow.id.unwrap())
                .map_err(|e| e.to_string())?;
            insemination_query::remove_sire_from_inseminations(&tx, self.cow.id.unwrap())
                .map_err(|e| e.to_string())?;
        } else {
            self.deleted_births = birth_query::get_births_by_mother(&tx, self.cow.id.unwrap())
                .map_err(|e| e.to_string())?;
            birth_query::delete_births_by_mother(&tx, self.cow.id.unwrap())
                .map_err(|e| e.to_string())?;

            self.deleted_inseminations = insemination_query::get_inseminations_by_dam(&tx, self.cow.id.unwrap())
                .map_err(|e| e.to_string())?;
            insemination_query::delete_insemination_by_dam(&tx, self.cow.id.unwrap())
                .map_err(|e| e.to_string())?;
        }
        self.return_value = cow_query::delete_cow(&tx, self.cow.id.unwrap())
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        cow_query::insert_cow(&tx, &self.cow)
            .map_err(|e| e.to_string())?;
        if self.cow.sex == Sex::Male {
            for ins in &self.deleted_inseminations {
                insemination_query::insert_insemination(&tx, ins)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            for b in &self.deleted_births {
                birth_query::insert_birth(&tx, b).map_err(|e| e.to_string())?;
            }
            for ins in &self.deleted_inseminations {
                insemination_query::insert_insemination(&tx, ins)
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct UpdateCowCommand {
    pub old_cow: Cow,
    pub new_cow: Cow,
    deleted_births: Vec<Birth>,
    deleted_inseminations: Vec<Insemination>,
    pub return_value: bool,
}

impl UpdateCowCommand {
    pub fn new(old_cow: Cow, new_cow: Cow) -> Self {
        Self {
            old_cow,
            new_cow,
            deleted_births: Vec::new(),
            deleted_inseminations: Vec::new(),
            return_value: false,
        }
    }
}

impl Command for UpdateCowCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        if self.old_cow.sex != self.new_cow.sex {
            if self.old_cow.sex == Sex::Male {
                self.deleted_inseminations = insemination_query::get_inseminations_by_sire(&tx, self.old_cow.id.unwrap())
                    .map_err(|e| e.to_string())?;
                insemination_query::remove_sire_from_inseminations(&tx, self.old_cow.id.unwrap())
                    .map_err(|e| e.to_string())?;
            } else {
                self.deleted_births = birth_query::get_births_by_mother(&tx, self.old_cow.id.unwrap())
                    .map_err(|e| e.to_string())?;
                birth_query::delete_births_by_mother(&tx, self.old_cow.id.unwrap())
                    .map_err(|e| e.to_string())?;

                self.deleted_inseminations = insemination_query::get_inseminations_by_dam(&tx, self.old_cow.id.unwrap())
                    .map_err(|e| e.to_string())?;
                insemination_query::delete_insemination_by_dam(&tx, self.old_cow.id.unwrap())
                    .map_err(|e| e.to_string())?;
            }
        }

        self.return_value = cow_query::update_cow(&tx, &self.new_cow)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        cow_query::update_cow(&tx, &self.old_cow).map_err(|e| e.to_string())?;
        if self.old_cow.sex != self.new_cow.sex {
            if self.old_cow.sex == Sex::Male {
                for ins in &self.deleted_inseminations {
                    insemination_query::update_insemination(&tx, ins)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                for b in &self.deleted_births {
                    birth_query::insert_birth(&tx, b).map_err(|e| e.to_string())?;
                }
                for ins in &self.deleted_inseminations {
                    insemination_query::insert_insemination(&tx, ins)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }
}