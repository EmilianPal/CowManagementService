#![cfg(test)]

use cowmanagementservice_lib::command::birth_command::{AddBirthCommand, DeleteBirthCommand, UpdateBirthCommand};
use cowmanagementservice_lib::command::command_manager::CommandManager;
use cowmanagementservice_lib::command::cow_command::{AddCowCommand, DeleteCowCommand, UpdateCowCommand};
use cowmanagementservice_lib::command::insemination_command::{
    AddInseminationCommand, DeleteInseminationCommand, UpdateInseminationCommand,
};
use cowmanagementservice_lib::database;
use cowmanagementservice_lib::database::query::{birth_query, cow_query, insemination_query};
use cowmanagementservice_lib::model::birth::Birth;
use cowmanagementservice_lib::model::cow::{Breed, Category, Cow, Sex};
use cowmanagementservice_lib::model::insemination::Insemination;
use chrono::NaiveDate;
use rusqlite::Connection;

fn setup() -> Connection {
    let mut conn = Connection::open_in_memory().unwrap();
    database::database::create_tables(&mut conn).unwrap();
    conn
}

#[test]
fn test_add_cow_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let cow = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };

    // Execute AddCowCommand
    let add_command = Box::new(AddCowCommand::new(cow.clone()));
    assert!(command_manager.execute(add_command, &mut conn).is_ok());

    // Verify cow is added
    let fetched_cow = cow_query::get_cow_by_eartag(&conn, &cow.ear_tag).unwrap();
    assert_eq!(fetched_cow.ear_tag, cow.ear_tag);

    let added_cow_id = fetched_cow.id.unwrap();

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify cow is removed
    assert!(cow_query::get_cow(&conn, added_cow_id).is_err());

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify cow is added again
    let fetched_cow = cow_query::get_cow_by_eartag(&conn, &cow.ear_tag).unwrap();
    assert_eq!(fetched_cow.ear_tag, cow.ear_tag);
}

#[test]
fn test_delete_cow_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mut cow = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let cow_id = cow_query::insert_cow(&conn, &cow).unwrap();
    cow.id = Some(cow_id);

    // Execute DeleteCowCommand
    let delete_command = Box::new(DeleteCowCommand::new(cow.clone()));
    assert!(command_manager.execute(delete_command, &mut conn).is_ok());

    // Verify cow is deleted
    assert!(cow_query::get_cow(&conn, cow_id).is_err());

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify cow is restored
    let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
    assert_eq!(fetched_cow.ear_tag, cow.ear_tag);

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify cow is deleted again
    assert!(cow_query::get_cow(&conn, cow_id).is_err());
}

#[test]
fn test_update_cow_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mut old_cow = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let cow_id = cow_query::insert_cow(&conn, &old_cow).unwrap();
    old_cow.id = Some(cow_id);

    let mut new_cow = old_cow.clone();
    new_cow.ear_tag = "5678".to_string();

    // Execute UpdateCowCommand
    let update_command = Box::new(UpdateCowCommand::new(old_cow.clone(), new_cow.clone()));
    assert!(command_manager.execute(update_command, &mut conn).is_ok());

    // Verify cow is updated
    let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
    assert_eq!(fetched_cow.ear_tag, new_cow.ear_tag);

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify cow is restored to old state
    let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
    assert_eq!(fetched_cow.ear_tag, old_cow.ear_tag);

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify cow is updated again
    let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
    assert_eq!(fetched_cow.ear_tag, new_cow.ear_tag);
}

