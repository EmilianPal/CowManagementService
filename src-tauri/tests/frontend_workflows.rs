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
fn newborn(tag: &str, born: &str) -> Cow {
    let mut calf = cow(tag);
    calf.birth_date = date(born);
    calf.entry_date = calf.birth_date;
    calf
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
fn changing_a_calfs_birth_date_detaches_only_that_calf_and_undo_restores_it() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let birth_id = service::add_birth(&mut conn, &mut history, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }).unwrap();
    let mut calf = cow("RO-FIRST");
    calf.birth_date = date("2025-01-01");
    calf.entry_date = date("2025-01-02");
    calf.birth_id = Some(birth_id);
    let first_id = service::add_cow(&mut conn, &mut history, 42, calf.clone()).unwrap();
    calf.ear_tag = "RO-SECOND".into();
    let second_id = service::add_cow(&mut conn, &mut history, 42, calf).unwrap();

    let mut edited = service::get_cow(&mut conn, first_id, 42).unwrap();
    edited.birth_date = date("2025-01-02");
    // The UI may still send the old birth ID; the update must detach it anyway.
    assert_eq!(edited.birth_id, Some(birth_id));
    assert!(service::update_cow(&mut conn, &mut history, edited).unwrap());
    let changed = service::get_cow(&mut conn, first_id, 42).unwrap();
    assert_eq!(changed.birth_date, date("2025-01-02"));
    assert_eq!(changed.birth_id, None);
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(birth_id));
    assert!(service::get_birth(&mut conn, 42, birth_id).is_ok());

    history.undo(&mut conn).unwrap();
    let restored = service::get_cow(&mut conn, first_id, 42).unwrap();
    assert_eq!(restored.birth_date, date("2025-01-01"));
    assert_eq!(restored.birth_id, Some(birth_id));
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, None);
}

#[test]
fn filtered_report_exports_a_real_excel_file() {
    use cowmanagementservice_lib::utils::cow_filter::CowFilter;
    use std::io::Read;
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
    assert!(bytes.starts_with(b"PK"));
    assert!(bytes.len() > 1000);
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes)).unwrap();
    let mut strings = String::new();
    archive.by_name("xl/sharedStrings.xml").unwrap().read_to_string(&mut strings).unwrap();
    assert!(strings.contains("RO3000"));
    assert!(!strings.contains("RO4000"));
    assert!(strings.contains("Crotalie"));
    let mut sheet = String::new();
    archive.by_name("xl/worksheets/sheet1.xml").unwrap().read_to_string(&mut sheet).unwrap();
    assert!(sheet.contains("<row"));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn birth_can_be_saved_without_a_calf_and_linked_later_atomically() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let birth_id = service::save_birth_with_calf(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, None, None).unwrap();
    let mut calf = cow("RO-CALF");
    calf.birth_date = date("2025-01-01");
    calf.entry_date = calf.birth_date;
    let calf_id = service::add_cow(&mut conn, &mut history, 42, calf).unwrap();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    service::save_birth_with_calf(&mut conn, &mut history, 42, birth, Some(calf_id), None).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth_id));
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, None);
    assert!(service::get_birth(&mut conn, 42, birth_id).is_ok());
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn birth_and_existing_calf_are_undone_and_redone_together() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut calf = cow("RO-CALF");
    calf.birth_date = date("2025-01-01"); calf.entry_date = calf.birth_date;
    let calf_id = service::add_cow(&mut conn, &mut history, 42, calf).unwrap();
    let id = service::save_birth_with_calf(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, Some(calf_id), None).unwrap();
    history.undo(&mut conn).unwrap();
    assert!(service::get_birth(&mut conn, 42, id).is_err());
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, None);
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(id));
}

