use chrono::NaiveDate;
use cowmanagementservice_lib::{
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
fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    conn.execute(
        "INSERT INTO farms (id, name) VALUES (1, 'Prima'), (2, 'A doua')",
        [],
    )
    .unwrap();
    conn
}
fn cow(tag: &str, farm_id: i64, born: &str, entered: &str) -> Cow {
    Cow {
        id: None,
        farm_id,
        ear_tag: tag.into(),
        sex: Sex::Female,
        breed: Breed::Metis,
        category: Category::Carne,
        birth_date: date(born),
        entry_date: date(entered),
        exit_date: None,
        birth_id: None,
        birth_count: 0,
        insemination_count: 0,
    }
}

#[test]
fn cow_crud_and_tag_lookup_are_scoped_to_the_farm() {
    let conn = setup();
    let id =
        cow_query::insert_cow(&conn, &cow("RO-100", 1, "2020-01-01", "2020-02-01"), 1).unwrap();
    assert_eq!(
        cow_query::get_cow_by_eartag(&conn, "RO-100", 1).unwrap().id,
        Some(id)
    );
    assert!(cow_query::get_cow(&conn, id, 2).is_err());
    assert!(cow_query::get_cow_by_eartag(&conn, "RO-100", 2).is_err());
    let mut edited = cow_query::get_cow(&conn, id, 1).unwrap();
    edited.ear_tag = "RO-101".into();
    edited.category = Category::Lapte;
    assert!(!cow_query::update_cow(&conn, &edited, 2).unwrap());
    assert!(cow_query::update_cow(&conn, &edited, 1).unwrap());
    assert_eq!(
        cow_query::get_cow_by_eartag(&conn, "RO-101", 1)
            .unwrap()
            .category,
        Category::Lapte
    );
    assert_eq!(cow_query::get_cows(&conn, 1).unwrap().len(), 1);
    assert!(cow_query::get_cows(&conn, 2).unwrap().is_empty());
    assert!(!cow_query::delete_cow(&conn, id, 2).unwrap());
    assert!(cow_query::delete_cow(&conn, id, 1).unwrap());
    assert!(cow_query::get_cow(&conn, id, 1).is_err());
}

