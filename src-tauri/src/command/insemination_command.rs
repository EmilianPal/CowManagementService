use rusqlite::Connection;
use crate::model::insemination::Insemination;
use crate::database::query::insemination_query;
use crate::command::command::Command;
use std::any::Any;


#[derive(Debug)]
pub struct AddInseminationCommand {
    pub insemination: Insemination,
    pub return_value: i64,
}

impl AddInseminationCommand {
    pub fn new(insemination: Insemination) -> Self {
        Self {
            insemination,
            return_value: 0,
        }
    }
}

impl Command for AddInseminationCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = insemination_query::insert_insemination(conn, &self.insemination, self.insemination.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        insemination_query::delete_insemination(conn, self.return_value, self.insemination.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct DeleteInseminationCommand {
    pub insemination: Insemination,
    pub return_value: bool,
}

impl DeleteInseminationCommand {
    pub fn new(insemination: Insemination) -> Self {
        Self {
            insemination,
            return_value: false,
        }
    }
}

impl Command for DeleteInseminationCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = insemination_query::delete_insemination(conn, self.insemination.id.unwrap(), self.insemination.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        insemination_query::insert_insemination(conn, &self.insemination, self.insemination.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct UpdateInseminationCommand{
    pub old_insemination: Insemination,
    pub new_insemination: Insemination,
    pub return_value: bool
}

impl UpdateInseminationCommand {
    pub fn new(old_insemination: Insemination, new_insemination: Insemination) -> Self {
        Self {
            old_insemination,
            new_insemination,
            return_value: false
        }
    }   
}

impl Command for UpdateInseminationCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = insemination_query::update_insemination(conn, &self.new_insemination, self.old_insemination.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        insemination_query::update_insemination(conn, &self.old_insemination, self.new_insemination.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}