#[test]
fn test_add_birth_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mother = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();

    let birth = Birth {
        id: None,
        mother_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };

    // Execute AddBirthCommand
    let add_command = Box::new(AddBirthCommand::new(birth.clone()));
    assert!(command_manager.execute(add_command, &mut conn).is_ok());

    // Verify birth is added
    let fetched_birth =
        birth_query::get_birth_by_mother_and_date(&conn, birth.mother_id, &birth.date.to_string())
            .unwrap();
    assert_eq!(fetched_birth.mother_id, birth.mother_id);

    let added_birth_id = fetched_birth.id.unwrap();

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify birth is removed
    assert!(birth_query::get_birth(&conn, added_birth_id).is_err());

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify birth is added again
    let fetched_birth =
        birth_query::get_birth_by_mother_and_date(&conn, birth.mother_id, &birth.date.to_string())
            .unwrap();
    assert_eq!(fetched_birth.mother_id, birth.mother_id);
}

#[test]
fn test_delete_birth_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mother = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();

    let mut birth = Birth {
        id: None,
        mother_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };
    let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();
    birth.id = Some(birth_id);

    // Execute DeleteBirthCommand
    let delete_command = Box::new(DeleteBirthCommand::new(birth.clone()));
    assert!(command_manager.execute(delete_command, &mut conn).is_ok());

    // Verify birth is deleted
    assert!(birth_query::get_birth(&conn, birth_id).is_err());

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify birth is restored
    let fetched_birth = birth_query::get_birth(&conn, birth_id).unwrap();
    assert_eq!(fetched_birth.mother_id, birth.mother_id);

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify birth is deleted again
    assert!(birth_query::get_birth(&conn, birth_id).is_err());
}

#[test]
fn test_update_birth_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mother1 = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother1_id = cow_query::insert_cow(&conn, &mother1).unwrap();

    let mother2 = Cow {
        id: None,
        ear_tag: "5678".to_string(),
        sex: Sex::Female,
        breed: Breed::BaltataRomaneasca,
        category: Category::Lapte,
        birth_date: NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother2_id = cow_query::insert_cow(&conn, &mother2).unwrap();

    let mut old_birth = Birth {
        id: None,
        mother_id: mother1_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };
    let birth_id = birth_query::insert_birth(&conn, &old_birth).unwrap();
    old_birth.id = Some(birth_id);

    let mut new_birth = old_birth.clone();
    new_birth.mother_id = mother2_id;
    new_birth.date = NaiveDate::from_ymd_opt(2023, 6, 6).unwrap();

    // Execute UpdateBirthCommand
    let update_command = Box::new(UpdateBirthCommand::new(old_birth.clone(), new_birth.clone()));
    assert!(command_manager.execute(update_command, &mut conn).is_ok());

    // Verify birth is updated
    let fetched_birth = birth_query::get_birth(&conn, birth_id).unwrap();
    assert_eq!(fetched_birth.mother_id, new_birth.mother_id);
    assert_eq!(fetched_birth.date, new_birth.date);

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify birth is restored to old state
    let fetched_birth = birth_query::get_birth(&conn, birth_id).unwrap();
    assert_eq!(fetched_birth.mother_id, old_birth.mother_id);
    assert_eq!(fetched_birth.date, old_birth.date);

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify birth is updated again
    let fetched_birth = birth_query::get_birth(&conn, birth_id).unwrap();
    assert_eq!(fetched_birth.mother_id, new_birth.mother_id);
    assert_eq!(fetched_birth.date, new_birth.date);
}

#[test]
fn test_add_insemination_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let dam = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();

    let sire = Cow {
        id: None,
        ear_tag: "5678".to_string(),
        sex: Sex::Male,
        breed: Breed::AmbardeenAngus,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

    let insemination = Insemination {
        id: None,
        dam_id,
        sire_id: Some(sire_id),
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };

    // Execute AddInseminationCommand
    let add_command = Box::new(AddInseminationCommand::new(insemination.clone()));
    assert!(command_manager.execute(add_command, &mut conn).is_ok());

    // Verify insemination is added
    let fetched_inse = insemination_query::get_insemination_by_dam_and_date(
        &conn,
        insemination.dam_id,
        &insemination.date.to_string(),
    )
    .unwrap();
    assert_eq!(fetched_inse.dam_id, insemination.dam_id);

    let added_inse_id = fetched_inse.id.unwrap();

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify insemination is removed
    assert!(insemination_query::get_insemination(&conn, added_inse_id).is_err());

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify insemination is added again
    let fetched_inse = insemination_query::get_insemination_by_dam_and_date(
        &conn,
        insemination.dam_id,
        &insemination.date.to_string(),
    )
    .unwrap();
    assert_eq!(fetched_inse.dam_id, insemination.dam_id);
}