#[test]
fn another_calf_can_be_added_to_an_existing_birth_without_replacing_the_first() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut first = cow("RO-FIRST");
    first.birth_date = date("2025-01-01"); first.entry_date = first.birth_date;
    let mut second = first.clone(); second.ear_tag = "RO-SECOND".into();
    let first_id = service::add_cow(&mut conn, &mut history, 42, first).unwrap();
    let second_id = service::add_cow(&mut conn, &mut history, 42, second).unwrap();
    let birth_id = service::save_birth_with_calf(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, Some(first_id), None).unwrap();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    service::save_birth_with_calf(&mut conn, &mut history, 42, birth, Some(second_id), None).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_birth(&mut conn, 42, birth_id).unwrap().mother_id, mother);
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, None);
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn calf_combo_saves_multiple_selections_and_restores_them_with_undo() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut calf = cow("RO-FIRST");
    calf.birth_date = date("2025-01-01"); calf.entry_date = calf.birth_date;
    let first_id = service::add_cow(&mut conn, &mut history, 42, calf.clone()).unwrap();
    calf.ear_tag = "RO-SECOND".into();
    let second_id = service::add_cow(&mut conn, &mut history, 42, calf.clone()).unwrap();
    calf.ear_tag = "RO-THIRD".into();
    let third_id = service::add_cow(&mut conn, &mut history, 42, calf).unwrap();
    let birth_id = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, vec![first_id, second_id]).unwrap();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    service::save_birth_with_calves(&mut conn, &mut history, 42, birth, vec![second_id, third_id]).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, None);
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_cow(&mut conn, third_id, 42).unwrap().birth_id, Some(birth_id));
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_cow(&mut conn, third_id, 42).unwrap().birth_id, None);
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, third_id, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn invalid_calf_does_not_save_birth_or_change_history() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let calf_id = service::add_cow(&mut conn, &mut history, 42, cow("RO-CALF")).unwrap();
    let depth = history.undo_stack.len();
    let birth = Birth { id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42 };
    assert!(service::save_birth_with_calf(&mut conn, &mut history, 42, birth.clone(), Some(calf_id), None).is_err());
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM births", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    assert_eq!(history.undo_stack.len(), depth);
    assert!(service::save_birth_with_calf(&mut conn, &mut history, 99, birth, None, None).is_err());
    assert_eq!(history.undo_stack.len(), depth);
}

#[test]
fn editing_birth_can_keep_replace_and_remove_calf_with_undo_redo() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut first = cow("RO-FIRST");
    first.birth_date = date("2025-01-01"); first.entry_date = first.birth_date;
    let mut second = first.clone(); second.ear_tag = "RO-SECOND".into();
    let first_id = service::add_cow(&mut conn, &mut history, 42, first).unwrap();
    let second_id = service::add_cow(&mut conn, &mut history, 42, second).unwrap();
    let id = service::save_birth_with_calf(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, Some(first_id), None).unwrap();
    let birth = service::get_birth(&mut conn, 42, id).unwrap();
    // Saving the same calf is valid.
    service::save_birth_with_calf(&mut conn, &mut history, 42, birth.clone(), Some(first_id), Some(first_id)).unwrap();
    service::save_birth_with_calf(&mut conn, &mut history, 42, birth.clone(), Some(second_id), Some(first_id)).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, None);
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(id));
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, Some(id));
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, None);
    history.redo(&mut conn).unwrap();
    service::save_birth_with_calf(&mut conn, &mut history, 42, birth, None, Some(second_id)).unwrap();
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, None);
    assert!(service::get_birth(&mut conn, 42, id).is_ok());
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(id));
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, None);
}

#[test]
fn rejected_replacement_preserves_original_calf_and_history() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut calf = cow("RO-FIRST");
    calf.birth_date = date("2025-01-01"); calf.entry_date = calf.birth_date;
    let first_id = service::add_cow(&mut conn, &mut history, 42, calf).unwrap();
    let wrong_id = service::add_cow(&mut conn, &mut history, 42, cow("RO-WRONG-DATE")).unwrap();
    let id = service::save_birth_with_calf(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, Some(first_id), None).unwrap();
    let birth = service::get_birth(&mut conn, 42, id).unwrap();
    let depth = history.undo_stack.len();
    assert!(service::save_birth_with_calf(&mut conn, &mut history, 42, birth.clone(), Some(wrong_id), Some(first_id)).is_err());
    assert!(service::save_birth_with_calf(&mut conn, &mut history, 42, birth.clone(), Some(mother), Some(first_id)).is_err());
    assert!(service::save_birth_with_calf(&mut conn, &mut history, 42, birth, None, Some(wrong_id)).is_err());
    assert_eq!(history.undo_stack.len(), depth);
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, Some(id));
    assert_eq!(service::get_cow(&mut conn, wrong_id, 42).unwrap().birth_id, None);
}

