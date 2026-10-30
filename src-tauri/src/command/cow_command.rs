use rusqlite::Connection;
use crate::model::cow::{Cow, Sex};
use crate::model::birth::Birth;
use crate::model::insemination::Insemination;
use crate::database::query::{cow_query, birth_query, insemination_query};
use crate::command::command::Command;
use std::any::Any;
use rusqlite::{params, OptionalExtension};



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
        self.return_value = cow_query::insert_cow(conn, &self.cow, self.cow.farm_id)
            .map_err(|e| e.to_string())?;
        self.cow.id = Some(self.return_value);
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        cow_query::delete_cow(conn, self.return_value, self.cow.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Adds a calf and links it to its mother's birth as one undoable transaction.
#[derive(Debug)]
pub struct AddCowWithBirthCommand {
    pub cow: Cow,
    pub mother_id: i64,
    pub return_value: i64,
    birth_id: Option<i64>,
    created_birth: bool,
}

impl AddCowWithBirthCommand {
    pub fn new(cow: Cow, mother_id: i64) -> Self {
        Self { cow, mother_id, return_value: 0, birth_id: None, created_birth: false }
    }
}

impl Command for AddCowWithBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let farm_id = self.cow.farm_id;
        let mother = cow_query::get_cow(&tx, self.mother_id, farm_id).map_err(|_| "Mama selectată nu există în această fermă.".to_string())?;
        if !matches!(mother.sex, Sex::Female) || self.cow.birth_date < mother.birth_date {
            return Err("Mama sau data nașterii vițelului nu este validă.".into());
        }
        if self.cow.birth_id.is_some() || self.cow.birth_date > chrono::Local::now().date_naive()
            || self.cow.entry_date < self.cow.birth_date
            || self.cow.exit_date.is_some_and(|date| date < self.cow.entry_date) {
            return Err("Datele vițelului nu sunt valide.".into());
        }
        let existing: Option<i64> = tx.query_row(
            "SELECT id FROM births WHERE mother_id = ?1 AND date = ?2 AND farm_id = ?3",
            params![self.mother_id, self.cow.birth_date, farm_id], |row| row.get(0),
        ).optional().map_err(|e| e.to_string())?;
        let (birth_id, created_birth) = if let Some(id) = existing {
            (id, false)
        } else {
            let id = birth_query::insert_birth(&tx, &Birth {
                id: self.birth_id, mother_id: self.mother_id,
                date: self.cow.birth_date, farm_id,
            }, farm_id).map_err(|e| e.to_string())?;
            (id, true)
        };
        let mut linked = self.cow.clone();
        linked.birth_id = Some(birth_id);
        let cow_id = cow_query::insert_cow(&tx, &linked, farm_id).map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        self.return_value = cow_id;
        self.cow.id = Some(cow_id);
        self.birth_id = Some(birth_id);
        self.created_birth = created_birth;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        cow_query::delete_cow(&tx, self.return_value, self.cow.farm_id).map_err(|e| e.to_string())?;
        if self.created_birth {
            birth_query::delete_birth(&tx, self.birth_id.unwrap(), self.cow.farm_id).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[derive(Debug)]
pub struct DeleteCowCommand {
    pub cow: Cow,
    pub return_value: bool,
    pub deleted_births: Vec<Birth>,
    pub deleted_inseminations: Vec<Insemination>,
    pub affected_calves: Vec<Cow>,
}

impl DeleteCowCommand {
    pub fn new(cow: Cow) -> Self {
        Self {
            cow,
            return_value: false,
            deleted_births: Vec::new(),
            deleted_inseminations: Vec::new(),
            affected_calves: Vec::new(),
        }
    }
}


impl Command for DeleteCowCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        if self.cow.sex == Sex::Male {
            self.deleted_inseminations = insemination_query::get_inseminations_by_sire(&tx, self.cow.id.unwrap(), self.cow.farm_id)
                .map_err(|e| e.to_string())?;
            insemination_query::remove_sire_from_inseminations(&tx, self.cow.id.unwrap(), self.cow.farm_id)
                .map_err(|e| e.to_string())?;
        } else {
            self.deleted_births = birth_query::get_births_by_mother(&tx, self.cow.id.unwrap(), self.cow.farm_id)
                .map_err(|e| e.to_string())?;
            self.affected_calves.clear();
            for birth in &self.deleted_births {
                self.affected_calves.extend(cow_query::get_cows_born_on_a_given_birth(
                    &tx, birth.id.unwrap(), self.cow.farm_id,
                ).map_err(|e| e.to_string())?);
            }
            birth_query::delete_births_by_mother(&tx, self.cow.id.unwrap(), self.cow.farm_id)
                .map_err(|e| e.to_string())?;

            self.deleted_inseminations = insemination_query::get_inseminations_by_dam(&tx, self.cow.id.unwrap(), self.cow.farm_id)
                .map_err(|e| e.to_string())?;
            insemination_query::delete_insemination_by_dam(&tx, self.cow.id.unwrap(), self.cow.farm_id)
                .map_err(|e| e.to_string())?;
        }
        self.return_value = cow_query::delete_cow(&tx, self.cow.id.unwrap(), self.cow.farm_id)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        cow_query::insert_cow(&tx, &self.cow, self.cow.farm_id)
            .map_err(|e| e.to_string())?;
        if self.cow.sex == Sex::Male {
            for ins in &self.deleted_inseminations {
                insemination_query::update_insemination(&tx, ins, self.cow.farm_id)
                    .map_err(|e| e.to_string())?;
            }
        } else {
            for b in &self.deleted_births {
                birth_query::insert_birth(&tx, b, self.cow.farm_id).map_err(|e| e.to_string())?;
            }
            for calf in &self.affected_calves {
                cow_query::update_cow(&tx, calf, self.cow.farm_id).map_err(|e| e.to_string())?;
            }
            for ins in &self.deleted_inseminations {
                insemination_query::insert_insemination(&tx, ins, self.cow.farm_id)
                    .map_err(|e| e.to_string())?;
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct UpdateCowCommand {
    pub old_cow: Cow,
    pub new_cow: Cow,
    deleted_births: Vec<Birth>,
    deleted_inseminations: Vec<Insemination>,
    affected_calves: Vec<Cow>,
    pub return_value: bool,
}

impl UpdateCowCommand {
    pub fn new(old_cow: Cow, new_cow: Cow) -> Self {
        Self {
            old_cow,
            new_cow,
            deleted_births: Vec::new(),
            deleted_inseminations: Vec::new(),
            affected_calves: Vec::new(),
            return_value: false,
        }
    }
}

impl Command for UpdateCowCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        if self.old_cow.birth_id.is_some() && self.old_cow.birth_date != self.new_cow.birth_date {
            self.new_cow.birth_id = None;
        }
        if self.old_cow.sex != self.new_cow.sex {
            if self.old_cow.sex == Sex::Male {
                self.deleted_inseminations = insemination_query::get_inseminations_by_sire(&tx, self.old_cow.id.unwrap(),self.old_cow.farm_id)
                    .map_err(|e| e.to_string())?;
                insemination_query::remove_sire_from_inseminations(&tx, self.old_cow.id.unwrap(), self.old_cow.farm_id)
                    .map_err(|e| e.to_string())?;
            } else {
                self.deleted_births = birth_query::get_births_by_mother(&tx, self.old_cow.id.unwrap(), self.old_cow.farm_id)
                    .map_err(|e| e.to_string())?;
                self.affected_calves.clear();
                for birth in &self.deleted_births {
                    self.affected_calves.extend(cow_query::get_cows_born_on_a_given_birth(
                        &tx, birth.id.unwrap(), self.old_cow.farm_id,
                    ).map_err(|e| e.to_string())?);
                }
                birth_query::delete_births_by_mother(&tx, self.old_cow.id.unwrap(), self.old_cow.farm_id)
                    .map_err(|e| e.to_string())?;

                self.deleted_inseminations = insemination_query::get_inseminations_by_dam(&tx, self.old_cow.id.unwrap(), self.old_cow.farm_id)
                    .map_err(|e| e.to_string())?;
                insemination_query::delete_insemination_by_dam(&tx, self.old_cow.id.unwrap(), self.old_cow.farm_id)
                    .map_err(|e| e.to_string())?;
            }
        }

        self.return_value = cow_query::update_cow(&tx, &self.new_cow, self.old_cow.farm_id)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        cow_query::update_cow(&tx, &self.old_cow, self.new_cow.farm_id).map_err(|e| e.to_string())?;
        if self.old_cow.sex != self.new_cow.sex {
            if self.old_cow.sex == Sex::Male {
                for ins in &self.deleted_inseminations {
                    insemination_query::update_insemination(&tx, ins, self.old_cow.farm_id)
                        .map_err(|e| e.to_string())?;
                }
            } else {
                for b in &self.deleted_births {
                    birth_query::insert_birth(&tx, b, self.old_cow.farm_id).map_err(|e| e.to_string())?;
                }
                for calf in &self.affected_calves {
                    cow_query::update_cow(&tx, calf, self.old_cow.farm_id)
                        .map_err(|e| e.to_string())?;
                }
                for ins in &self.deleted_inseminations {
                    insemination_query::insert_insemination(&tx, ins, self.old_cow.farm_id)
                        .map_err(|e| e.to_string())?;
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