#[test]
fn test_delete_insemination_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let dam = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();

    let mut insemination = Insemination {
        id: None,
        dam_id,
        sire_id: None,
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };
    let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();
    insemination.id = Some(insemination_id);

    // Execute DeleteInseminationCommand
    let delete_command = Box::new(DeleteInseminationCommand::new(insemination.clone()));
    assert!(command_manager.execute(delete_command, &mut conn).is_ok());

    // Verify insemination is deleted
    assert!(insemination_query::get_insemination(&conn, insemination_id).is_err());

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify insemination is restored
    let fetched_inse = insemination_query::get_insemination(&conn, insemination_id).unwrap();
    assert_eq!(fetched_inse.dam_id, insemination.dam_id);

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify insemination is deleted again
    assert!(insemination_query::get_insemination(&conn, insemination_id).is_err());
}

#[test]
fn test_update_insemination_command() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let dam1 = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam1_id = cow_query::insert_cow(&conn, &dam1).unwrap();

    let dam2 = Cow {
        id: None,
        ear_tag: "5678".to_string(),
        sex: Sex::Female,
        breed: Breed::BaltataRomaneasca,
        category: Category::Lapte,
        birth_date: NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam2_id = cow_query::insert_cow(&conn, &dam2).unwrap();

    let mut old_insemination = Insemination {
        id: None,
        dam_id: dam1_id,
        sire_id: None,
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };
    let insemination_id = insemination_query::insert_insemination(&conn, &old_insemination).unwrap();
    old_insemination.id = Some(insemination_id);

    let mut new_insemination = old_insemination.clone();
    new_insemination.dam_id = dam2_id;
    new_insemination.date = NaiveDate::from_ymd_opt(2023, 8, 8).unwrap();

    // Execute UpdateInseminationCommand
    let update_command = Box::new(UpdateInseminationCommand::new(old_insemination.clone(), new_insemination.clone()));
    assert!(command_manager.execute(update_command, &mut conn).is_ok());

    // Verify insemination is updated
    let fetched_inse = insemination_query::get_insemination(&conn, insemination_id).unwrap();
    assert_eq!(fetched_inse.dam_id, new_insemination.dam_id);
    assert_eq!(fetched_inse.date, new_insemination.date);

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify insemination is restored to old state
    let fetched_inse = insemination_query::get_insemination(&conn, insemination_id).unwrap();
    assert_eq!(fetched_inse.dam_id, old_insemination.dam_id);
    assert_eq!(fetched_inse.date, old_insemination.date);

    // Redo the command
    assert!(command_manager.redo(&mut conn).is_ok());

    // Verify insemination is updated again
    let fetched_inse = insemination_query::get_insemination(&conn, insemination_id).unwrap();
    assert_eq!(fetched_inse.dam_id, new_insemination.dam_id);
    assert_eq!(fetched_inse.date, new_insemination.date);
}

#[test]
fn test_delete_female_cow_cascades() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mut mother = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
    mother.id = Some(mother_id);

    let birth = Birth {
        id: None,
        mother_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };
    let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();

    let insemination = Insemination {
        id: None,
        dam_id: mother_id,
        sire_id: None,
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };
    let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();

    // Execute DeleteCowCommand
    let delete_command = Box::new(DeleteCowCommand::new(mother.clone()));
    assert!(command_manager.execute(delete_command, &mut conn).is_ok());

    // Verify cow, birth, and insemination are deleted
    assert!(cow_query::get_cow(&conn, mother_id).is_err());
    assert!(birth_query::get_birth(&conn, birth_id).is_err());
    assert!(insemination_query::get_insemination(&conn, insemination_id).is_err());

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify cow, birth, and insemination are restored
    assert!(cow_query::get_cow(&conn, mother_id).is_ok());
    assert!(birth_query::get_birth(&conn, birth_id).is_ok());
    assert!(insemination_query::get_insemination(&conn, insemination_id).is_ok());
}