#[test]
fn replacement_can_correct_birth_date_and_undo_restores_it() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut first = cow("RO-FIRST");
    first.birth_date = date("2025-01-01"); first.entry_date = first.birth_date;
    let mut second = first.clone(); second.ear_tag = "RO-SECOND".into();
    second.birth_date = date("2025-01-02"); second.entry_date = second.birth_date;
    let first_id = service::add_cow(&mut conn, &mut history, 42, first).unwrap();
    let second_id = service::add_cow(&mut conn, &mut history, 42, second).unwrap();
    let id = service::save_birth_with_calf(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42,
    }, Some(first_id), None).unwrap();
    let mut birth = service::get_birth(&mut conn, 42, id).unwrap();
    birth.date = date("2025-01-02");
    service::save_birth_with_calf(&mut conn, &mut history, 42, birth, Some(second_id), Some(first_id)).unwrap();
    assert_eq!(service::get_birth(&mut conn, 42, id).unwrap().date, date("2025-01-02"));
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_birth(&mut conn, 42, id).unwrap().date, date("2025-01-01"));
    assert_eq!(service::get_cow(&mut conn, first_id, 42).unwrap().birth_id, Some(id));
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, None);
}

#[test]
fn adding_calf_with_mother_creates_and_undoes_the_birth_together() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut calf = cow("RO-CALF");
    calf.birth_date = date("2025-04-03");
    calf.entry_date = calf.birth_date;
    let calf_id = service::add_cow_with_birth(&mut conn, &mut history, 42, calf, mother).unwrap();
    let linked = service::get_cow(&mut conn, calf_id, 42).unwrap();
    let birth_id = linked.birth_id.unwrap();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    assert_eq!(birth.mother_id, mother);
    assert_eq!(birth.date, linked.birth_date);
    history.undo(&mut conn).unwrap();
    assert!(service::get_cow(&mut conn, calf_id, 42).is_err());
    assert!(service::get_birth(&mut conn, 42, birth_id).is_err());
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn second_calf_on_the_same_day_reuses_the_mothers_birth() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut first = cow("RO-FIRST");
    first.birth_date = date("2025-04-03"); first.entry_date = first.birth_date;
    let mut second = first.clone(); second.ear_tag = "RO-SECOND".into();
    let first_id = service::add_cow_with_birth(&mut conn, &mut history, 42, first, mother).unwrap();
    let second_id = service::add_cow_with_birth(&mut conn, &mut history, 42, second, mother).unwrap();
    let birth_id = service::get_cow(&mut conn, first_id, 42).unwrap().birth_id.unwrap();
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(birth_id));
    history.undo(&mut conn).unwrap();
    assert!(service::get_cow(&mut conn, second_id, 42).is_err());
    assert!(service::get_birth(&mut conn, 42, birth_id).is_ok());
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, second_id, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn adding_a_calf_uses_an_existing_birth_without_recreating_it() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let birth_id = service::add_birth(&mut conn, &mut history, Birth {
        id: None, mother_id: mother, date: date("2025-04-03"), farm_id: 42,
    }).unwrap();
    let mut calf = cow("RO-CALF");
    calf.birth_date = date("2025-04-03");
    calf.entry_date = calf.birth_date;
    let calf_id = service::add_cow_with_birth(&mut conn, &mut history, 42, calf, mother).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM births WHERE mother_id = ?1 AND date = ?2", rusqlite::params![mother, date("2025-04-03")], |row| row.get::<_, i64>(0)).unwrap(), 1);
    history.undo(&mut conn).unwrap();
    assert!(service::get_cow(&mut conn, calf_id, 42).is_err());
    assert!(service::get_birth(&mut conn, 42, birth_id).is_ok());
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, calf_id, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn invalid_mother_and_duplicate_calf_do_not_leave_partial_records() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let mut calf = cow("RO-CALF");
    calf.birth_date = date("2025-04-03"); calf.entry_date = calf.birth_date;
    let depth = history.undo_stack.len();
    assert!(service::add_cow_with_birth(&mut conn, &mut history, 42, calf.clone(), 999).is_err());
    let mut male = cow("RO-MALE"); male.sex = Sex::Male;
    let male_id = service::add_cow(&mut conn, &mut history, 42, male).unwrap();
    let depth_with_male = history.undo_stack.len();
    assert!(service::add_cow_with_birth(&mut conn, &mut history, 42, calf.clone(), male_id).is_err());
    assert_eq!(history.undo_stack.len(), depth_with_male);
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM births", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM cows WHERE ear_tag = 'RO-CALF'", [], |r| r.get::<_, i64>(0)).unwrap(), 0);
    assert_eq!(depth + 1, depth_with_male);
    service::add_cow_with_birth(&mut conn, &mut history, 42, calf.clone(), mother).unwrap();
    let before = history.undo_stack.len();
    assert!(service::add_cow_with_birth(&mut conn, &mut history, 42, calf.clone(), mother).is_err());
    calf.birth_date = date("2025-04-04");
    calf.entry_date = calf.birth_date;
    assert!(service::add_cow_with_birth(&mut conn, &mut history, 42, calf, mother).is_err());
    assert_eq!(history.undo_stack.len(), before);
    assert_eq!(conn.query_row("SELECT COUNT(*) FROM births", [], |r| r.get::<_, i64>(0)).unwrap(), 1);
}