#[test]
fn unassigned_calves_and_presence_queries_respect_dates_and_farms() {
    let conn = setup();
    let mother =
        cow_query::insert_cow(&conn, &cow("RO-MOTHER", 1, "2020-01-01", "2020-02-01"), 1).unwrap();
    let birth_id = birth_query::insert_birth(
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
    let available = cow_query::insert_cow(
        &conn,
        &cow("RO-AVAILABLE", 1, "2025-01-01", "2025-01-02"),
        1,
    )
    .unwrap();
    let mut linked = cow("RO-LINKED", 1, "2025-01-01", "2025-01-01");
    linked.birth_id = Some(birth_id);
    cow_query::insert_cow(&conn, &linked, 1).unwrap();
    let mut exited = cow("RO-EXITED", 1, "2020-01-01", "2020-02-01");
    exited.exit_date = Some(date("2025-01-01"));
    cow_query::insert_cow(&conn, &exited, 1).unwrap();
    cow_query::insert_cow(&conn, &cow("RO-FOREIGN", 2, "2025-01-01", "2025-01-01"), 2).unwrap();
    let unassigned =
        cow_query::get_unassigned_calves_on_date(&conn, &date("2025-01-01"), 1).unwrap();
    assert_eq!(
        unassigned.iter().map(|c| c.id.unwrap()).collect::<Vec<_>>(),
        vec![available]
    );
    let present = cow_query::get_cows_in_the_plantation(&conn, &date("2025-01-01"), 1).unwrap();
    let tags: Vec<_> = present.iter().map(|c| c.ear_tag.as_str()).collect();
    assert!(tags.contains(&"RO-MOTHER"));
    assert!(tags.contains(&"RO-LINKED"));
    assert!(!tags.contains(&"RO-AVAILABLE"));
    assert!(!tags.contains(&"RO-EXITED"));
    assert!(!tags.contains(&"RO-FOREIGN"));
}

#[test]
fn birth_queries_keep_one_event_per_mother_and_day_and_list_all_calves() {
    let conn = setup();
    let mother =
        cow_query::insert_cow(&conn, &cow("RO-MOTHER", 1, "2020-01-01", "2020-02-01"), 1).unwrap();
    let other =
        cow_query::insert_cow(&conn, &cow("RO-OTHER", 1, "2020-01-01", "2020-02-01"), 1).unwrap();
    let birth = Birth {
        id: None,
        mother_id: mother,
        date: date("2025-01-01"),
        farm_id: 1,
    };
    let id = birth_query::insert_birth(&conn, &birth, 1).unwrap();
    assert!(birth_query::insert_birth(&conn, &birth, 1).is_err());
    birth_query::insert_birth(
        &conn,
        &Birth {
            mother_id: other,
            ..birth.clone()
        },
        1,
    )
    .unwrap();
    assert_eq!(
        birth_query::get_birth_by_mother_and_date(&conn, mother, "2025-01-01", 1)
            .unwrap()
            .id,
        Some(id)
    );
    assert_eq!(
        birth_query::get_births_by_mother(&conn, mother, 1)
            .unwrap()
            .len(),
        1
    );
    assert!(birth_query::get_births_by_mother(&conn, mother, 2)
        .unwrap()
        .is_empty());
    for tag in ["RO-TWIN-1", "RO-TWIN-2"] {
        let mut calf = cow(tag, 1, "2025-01-01", "2025-01-01");
        calf.birth_id = Some(id);
        cow_query::insert_cow(&conn, &calf, 1).unwrap();
    }
    assert_eq!(
        cow_query::get_cows_born_on_a_given_birth(&conn, id, 1)
            .unwrap()
            .len(),
        2
    );
    assert_eq!(cow_query::get_cow(&conn, mother, 1).unwrap().birth_count, 1);
    assert!(birth_query::delete_birth(&conn, id, 1).unwrap());
    assert!(cow_query::get_cows_born_on_a_given_birth(&conn, id, 1)
        .unwrap()
        .is_empty());
}

#[test]
fn insemination_queries_filter_by_dam_sire_date_and_farm() {
    let conn = setup();
    let dam =
        cow_query::insert_cow(&conn, &cow("RO-DAM", 1, "2020-01-01", "2020-02-01"), 1).unwrap();
    let mut sire_cow = cow("RO-SIRE", 1, "2020-01-01", "2020-02-01");
    sire_cow.sex = Sex::Male;
    let sire = cow_query::insert_cow(&conn, &sire_cow, 1).unwrap();
    let event = Insemination {
        id: None,
        dam_id: dam,
        sire_id: Some(sire),
        date: date("2025-02-03"),
        farm_id: 1,
    };
    let id = insemination_query::insert_insemination(&conn, &event, 1).unwrap();
    assert_eq!(
        insemination_query::get_inseminations_by_dam(&conn, dam, 1)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        insemination_query::get_inseminations_by_sire(&conn, sire, 1)
            .unwrap()
            .len(),
        1
    );
    assert_eq!(
        insemination_query::get_insemination_by_dam_and_date(&conn, dam, "2025-02-03", 1)
            .unwrap()
            .id,
        Some(id)
    );
    assert_eq!(
        insemination_query::get_insemination_by_sire_and_date(&conn, sire, "2025-02-03", 1)
            .unwrap()
            .id,
        Some(id)
    );
    assert!(insemination_query::get_inseminations_by_dam(&conn, dam, 2)
        .unwrap()
        .is_empty());
    assert!(insemination_query::insert_insemination(&conn, &event, 1).is_err());
    assert_eq!(
        cow_query::get_cow(&conn, dam, 1)
            .unwrap()
            .insemination_count,
        1
    );
    assert_eq!(
        cow_query::get_cow(&conn, sire, 1)
            .unwrap()
            .insemination_count,
        1
    );
    let mut edited = insemination_query::get_insemination(&conn, id, 1).unwrap();
    edited.sire_id = None;
    assert!(insemination_query::update_insemination(&conn, &edited, 1).unwrap());
    assert!(
        insemination_query::get_inseminations_by_sire(&conn, sire, 1)
            .unwrap()
            .is_empty()
    );
    assert!(insemination_query::delete_insemination(&conn, id, 1).unwrap());
    assert!(insemination_query::get_insemination(&conn, id, 1).is_err());
}