#[test]
fn test_delete_male_cow_cascades() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let dam = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();

    let mut sire = Cow {
        id: None,
        ear_tag: "5678".to_string(),
        sex: Sex::Male,
        breed: Breed::AmbardeenAngus,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();
    sire.id = Some(sire_id);

    let insemination = Insemination {
        id: None,
        dam_id,
        sire_id: Some(sire_id),
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };
    let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();

    // Execute DeleteCowCommand
    let delete_command = Box::new(DeleteCowCommand::new(sire.clone()));
    assert!(command_manager.execute(delete_command, &mut conn).is_ok());

    // Verify sire is deleted and insemination sire_id is null
    assert!(cow_query::get_cow(&conn, sire_id).is_err());
    let fetched_insemination = insemination_query::get_insemination(&conn, insemination_id).unwrap();
    assert!(fetched_insemination.sire_id.is_none());

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify sire is restored and insemination sire_id is restored
    assert!(cow_query::get_cow(&conn, sire_id).is_ok());
    let fetched_insemination = insemination_query::get_insemination(&conn, insemination_id).unwrap();
    assert_eq!(fetched_insemination.sire_id, Some(sire_id));
}

#[test]
fn test_update_cow_sex_cascades() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mut old_cow = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let cow_id = cow_query::insert_cow(&conn, &old_cow).unwrap();
    old_cow.id = Some(cow_id);

    let birth = Birth {
        id: None,
        mother_id: cow_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };
    let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();

    let insemination = Insemination {
        id: None,
        dam_id: cow_id,
        sire_id: None,
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };
    let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();

    let mut new_cow = old_cow.clone();
    new_cow.sex = Sex::Male;

    // Execute UpdateCowCommand
    let update_command = Box::new(UpdateCowCommand::new(old_cow.clone(), new_cow.clone()));
    assert!(command_manager.execute(update_command, &mut conn).is_ok());

    // Verify cow sex is updated, and birth/insemination are deleted
    let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
    assert_eq!(fetched_cow.sex, Sex::Male);
    assert!(birth_query::get_birth(&conn, birth_id).is_err());
    assert!(insemination_query::get_insemination(&conn, insemination_id).is_err());

    // Undo the command
    assert!(command_manager.undo(&mut conn).is_ok());

    // Verify cow sex is restored, and birth/insemination are restored
    let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
    assert_eq!(fetched_cow.sex, Sex::Female);
    assert!(birth_query::get_birth(&conn, birth_id).is_ok());
    assert!(insemination_query::get_insemination(&conn, insemination_id).is_ok());
}