#[test]
fn birth_can_gain_and_lose_all_calves_without_deleting_the_event() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let first = service::add_cow(&mut conn, &mut history, 42, newborn("RO-FIRST", "2025-05-01")).unwrap();
    let second = service::add_cow(&mut conn, &mut history, 42, newborn("RO-SECOND", "2025-05-01")).unwrap();
    let birth_id = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![]).unwrap();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    service::save_birth_with_calves(&mut conn, &mut history, 42, birth.clone(), vec![first, second]).unwrap();
    assert_eq!(service::get_cows_born_on_a_given_birth(&mut conn, 42, birth_id).unwrap().len(), 2);

    service::save_birth_with_calves(&mut conn, &mut history, 42, birth, vec![]).unwrap();
    assert!(service::get_cows_born_on_a_given_birth(&mut conn, 42, birth_id).unwrap().is_empty());
    assert!(service::get_birth(&mut conn, 42, birth_id).is_ok());
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_cows_born_on_a_given_birth(&mut conn, 42, birth_id).unwrap().len(), 2);
    history.redo(&mut conn).unwrap();
    assert!(service::get_cows_born_on_a_given_birth(&mut conn, 42, birth_id).unwrap().is_empty());
}

#[test]
fn invalid_batch_of_calves_does_not_change_birth_or_undo_history() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let first = service::add_cow(&mut conn, &mut history, 42, newborn("RO-FIRST", "2025-05-01")).unwrap();
    let second = service::add_cow(&mut conn, &mut history, 42, newborn("RO-SECOND", "2025-05-01")).unwrap();
    let wrong_date = service::add_cow(&mut conn, &mut history, 42, newborn("RO-WRONG", "2025-05-02")).unwrap();
    let birth_id = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![first]).unwrap();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    let depth = history.undo_stack.len();

    assert!(service::save_birth_with_calves(&mut conn, &mut history, 42, birth.clone(), vec![first, second, second]).is_err());
    assert!(service::save_birth_with_calves(&mut conn, &mut history, 42, birth.clone(), vec![first, second, wrong_date]).is_err());
    assert_eq!(history.undo_stack.len(), depth);
    assert_eq!(service::get_birth(&mut conn, 42, birth_id).unwrap().date, birth.date);
    assert_eq!(service::get_cow(&mut conn, first, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_cow(&mut conn, second, 42).unwrap().birth_id, None);
    assert_eq!(service::get_cow(&mut conn, wrong_date, 42).unwrap().birth_id, None);
}

#[test]
fn batch_cannot_use_mother_or_move_calf_from_another_birth() {
    let (mut conn, mut history) = setup();
    let first_mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER-1")).unwrap();
    let second_mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER-2")).unwrap();
    let calf = service::add_cow(&mut conn, &mut history, 42, newborn("RO-CALF", "2025-05-01")).unwrap();
    let first_birth = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: first_mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![calf]).unwrap();
    let second_birth = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: second_mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![]).unwrap();
    let birth = service::get_birth(&mut conn, 42, second_birth).unwrap();
    let depth = history.undo_stack.len();

    assert!(service::save_birth_with_calves(&mut conn, &mut history, 42, birth.clone(), vec![calf]).is_err());
    assert!(service::save_birth_with_calves(&mut conn, &mut history, 42, birth, vec![second_mother]).is_err());
    assert_eq!(history.undo_stack.len(), depth);
    assert_eq!(service::get_cow(&mut conn, calf, 42).unwrap().birth_id, Some(first_birth));
    assert!(service::get_cows_born_on_a_given_birth(&mut conn, 42, second_birth).unwrap().is_empty());
}

