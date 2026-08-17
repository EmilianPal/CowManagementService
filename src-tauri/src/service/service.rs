use rusqlite::Connection;
use crate::command::command_manager::CommandManager;
use crate::database::query::{cow_query, birth_query, insemination_query};
use crate::command::{cow_command, birth_command, insemination_command};
use crate::model::{cow::Cow, birth::Birth, insemination::Insemination};
use crate::utils::{cow_filter::CowFilter, xlsx_export};
use chrono::NaiveDate;



// Command-related methods
pub fn add_cow(conn: &mut Connection, command_manager: &mut CommandManager, farm_id: i64, cow: Cow) -> Result<i64, String> {
    let cow = Cow {
        id: None,
        farm_id,
        ..cow
    };
    let command = Box::new(cow_command::AddCowCommand::new(cow));
    let result = command_manager.execute(command, conn)?;
    let add_cow_command = result
        .as_any_mut()
        .downcast_mut::<cow_command::AddCowCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(add_cow_command.return_value)
}

pub fn update_cow(conn: &mut Connection, command_manager: &mut CommandManager, cow: Cow) -> Result<bool, String> {
    let old_cow = get_cow(conn, cow.id.unwrap(), cow.farm_id)?;
    let command = Box::new(cow_command::UpdateCowCommand::new(old_cow, cow));
    let result = command_manager.execute(command, conn)?;
    let update_cow_command = result
        .as_any_mut()
        .downcast_mut::<cow_command::UpdateCowCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(update_cow_command.return_value)
}

pub fn delete_cow(conn: &mut Connection, command_manager: &mut CommandManager, farm_id: i64, cow_id: i64) -> Result<bool, String> {
    let cow = get_cow(conn, cow_id, farm_id)?;
    let command = Box::new(cow_command::DeleteCowCommand::new(cow));
    let result = command_manager.execute(command, conn)?;
    let delete_cow_command = result
        .as_any_mut()
        .downcast_mut::<cow_command::DeleteCowCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(delete_cow_command.return_value)
}

pub fn add_birth(conn: &mut Connection, command_manager: &mut CommandManager, birth: Birth) -> Result<i64, String> {
    let birth = Birth {
        id: None,
        farm_id: birth.farm_id,
        ..birth
    };
    let command = Box::new(birth_command::AddBirthCommand::new(birth));
    let result = command_manager.execute(command, conn)?;
    let add_birth_command = result
        .as_any_mut()
        .downcast_mut::<birth_command::AddBirthCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(add_birth_command.return_value)
}

pub fn update_birth(conn: &mut Connection, command_manager: &mut CommandManager, birth: Birth) -> Result<bool, String> {
    let old_birth = get_birth(conn, birth.id.unwrap(), birth.farm_id)?;
    let command = Box::new(birth_command::UpdateBirthCommand::new(old_birth, birth));
    let result = command_manager.execute(command, conn)?;
    let update_birth_command = result
        .as_any_mut()
        .downcast_mut::<birth_command::UpdateBirthCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(update_birth_command.return_value)
}

pub fn delete_birth(conn: &mut Connection, command_manager: &mut CommandManager, farm_id: i64, birth_id: i64) -> Result<bool, String> {
    let birth = get_birth(conn, birth_id, farm_id)?;
    let command = Box::new(birth_command::DeleteBirthCommand::new(birth));
    let result = command_manager.execute(command, conn)?;
    let delete_birth_command = result
        .as_any_mut()
        .downcast_mut::<birth_command::DeleteBirthCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(delete_birth_command.return_value)
}

pub fn add_insemination(conn: &mut Connection, command_manager: &mut CommandManager, insemination: Insemination) -> Result<i64, String> {
    let insemination = Insemination {
        id: None,
        farm_id: insemination.farm_id,
        ..insemination
    };
    let command = Box::new(insemination_command::AddInseminationCommand::new(insemination));
    let result = command_manager.execute(command, conn)?;
    let add_insemination_command = result
        .as_any_mut()
        .downcast_mut::<insemination_command::AddInseminationCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(add_insemination_command.return_value)
}

pub fn update_insemination(conn: &mut Connection, command_manager: &mut CommandManager, farm_id: i64, insemination: Insemination) -> Result<bool, String> {
    let old_insemination = get_insemination(conn, insemination.id.unwrap(), farm_id)?;
    let command = Box::new(insemination_command::UpdateInseminationCommand::new(old_insemination, insemination));
    let result = command_manager.execute(command, conn)?;
    let update_insemination_command = result
        .as_any_mut()
        .downcast_mut::<insemination_command::UpdateInseminationCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(update_insemination_command.return_value)
}

pub fn delete_insemination(conn: &mut Connection, command_manager: &mut CommandManager, farm_id: i64, insemination_id: i64) -> Result<bool, String> {
    let insemination = get_insemination(conn, insemination_id, farm_id)?;
    let command = Box::new(insemination_command::DeleteInseminationCommand::new(insemination));
    let result = command_manager.execute(command, conn)?;
    let delete_insemination_command = result
        .as_any_mut()
        .downcast_mut::<insemination_command::DeleteInseminationCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(delete_insemination_command.return_value)
}