#[test]
fn test_birth_count_updates() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mother = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();

    let fetched_mother = cow_query::get_cow(&conn, mother_id).unwrap();
    assert_eq!(fetched_mother.birth_count, 0);

    let birth = Birth {
        id: None,
        mother_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };

    // Execute AddBirthCommand
    let add_command = Box::new(AddBirthCommand::new(birth.clone()));
    command_manager.execute(add_command, &mut conn).unwrap();

    // Verify birth_count is updated
    let fetched_mother = cow_query::get_cow(&conn, mother_id).unwrap();
    assert_eq!(fetched_mother.birth_count, 1);

    // Undo the command
    command_manager.undo(&mut conn).unwrap();

    // Verify birth_count is restored
    let fetched_mother = cow_query::get_cow(&conn, mother_id).unwrap();
    assert_eq!(fetched_mother.birth_count, 0);

    // Redo the command
    command_manager.redo(&mut conn).unwrap();

    // Verify birth_count is updated again
    let fetched_mother = cow_query::get_cow(&conn, mother_id).unwrap();
    assert_eq!(fetched_mother.birth_count, 1);

    // Get the added birth to delete it
    let fetched_birth =
        birth_query::get_birth_by_mother_and_date(&conn, birth.mother_id, &birth.date.to_string())
            .unwrap();
    
    // Execute DeleteBirthCommand
    let delete_command = Box::new(DeleteBirthCommand::new(fetched_birth.clone()));
    command_manager.execute(delete_command, &mut conn).unwrap();

    // Verify birth_count is updated
    let fetched_mother = cow_query::get_cow(&conn, mother_id).unwrap();
    assert_eq!(fetched_mother.birth_count, 0);

    // Undo the command
    command_manager.undo(&mut conn).unwrap();

    // Verify birth_count is restored
    let fetched_mother = cow_query::get_cow(&conn, mother_id).unwrap();
    assert_eq!(fetched_mother.birth_count, 1);
}

#[test]
fn test_insemination_count_updates() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let dam = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();

    let fetched_dam = cow_query::get_cow(&conn, dam_id).unwrap();
    assert_eq!(fetched_dam.insemination_count, 0);

    let insemination = Insemination {
        id: None,
        dam_id,
        sire_id: None,
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };

    // Execute AddInseminationCommand
    let add_command = Box::new(AddInseminationCommand::new(insemination.clone()));
    command_manager.execute(add_command, &mut conn).unwrap();

    // Verify insemination_count is updated
    let fetched_dam = cow_query::get_cow(&conn, dam_id).unwrap();
    assert_eq!(fetched_dam.insemination_count, 1);

    // Undo the command
    command_manager.undo(&mut conn).unwrap();

    // Verify insemination_count is restored
    let fetched_dam = cow_query::get_cow(&conn, dam_id).unwrap();
    assert_eq!(fetched_dam.insemination_count, 0);

    // Redo the command
    command_manager.redo(&mut conn).unwrap();

    // Verify insemination_count is updated again
    let fetched_dam = cow_query::get_cow(&conn, dam_id).unwrap();
    assert_eq!(fetched_dam.insemination_count, 1);

    // Get the added insemination to delete it
    let fetched_insemination = insemination_query::get_insemination_by_dam_and_date(
        &conn,
        insemination.dam_id,
        &insemination.date.to_string(),
    )
    .unwrap();

    // Execute DeleteInseminationCommand
    let delete_command = Box::new(DeleteInseminationCommand::new(fetched_insemination.clone()));
    command_manager.execute(delete_command, &mut conn).unwrap();

    // Verify insemination_count is updated
    let fetched_dam = cow_query::get_cow(&conn, dam_id).unwrap();
    assert_eq!(fetched_dam.insemination_count, 0);

    // Undo the command
    command_manager.undo(&mut conn).unwrap();

    // Verify insemination_count is restored
    let fetched_dam = cow_query::get_cow(&conn, dam_id).unwrap();
    assert_eq!(fetched_dam.insemination_count, 1);
}