#[test]
fn changing_birth_date_replaces_its_calves_in_one_undoable_edit() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let old_calf = service::add_cow(&mut conn, &mut history, 42, newborn("RO-OLD", "2025-05-01")).unwrap();
    let new_calf = service::add_cow(&mut conn, &mut history, 42, newborn("RO-NEW", "2025-05-02")).unwrap();
    let birth_id = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![old_calf]).unwrap();
    let mut edited = service::get_birth(&mut conn, 42, birth_id).unwrap();
    edited.date = date("2025-05-02");
    service::save_birth_with_calves(&mut conn, &mut history, 42, edited, vec![new_calf]).unwrap();
    assert_eq!(service::get_birth(&mut conn, 42, birth_id).unwrap().date, date("2025-05-02"));
    assert_eq!(service::get_cow(&mut conn, old_calf, 42).unwrap().birth_id, None);
    assert_eq!(service::get_cow(&mut conn, new_calf, 42).unwrap().birth_id, Some(birth_id));
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_birth(&mut conn, 42, birth_id).unwrap().date, date("2025-05-01"));
    assert_eq!(service::get_cow(&mut conn, old_calf, 42).unwrap().birth_id, Some(birth_id));
    assert_eq!(service::get_cow(&mut conn, new_calf, 42).unwrap().birth_id, None);
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_cow(&mut conn, new_calf, 42).unwrap().birth_id, Some(birth_id));
}

#[test]
fn deleting_birth_unlinks_every_calf_and_undo_restores_the_family() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let first = service::add_cow(&mut conn, &mut history, 42, newborn("RO-FIRST", "2025-05-01")).unwrap();
    let second = service::add_cow(&mut conn, &mut history, 42, newborn("RO-SECOND", "2025-05-01")).unwrap();
    let birth_id = service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![first, second]).unwrap();

    assert!(service::delete_birth(&mut conn, &mut history, 42, birth_id).unwrap());
    assert!(service::get_birth(&mut conn, 42, birth_id).is_err());
    for id in [first, second] {
        assert_eq!(service::get_cow(&mut conn, id, 42).unwrap().birth_id, None);
    }
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_birth(&mut conn, 42, birth_id).unwrap().mother_id, mother);
    for id in [first, second] {
        assert_eq!(service::get_cow(&mut conn, id, 42).unwrap().birth_id, Some(birth_id));
    }
    history.redo(&mut conn).unwrap();
    assert!(service::get_birth(&mut conn, 42, birth_id).is_err());
}

#[test]
fn editing_calf_details_without_changing_birth_date_keeps_its_birth() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let calf_id = service::add_cow_with_birth(&mut conn, &mut history, 42, newborn("RO-CALF", "2025-05-01"), mother).unwrap();
    let original = service::get_cow(&mut conn, calf_id, 42).unwrap();
    let mut edited = original.clone();
    edited.ear_tag = "RO-RENAMED".into();
    edited.category = Category::Lapte;
    assert!(service::update_cow(&mut conn, &mut history, edited).unwrap());
    let saved = service::get_cow(&mut conn, calf_id, 42).unwrap();
    assert_eq!(saved.birth_id, original.birth_id);
    assert_eq!(saved.ear_tag, "RO-RENAMED");
    history.undo(&mut conn).unwrap();
    let restored = service::get_cow(&mut conn, calf_id, 42).unwrap();
    assert_eq!(restored.birth_id, original.birth_id);
    assert_eq!(restored.ear_tag, original.ear_tag);
}

#[test]
fn calves_and_mothers_from_another_farm_cannot_be_linked() {
    let (mut conn, mut history) = setup();
    conn.execute("INSERT INTO farms(id, name) VALUES (99, 'Altă fermă')", []).unwrap();
    let local_mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-LOCAL-MOTHER")).unwrap();
    let foreign_mother = service::add_cow(&mut conn, &mut history, 99, cow("RO-FOREIGN-MOTHER")).unwrap();
    let foreign_calf = service::add_cow(&mut conn, &mut history, 99, newborn("RO-FOREIGN-CALF", "2025-05-01")).unwrap();
    let depth = history.undo_stack.len();

    assert!(service::add_cow_with_birth(&mut conn, &mut history, 42, newborn("RO-LOCAL-CALF", "2025-05-01"), foreign_mother).is_err());
    assert!(service::save_birth_with_calves(&mut conn, &mut history, 42, Birth {
        id: None, mother_id: local_mother, date: date("2025-05-01"), farm_id: 42,
    }, vec![foreign_calf]).is_err());
    assert_eq!(history.undo_stack.len(), depth);
    assert!(service::get_births(&mut conn, 42).unwrap().is_empty());
    assert!(service::get_cow_by_eartag(&mut conn, 42, "RO-LOCAL-CALF").is_err());
    assert_eq!(service::get_cow(&mut conn, foreign_calf, 99).unwrap().birth_id, None);
}

