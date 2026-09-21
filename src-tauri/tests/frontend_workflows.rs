use chrono::NaiveDate;
use cowmanagementservice_lib::{
    command::command_manager::CommandManager,
    database::database::create_tables,
    model::{birth::Birth, cow::{Breed, Category, Cow, Sex}, insemination::Insemination},
    service::service,
};
use rusqlite::Connection;

fn setup() -> (Connection, CommandManager) {
    let conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    conn.execute("INSERT INTO farms(id, name) VALUES (42, 'Ferma de test')", []).unwrap();
    (conn, CommandManager::new())
}
fn date(value: &str) -> NaiveDate { value.parse().unwrap() }
fn cow(tag: &str) -> Cow {
    Cow { id: None, farm_id: 42, ear_tag: tag.into(), sex: Sex::Female,
        breed: Breed::BaltataRomaneasca, category: Category::Carne,
        birth_date: date("2020-01-01"), entry_date: date("2020-02-01"),
        exit_date: None, birth_id: None, birth_count: 0, insemination_count: 0 }
}

#[test]
fn reproduction_updates_and_deletes_use_the_correct_farm_and_record_ids() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO1234")).unwrap();
    let birth_id = service::add_birth(&mut conn, &mut history, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }).unwrap();
    let mut birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    birth.date = date("2025-01-02");
    assert!(service::update_birth(&mut conn, &mut history, birth).unwrap());
    assert_eq!(service::get_birth(&mut conn, 42, birth_id).unwrap().date, date("2025-01-02"));
    assert!(service::delete_birth(&mut conn, &mut history, 42, birth_id).unwrap());
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, mother, 42).unwrap().birth_count, 1);
    let ins_id = service::add_insemination(&mut conn, &mut history, Insemination {
        id: None, dam_id: mother, sire_id: None, date: date("2025-06-01"), farm_id: 42,
    }).unwrap();
    let mut ins = service::get_insemination(&mut conn, 42, ins_id).unwrap();
    ins.date = date("2025-06-02");
    assert!(service::update_insemination(&mut conn, &mut history, 42, ins).unwrap());
    assert_eq!(service::get_insemination(&mut conn, 42, ins_id).unwrap().date, date("2025-06-02"));
    assert!(service::delete_insemination(&mut conn, &mut history, 42, ins_id).unwrap());
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, mother, 42).unwrap().insemination_count, 1);
}

#[test]
fn undo_redo_keeps_identifiers_and_calf_relationships() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO1000")).unwrap();
    let birth = service::add_birth(&mut conn, &mut history, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }).unwrap();
    let mut calf = cow("RO2000");
    calf.birth_date = date("2025-01-01");
    calf.entry_date = date("2025-01-01");
    calf.birth_id = Some(birth);
    let calf_id = service::add_cow(&mut conn, &mut history, 42, calf).unwrap();
    for _ in 0..3 { history.undo(&mut conn).unwrap(); }
    for _ in 0..3 { history.redo(&mut conn).unwrap(); }
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth));
    service::delete_cow(&mut conn, &mut history, 42, mother).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, None);
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth));
    assert!(service::get_cow(&mut conn, mother, 99).is_err());
}

#[test]
fn filtered_report_exports_a_real_excel_file() {
    use cowmanagementservice_lib::utils::cow_filter::CowFilter;
    let (mut conn, mut history) = setup();
    service::add_cow(&mut conn, &mut history, 42, cow("RO3000")).unwrap();
    let mut exited = cow("RO4000");
    exited.exit_date = Some(date("2024-01-01"));
    service::add_cow(&mut conn, &mut history, 42, exited).unwrap();
    let filter = CowFilter { date: Some(date("2025-01-01")), show_only_entered: true, ..Default::default() };
    let result = service::get_cows_filtered(&mut conn, 42, filter.clone()).unwrap();
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].ear_tag, "RO3000");
    let path = std::env::temp_dir().join(format!("ferma-report-test-{}.xlsx", std::process::id()));
    service::export_to_xlsx(&mut conn, 42, path.to_str().unwrap(), filter).unwrap();
    let bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(bytes.starts_with(b"PK"));
    assert!(bytes.len() > 1000);
}