#[test]
fn test_update_birth_command_updates_counts() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let mother1 = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother1_id = cow_query::insert_cow(&conn, &mother1).unwrap();

    let mother2 = Cow {
        id: None,
        ear_tag: "5678".to_string(),
        sex: Sex::Female,
        breed: Breed::BaltataRomaneasca,
        category: Category::Lapte,
        birth_date: NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let mother2_id = cow_query::insert_cow(&conn, &mother2).unwrap();

    let mut old_birth = Birth {
        id: None,
        mother_id: mother1_id,
        date: NaiveDate::from_ymd_opt(2023, 5, 5).unwrap(),
    };
    let birth_id = birth_query::insert_birth(&conn, &old_birth).unwrap();
    old_birth.id = Some(birth_id);
    
    let fetched_mother1 = cow_query::get_cow(&conn, mother1_id).unwrap();
    assert_eq!(fetched_mother1.birth_count, 1);
    let fetched_mother2 = cow_query::get_cow(&conn, mother2_id).unwrap();
    assert_eq!(fetched_mother2.birth_count, 0);

    let mut new_birth = old_birth.clone();
    new_birth.mother_id = mother2_id;

    // Execute UpdateBirthCommand
    let update_command = Box::new(UpdateBirthCommand::new(old_birth.clone(), new_birth.clone()));
    command_manager.execute(update_command, &mut conn).unwrap();

    // Verify counts are updated
    let fetched_mother1 = cow_query::get_cow(&conn, mother1_id).unwrap();
    assert_eq!(fetched_mother1.birth_count, 0);
    let fetched_mother2 = cow_query::get_cow(&conn, mother2_id).unwrap();
    assert_eq!(fetched_mother2.birth_count, 1);
    
    // Undo the command
    command_manager.undo(&mut conn).unwrap();

    // Verify counts are restored
    let fetched_mother1 = cow_query::get_cow(&conn, mother1_id).unwrap();
    assert_eq!(fetched_mother1.birth_count, 1);
    let fetched_mother2 = cow_query::get_cow(&conn, mother2_id).unwrap();
    assert_eq!(fetched_mother2.birth_count, 0);
}

#[test]
fn test_update_insemination_command_updates_counts() {
    let mut conn = setup();
    let mut command_manager = CommandManager::new();

    let dam1 = Cow {
        id: None,
        ear_tag: "1234".to_string(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam1_id = cow_query::insert_cow(&conn, &dam1).unwrap();

    let dam2 = Cow {
        id: None,
        ear_tag: "5678".to_string(),
        sex: Sex::Female,
        breed: Breed::BaltataRomaneasca,
        category: Category::Lapte,
        birth_date: NaiveDate::from_ymd_opt(2019, 1, 1).unwrap(),
        entry_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    };
    let dam2_id = cow_query::insert_cow(&conn, &dam2).unwrap();

    let mut old_insemination = Insemination {
        id: None,
        dam_id: dam1_id,
        sire_id: None,
        date: NaiveDate::from_ymd_opt(2023, 7, 7).unwrap(),
    };
    let insemination_id = insemination_query::insert_insemination(&conn, &old_insemination).unwrap();
    old_insemination.id = Some(insemination_id);

    let fetched_dam1 = cow_query::get_cow(&conn, dam1_id).unwrap();
    assert_eq!(fetched_dam1.insemination_count, 1);
    let fetched_dam2 = cow_query::get_cow(&conn, dam2_id).unwrap();
    assert_eq!(fetched_dam2.insemination_count, 0);

    let mut new_insemination = old_insemination.clone();
    new_insemination.dam_id = dam2_id;

    // Execute UpdateInseminationCommand
    let update_command = Box::new(UpdateInseminationCommand::new(old_insemination.clone(), new_insemination.clone()));
    command_manager.execute(update_command, &mut conn).unwrap();

    // Verify counts are updated
    let fetched_dam1 = cow_query::get_cow(&conn, dam1_id).unwrap();
    assert_eq!(fetched_dam1.insemination_count, 0);
    let fetched_dam2 = cow_query::get_cow(&conn, dam2_id).unwrap();
    assert_eq!(fetched_dam2.insemination_count, 1);
    
    // Undo the command
    command_manager.undo(&mut conn).unwrap();

    // Verify counts are restored
    let fetched_dam1 = cow_query::get_cow(&conn, dam1_id).unwrap();
    assert_eq!(fetched_dam1.insemination_count, 1);
    let fetched_dam2 = cow_query::get_cow(&conn, dam2_id).unwrap();
    assert_eq!(fetched_dam2.insemination_count, 0);
}

