use rusqlite::Connection;
use crate::command::command_manager::CommandManager;
use crate::database::query::{cow_query, birth_query, insemination_query};
use crate::command::{cow_command, birth_command, insemination_command};
use crate::model::{cow::Cow, birth::Birth, insemination::Insemination};
use crate::utils::{cow_filter::CowFilter, xlsx_export};
use chrono::NaiveDate;

pub struct Service {
    pub conn: Connection,
    pub command_manager: CommandManager,
}

impl Service {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            command_manager: CommandManager::new(),
        }
    }

    // Command-related methods
    pub fn add_cow(&mut self, cow: Cow) -> Result<i64, String> {
        let command = Box::new(cow_command::AddCowCommand::new(cow));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let add_cow_command = result
            .as_any_mut()
            .downcast_mut::<cow_command::AddCowCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(add_cow_command.return_value)
    }

    pub fn update_cow(&mut self, cow: Cow) -> Result<bool, String> {
        let old_cow = self.get_cow(cow.id.unwrap())?;
        let command = Box::new(cow_command::UpdateCowCommand::new(old_cow, cow));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let update_cow_command = result
            .as_any_mut()
            .downcast_mut::<cow_command::UpdateCowCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(update_cow_command.return_value)
    }

    pub fn delete_cow(&mut self, cow_id: i64) -> Result<bool, String> {
        let cow = self.get_cow(cow_id)?;
        let command = Box::new(cow_command::DeleteCowCommand::new(cow));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let delete_cow_command = result
            .as_any_mut()
            .downcast_mut::<cow_command::DeleteCowCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(delete_cow_command.return_value)
    }

    pub fn add_birth(&mut self, birth: Birth) -> Result<i64, String> {
        let command = Box::new(birth_command::AddBirthCommand::new(birth));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let add_birth_command = result
            .as_any_mut()
            .downcast_mut::<birth_command::AddBirthCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(add_birth_command.return_value)
    }

    pub fn update_birth(&mut self, birth: Birth) -> Result<bool, String> {
        let old_birth = self.get_birth(birth.id.unwrap())?;
        let command = Box::new(birth_command::UpdateBirthCommand::new(old_birth, birth));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let update_birth_command = result
            .as_any_mut()
            .downcast_mut::<birth_command::UpdateBirthCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(update_birth_command.return_value)
    }

    pub fn delete_birth(&mut self, birth_id: i64) -> Result<bool, String> {
        let birth = self.get_birth(birth_id)?;
        let command = Box::new(birth_command::DeleteBirthCommand::new(birth));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let delete_birth_command = result
            .as_any_mut()
            .downcast_mut::<birth_command::DeleteBirthCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(delete_birth_command.return_value)
    }

    pub fn add_insemination(&mut self, insemination: Insemination) -> Result<i64, String> {
        let command = Box::new(insemination_command::AddInseminationCommand::new(insemination));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let add_insemination_command = result
            .as_any_mut()
            .downcast_mut::<insemination_command::AddInseminationCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(add_insemination_command.return_value)
    }

    pub fn update_insemination(&mut self, insemination: Insemination) -> Result<bool, String> {
        let old_insemination = self.get_insemination(insemination.id.unwrap())?;
        let command = Box::new(insemination_command::UpdateInseminationCommand::new(old_insemination, insemination));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let update_insemination_command = result
            .as_any_mut()
            .downcast_mut::<insemination_command::UpdateInseminationCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(update_insemination_command.return_value)
    }

    pub fn delete_insemination(&mut self, insemination_id: i64) -> Result<bool, String> {
        let insemination = self.get_insemination(insemination_id)?;
        let command = Box::new(insemination_command::DeleteInseminationCommand::new(insemination));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let delete_insemination_command = result
            .as_any_mut()
            .downcast_mut::<insemination_command::DeleteInseminationCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(delete_insemination_command.return_value)
    }

    pub fn assign_calf_to_birth(&mut self, cow_id: i64, birth_id: i64) -> Result<bool, String> {
        let old_cow = self.get_cow(cow_id)?;
        let new_cow = Cow {
            id: Some(cow_id),
            ear_tag: old_cow.ear_tag.clone(),
            sex: old_cow.sex.clone(), 
            breed: old_cow.breed.clone(),
            category: old_cow.category.clone(),
            birth_date: old_cow.birth_date,
            entry_date: old_cow.entry_date,
            exit_date: old_cow.exit_date,
            birth_id: Some(birth_id),
            birth_count: old_cow.birth_count,
            insemination_count: old_cow.insemination_count,
        };

        let command = Box::new(cow_command::UpdateCowCommand::new(old_cow, new_cow));
        let result = self.command_manager.execute(command, &mut self.conn)?;
        let update_cow_command = result
            .as_any_mut()
            .downcast_mut::<cow_command::UpdateCowCommand>()
            .ok_or("Failed to downcast command")?;
        Ok(update_cow_command.return_value)
    }

    pub fn undo(&mut self) -> Result<(), String> {
        self.command_manager.undo(&mut self.conn)
    }

    pub fn redo(&mut self) -> Result<(), String> {
        self.command_manager.redo(&mut self.conn)
    }

    // Query methods
    pub fn get_cows(&self) -> Result<Vec<Cow>, String> {
        cow_query::get_cows(&self.conn).map_err(|e| e.to_string())
    }

    pub fn get_cow(&self, id: i64) -> Result<Cow, String> {
        cow_query::get_cow(&self.conn, id).map_err(|e| e.to_string())
    }

    pub fn get_cow_by_eartag(&self, ear_tag: &str) -> Result<Cow, String> {
        cow_query::get_cow_by_eartag(&self.conn, ear_tag).map_err(|e| e.to_string())
    }
    
    pub fn get_unassigned_calves_on_date(&self, date: &NaiveDate) -> Result<Vec<Cow>, String> {
        cow_query::get_unassigned_calves_on_date(&self.conn, date).map_err(|e| e.to_string())
    }

    pub fn get_cows_in_the_plantation(&self, date: &NaiveDate) -> Result<Vec<Cow>, String> {
        cow_query::get_cows_in_the_plantation(&self.conn, date).map_err(|e| e.to_string())
    }

    pub fn get_cows_born_on_a_given_birth(&self, birth_id: i64) -> Result<Vec<Cow>, String> {
        cow_query::get_cows_born_on_a_given_birth(&self.conn, birth_id).map_err(|e| e.to_string())
    }

    pub fn get_cows_filtered(&self, filter: CowFilter) -> Result<Vec<Cow>, String> {
        cow_query::get_cows_filtered(&self.conn, filter)
    }

    pub fn get_births(&self) -> Result<Vec<Birth>, String> {
        birth_query::get_births(&self.conn).map_err(|e| e.to_string())
    }

    pub fn get_birth(&self, id: i64) -> Result<Birth, String> {
        birth_query::get_birth(&self.conn, id).map_err(|e| e.to_string())
    }

    pub fn get_births_by_mother(&self, mother_id: i64) -> Result<Vec<Birth>, String> {
        birth_query::get_births_by_mother(&self.conn, mother_id).map_err(|e| e.to_string())
    }

    pub fn get_birth_by_mother_and_date(&self, mother_id: i64, date: &str) -> Result<Birth, String> {
        birth_query::get_birth_by_mother_and_date(&self.conn, mother_id, date).map_err(|e| e.to_string())
    }


    pub fn get_inseminations(&self) -> Result<Vec<Insemination>, String> {
        insemination_query::get_inseminations(&self.conn).map_err(|e| e.to_string())
    }

    pub fn get_insemination(&self, id: i64) -> Result<Insemination, String> {
        insemination_query::get_insemination(&self.conn, id).map_err(|e| e.to_string())
    }

    pub fn get_inseminations_by_dam(&self, dam_id: i64) -> Result<Vec<Insemination>, String> {
        insemination_query::get_inseminations_by_dam(&self.conn, dam_id).map_err(|e| e.to_string())
    }

    pub fn get_inseminations_by_sire(&self, sire_id: i64) -> Result<Vec<Insemination>, String> {
        insemination_query::get_inseminations_by_sire(&self.conn, sire_id).map_err(|e| e.to_string())
    }

    pub fn get_insemination_by_dam_and_date(&self, dam_id: i64, date: &str) -> Result<Insemination, String> {
        insemination_query::get_insemination_by_dam_and_date(&self.conn, dam_id, date).map_err(|e| e.to_string())
    }

    pub fn get_insemination_by_sire_and_date(&self, sire_id: i64, date: &str) -> Result<Insemination, String> {
        insemination_query::get_insemination_by_sire_and_date(&self.conn, sire_id, date).map_err(|e| e.to_string())
    }


    // XLSX export
    pub fn export_to_xlsx(&self, path: &str, filter: CowFilter) -> Result<(), String> {
        let cows = self.get_cows_filtered(filter.clone())?;
        let messages = xlsx_export::filter_to_messages(&filter);
        xlsx_export::write_to_xlsx(path, messages, cows).map_err(|e| e.to_string())
    }
}