#[test]
fn deleting_a_sire_keeps_insemination_and_undo_restores_its_reference() {
    let (mut conn, mut history) = setup();
    let dam = service::add_cow(&mut conn, &mut history, 42, cow("RO-DAM")).unwrap();
    let mut male = cow("RO-SIRE");
    male.sex = Sex::Male;
    let sire = service::add_cow(&mut conn, &mut history, 42, male).unwrap();
    let insemination = service::add_insemination(&mut conn, &mut history, Insemination {
        id: None, dam_id: dam, sire_id: Some(sire), date: date("2025-05-01"), farm_id: 42,
    }).unwrap();

    assert!(service::delete_cow(&mut conn, &mut history, 42, sire).unwrap());
    assert_eq!(service::get_insemination(&mut conn, 42, insemination).unwrap().sire_id, None);
    history.undo(&mut conn).unwrap();
    assert_eq!(service::get_insemination(&mut conn, 42, insemination).unwrap().sire_id, Some(sire));
    history.redo(&mut conn).unwrap();
    assert_eq!(service::get_insemination(&mut conn, 42, insemination).unwrap().sire_id, None);
}

#[test]
fn changing_a_mothers_sex_and_undo_restores_all_calf_links() {
    let (mut conn, mut history) = setup();
    let mother = service::add_cow(&mut conn, &mut history, 42, cow("RO-MOTHER")).unwrap();
    let birth_id = service::save_birth_with_calves(
        &mut conn,
        &mut history,
        42,
        Birth { id: None, mother_id: mother, date: date("2025-01-01"), farm_id: 42 },
        vec![],
    ).unwrap();
    let calves: Vec<_> = ["RO-TWIN-A", "RO-TWIN-B"].into_iter().map(|tag| {
        service::add_cow(&mut conn, &mut history, 42, newborn(tag, "2025-01-01")).unwrap()
    }).collect();
    let birth = service::get_birth(&mut conn, 42, birth_id).unwrap();
    service::save_birth_with_calves(&mut conn, &mut history, 42, birth, calves.clone()).unwrap();
    let second_birth = service::save_birth_with_calves(
        &mut conn,
        &mut history,
        42,
        Birth { id: None, mother_id: mother, date: date("2025-02-01"), farm_id: 42 },
        vec![],
    ).unwrap();
    let later_calf = service::add_cow(&mut conn, &mut history, 42, newborn("RO-LATER", "2025-02-01")).unwrap();
    let birth = service::get_birth(&mut conn, 42, second_birth).unwrap();
    service::save_birth_with_calves(&mut conn, &mut history, 42, birth, vec![later_calf]).unwrap();

    let mut edited = service::get_cow(&mut conn, mother, 42).unwrap();
    edited.sex = Sex::Male;
    assert!(service::update_cow(&mut conn, &mut history, edited).unwrap());
    assert!(service::get_birth(&mut conn, 42, birth_id).is_err());
    assert!(service::get_birth(&mut conn, 42, second_birth).is_err());
    for id in &calves { assert_eq!(service::get_cow(&mut conn, *id, 42).unwrap().birth_id, None); }
    assert_eq!(service::get_cow(&mut conn, later_calf, 42).unwrap().birth_id, None);

    history.undo(&mut conn).unwrap();
    assert!(service::get_birth(&mut conn, 42, birth_id).is_ok());
    assert!(service::get_birth(&mut conn, 42, second_birth).is_ok());
    for id in &calves { assert_eq!(service::get_cow(&mut conn, *id, 42).unwrap().birth_id, Some(birth_id)); }
    assert_eq!(service::get_cow(&mut conn, later_calf, 42).unwrap().birth_id, Some(second_birth));
    history.redo(&mut conn).unwrap();
    for id in &calves { assert_eq!(service::get_cow(&mut conn, *id, 42).unwrap().birth_id, None); }
    assert_eq!(service::get_cow(&mut conn, later_calf, 42).unwrap().birth_id, None);
}