pub fn assign_calf_to_birth(conn: &mut Connection, command_manager: &mut CommandManager, farm_id: i64, cow_id: i64, birth_id: i64) -> Result<bool, String> {
    let old_cow = get_cow(conn, cow_id, farm_id)?;
    let new_cow = Cow {
        id: Some(cow_id),
        farm_id: old_cow.farm_id,
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
    let result = command_manager.execute(command, conn)?;
    let update_cow_command = result
        .as_any_mut()
        .downcast_mut::<cow_command::UpdateCowCommand>()
        .ok_or("Failed to downcast command")?;
    Ok(update_cow_command.return_value)
}
// Query methods
pub fn get_cows(conn: &mut Connection, farm_id: i64) -> Result<Vec<Cow>, String> {
    cow_query::get_cows(conn, farm_id).map_err(|e| e.to_string())
}

pub fn get_cow(conn: &mut Connection, id: i64, farm_id: i64) -> Result<Cow, String> {
    cow_query::get_cow(conn, id, farm_id).map_err(|e| e.to_string())
}

pub fn get_cow_by_eartag(conn: &mut Connection, farm_id: i64, ear_tag: &str) -> Result<Cow, String> {
    cow_query::get_cow_by_eartag(conn, ear_tag, farm_id).map_err(|e| e.to_string())
}

pub fn get_unassigned_calves_on_date(conn: &mut Connection, farm_id: i64, date: &NaiveDate) -> Result<Vec<Cow>, String> {
    cow_query::get_unassigned_calves_on_date(conn, date, farm_id).map_err(|e| e.to_string())
}

pub fn get_cows_in_the_plantation(conn: &mut Connection, farm_id: i64, date: &NaiveDate) -> Result<Vec<Cow>, String> {
    cow_query::get_cows_in_the_plantation(conn, date, farm_id).map_err(|e| e.to_string())
}

pub fn get_cows_born_on_a_given_birth(conn: &mut Connection, farm_id: i64, birth_id: i64) -> Result<Vec<Cow>, String> {
    cow_query::get_cows_born_on_a_given_birth(conn, birth_id, farm_id).map_err(|e| e.to_string())
}

pub fn get_cows_filtered(conn: &mut Connection, farm_id: i64, filter: CowFilter) -> Result<Vec<Cow>, String> {
    cow_query::get_cows_filtered(conn, filter, farm_id)
}

pub fn get_births(conn: &mut Connection, farm_id: i64) -> Result<Vec<Birth>, String> {
    birth_query::get_births(conn, farm_id).map_err(|e| e.to_string())
}

pub fn get_birth(conn: &mut Connection, farm_id: i64, id: i64) -> Result<Birth, String> {
    birth_query::get_birth(conn, id, farm_id).map_err(|e| e.to_string())
}

pub fn get_births_by_mother(conn: &mut Connection, farm_id: i64, mother_id: i64) -> Result<Vec<Birth>, String> {
    birth_query::get_births_by_mother(conn, mother_id, farm_id).map_err(|e| e.to_string())
}

pub fn get_birth_by_mother_and_date(conn: &mut Connection, farm_id: i64, mother_id: i64, date: &str) -> Result<Birth, String> {
    birth_query::get_birth_by_mother_and_date(conn, mother_id, date, farm_id).map_err(|e| e.to_string())
}


pub fn get_inseminations(conn: &mut Connection, farm_id: i64) -> Result<Vec<Insemination>, String> {
    insemination_query::get_inseminations(conn, farm_id).map_err(|e| e.to_string())
}

pub fn get_insemination(conn: &mut Connection, farm_id: i64, id: i64) -> Result<Insemination, String> {
    insemination_query::get_insemination(conn, id, farm_id).map_err(|e| e.to_string())
}

pub fn get_inseminations_by_dam(conn: &mut Connection, farm_id: i64, dam_id: i64) -> Result<Vec<Insemination>, String> {
    insemination_query::get_inseminations_by_dam(conn, dam_id, farm_id).map_err(|e| e.to_string())
}

pub fn get_inseminations_by_sire(conn: &mut Connection, farm_id: i64, sire_id: i64) -> Result<Vec<Insemination>, String> {
    insemination_query::get_inseminations_by_sire(conn, sire_id, farm_id).map_err(|e| e.to_string())
}

pub fn get_insemination_by_dam_and_date(conn: &mut Connection, farm_id: i64, dam_id: i64, date: &str) -> Result<Insemination, String> {
    insemination_query::get_insemination_by_dam_and_date(conn, dam_id, date, farm_id).map_err(|e| e.to_string())
}

pub fn get_insemination_by_sire_and_date(conn: &mut Connection, farm_id: i64, sire_id: i64, date: &str) -> Result<Insemination, String> {
    insemination_query::get_insemination_by_sire_and_date(conn, sire_id, date, farm_id).map_err(|e| e.to_string())
}


// XLSX export
pub fn export_to_xlsx(conn: &mut Connection, farm_id: i64, path: &str, filter: CowFilter) -> Result<(), String> {
    let cows = get_cows_filtered(conn, farm_id, filter.clone())?;
    let messages = xlsx_export::filter_to_messages(&filter);
    xlsx_export::write_to_xlsx(path, messages, cows).map_err(|e| e.to_string())
}