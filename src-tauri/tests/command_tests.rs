use chrono::NaiveDate;
use cowmanagementservice_lib::{
    command::{
        birth_command::{AddBirthCommand, DeleteBirthCommand, UpdateBirthCommand},
        command_manager::CommandManager,
        cow_command::{AddCowCommand, DeleteCowCommand, UpdateCowCommand},
        insemination_command::{
            AddInseminationCommand, DeleteInseminationCommand, UpdateInseminationCommand,
        },
    },
    database::{
        database::create_tables,
        query::{birth_query, cow_query, insemination_query},
    },
    model::{
        birth::Birth,
        cow::{Breed, Category, Cow, Sex},
        insemination::Insemination,
    },
};
use rusqlite::Connection;

fn date(value: &str) -> NaiveDate {
    value.parse().unwrap()
}
fn setup() -> (Connection, CommandManager) {
    let conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    conn.execute("INSERT INTO farms (id, name) VALUES (1, 'Test')", [])
        .unwrap();
    (conn, CommandManager::new())
}
fn cow(tag: &str) -> Cow {
    Cow {
        id: None,
        farm_id: 1,
        ear_tag: tag.into(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: date("2020-01-01"),
        entry_date: date("2020-02-01"),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    }
}

#[test]
fn cow_commands_undo_redo_and_new_edit_invalidates_redo() {
    let (mut conn, mut history) = setup();
    history
        .execute(Box::new(AddCowCommand::new(cow("RO-100"))), &mut conn)
        .unwrap();
    let original = cow_query::get_cow_by_eartag(&conn, "RO-100", 1).unwrap();
    let id = original.id.unwrap();
    let mut edited = original.clone();
    edited.ear_tag = "RO-101".into();
    history
        .execute(
            Box::new(UpdateCowCommand::new(original.clone(), edited.clone())),
            &mut conn,
        )
        .unwrap();
    assert_eq!(cow_query::get_cow(&conn, id, 1).unwrap().ear_tag, "RO-101");
    history.undo(&mut conn).unwrap();
    assert_eq!(cow_query::get_cow(&conn, id, 1).unwrap().ear_tag, "RO-100");
    history.redo(&mut conn).unwrap();
    assert_eq!(cow_query::get_cow(&conn, id, 1).unwrap().ear_tag, "RO-101");
    history
        .execute(
            Box::new(DeleteCowCommand::new(
                cow_query::get_cow(&conn, id, 1).unwrap(),
            )),
            &mut conn,
        )
        .unwrap();
    assert!(cow_query::get_cow(&conn, id, 1).is_err());
    history.undo(&mut conn).unwrap();
    assert_eq!(cow_query::get_cow(&conn, id, 1).unwrap().ear_tag, "RO-101");
    history
        .execute(Box::new(AddCowCommand::new(cow("RO-OTHER"))), &mut conn)
        .unwrap();
    assert!(history.redo_stack.is_empty());
}

#[test]
fn birth_commands_restore_event_identity_and_counts() {
    let (mut conn, mut history) = setup();
    let mother = cow_query::insert_cow(&conn, &cow("RO-MOTHER"), 1).unwrap();
    let birth = Birth {
        id: None,
        mother_id: mother,
        date: date("2025-01-01"),
        farm_id: 1,
    };
    history
        .execute(Box::new(AddBirthCommand::new(birth)), &mut conn)
        .unwrap();
    let original =
        birth_query::get_birth_by_mother_and_date(&conn, mother, "2025-01-01", 1).unwrap();
    let id = original.id.unwrap();
    let mut edited = original.clone();
    edited.date = date("2025-01-02");
    history
        .execute(
            Box::new(UpdateBirthCommand::new(original, edited.clone())),
            &mut conn,
        )
        .unwrap();
    assert_eq!(
        birth_query::get_birth(&conn, id, 1).unwrap().date,
        edited.date
    );
    history.undo(&mut conn).unwrap();
    assert_eq!(
        birth_query::get_birth(&conn, id, 1).unwrap().date,
        date("2025-01-01")
    );
    history.redo(&mut conn).unwrap();
    history
        .execute(Box::new(DeleteBirthCommand::new(edited)), &mut conn)
        .unwrap();
    assert_eq!(cow_query::get_cow(&conn, mother, 1).unwrap().birth_count, 0);
    history.undo(&mut conn).unwrap();
    assert_eq!(
        birth_query::get_birth(&conn, id, 1).unwrap().date,
        date("2025-01-02")
    );
    assert_eq!(cow_query::get_cow(&conn, mother, 1).unwrap().birth_count, 1);
}

#[test]
fn insemination_commands_restore_date_sire_and_counts() {
    let (mut conn, mut history) = setup();
    let dam = cow_query::insert_cow(&conn, &cow("RO-DAM"), 1).unwrap();
    let mut male = cow("RO-SIRE");
    male.sex = Sex::Male;
    let sire = cow_query::insert_cow(&conn, &male, 1).unwrap();
    history
        .execute(
            Box::new(AddInseminationCommand::new(Insemination {
                id: None,
                dam_id: dam,
                sire_id: Some(sire),
                date: date("2025-01-01"),
                farm_id: 1,
            })),
            &mut conn,
        )
        .unwrap();
    let original =
        insemination_query::get_insemination_by_dam_and_date(&conn, dam, "2025-01-01", 1).unwrap();
    let id = original.id.unwrap();
    let mut edited = original.clone();
    edited.sire_id = None;
    edited.date = date("2025-01-02");
    history
        .execute(
            Box::new(UpdateInseminationCommand::new(original, edited.clone())),
            &mut conn,
        )
        .unwrap();
    assert_eq!(
        cow_query::get_cow(&conn, sire, 1)
            .unwrap()
            .insemination_count,
        0
    );
    history.undo(&mut conn).unwrap();
    assert_eq!(
        insemination_query::get_insemination(&conn, id, 1)
            .unwrap()
            .sire_id,
        Some(sire)
    );
    history.redo(&mut conn).unwrap();
    history
        .execute(Box::new(DeleteInseminationCommand::new(edited)), &mut conn)
        .unwrap();
    assert_eq!(
        cow_query::get_cow(&conn, dam, 1)
            .unwrap()
            .insemination_count,
        0
    );
    history.undo(&mut conn).unwrap();
    assert_eq!(
        insemination_query::get_insemination(&conn, id, 1)
            .unwrap()
            .date,
        date("2025-01-02")
    );
}

#[test]
fn deleting_mother_restores_birth_twins_and_insemination_on_undo() {
    let (mut conn, mut history) = setup();
    let mother = cow_query::insert_cow(&conn, &cow("RO-MOTHER"), 1).unwrap();
    let birth = birth_query::insert_birth(
        &conn,
        &Birth {
            id: None,
            mother_id: mother,
            date: date("2025-01-01"),
            farm_id: 1,
        },
        1,
    )
    .unwrap();
    let ins = insemination_query::insert_insemination(
        &conn,
        &Insemination {
            id: None,
            dam_id: mother,
            sire_id: None,
            date: date("2024-03-01"),
            farm_id: 1,
        },
        1,
    )
    .unwrap();
    let mut twins = Vec::new();
    for tag in ["RO-TWIN-1", "RO-TWIN-2"] {
        let mut calf = cow(tag);
        calf.birth_date = date("2025-01-01");
        calf.entry_date = calf.birth_date;
        calf.birth_id = Some(birth);
        twins.push(cow_query::insert_cow(&conn, &calf, 1).unwrap());
    }
    history
        .execute(
            Box::new(DeleteCowCommand::new(
                cow_query::get_cow(&conn, mother, 1).unwrap(),
            )),
            &mut conn,
        )
        .unwrap();
    assert!(birth_query::get_birth(&conn, birth, 1).is_err());
    assert!(insemination_query::get_insemination(&conn, ins, 1).is_err());
    for id in &twins {
        assert_eq!(cow_query::get_cow(&conn, *id, 1).unwrap().birth_id, None);
    }
    history.undo(&mut conn).unwrap();
    assert!(birth_query::get_birth(&conn, birth, 1).is_ok());
    assert!(insemination_query::get_insemination(&conn, ins, 1).is_ok());
    for id in &twins {
        assert_eq!(
            cow_query::get_cow(&conn, *id, 1).unwrap().birth_id,
            Some(birth)
        );
    }
    history.redo(&mut conn).unwrap();
    assert!(cow_query::get_cow(&conn, mother, 1).is_err());
}
