use chrono::NaiveDate;
use cowmanagementservice_lib::{
    database::{database::create_tables, query::cow_query::{insert_cow, get_cows_filtered}},
    model::cow::{Cow, Breed, Category, Sex},
    utils::{cow_filter::CowFilter, xlsx_export::filter_to_messages},
};
use rusqlite::Connection;

fn date(s: &str) -> NaiveDate { s.parse().unwrap() }
fn cow(tag: &str, birth: &str) -> Cow {
    Cow { id: None, farm_id: 42, ear_tag: tag.into(), breed: Breed::Metis,
        sex: Sex::Female, category: Category::Carne, birth_date: date(birth),
        entry_date: date("2024-03-01"), exit_date: None, birth_id: None,
        birth_count: 0, insemination_count: 0 }
}
fn setup() -> Connection {
    let conn = Connection::open_in_memory().unwrap();
    create_tables(&conn).unwrap();
    conn.execute("INSERT INTO farms(id,name) VALUES (42,'Ferma A'),(43,'Ferma B')", []).unwrap();
    conn
}
fn tags(conn: &Connection, filter: CowFilter) -> Vec<String> {
    get_cows_filtered(conn, filter, 42).unwrap().into_iter().map(|c| c.ear_tag).collect()
}

#[test]
fn legacy_filters_combine_and_export_describes_them() {
    let conn = setup();
    let mut wanted = cow("RO123456789", "2024-02-15");
    wanted.exit_date = Some(date("2025-03-01"));
    insert_cow(&conn, &wanted, 42).unwrap();
    let mut other = wanted.clone();
    other.ear_tag = "RO123456780".into();
    other.entry_date = date("2024-03-02");
    insert_cow(&conn, &other, 42).unwrap();
    other.ear_tag = "RO123456781".into();
    other.entry_date = wanted.entry_date;
    other.exit_date = None;
    insert_cow(&conn, &other, 42).unwrap();
    other.ear_tag = "RO123456782".into();
    other.farm_id = 43;
    other.exit_date = wanted.exit_date;
    insert_cow(&conn, &other, 43).unwrap();

    let filter = CowFilter {
        date: Some(date("2025-03-01")), ear_tag_contains: Some("ro123".into()),
        breed: Some(Breed::Metis), sex: Some(Sex::Female), category: Some(Category::Carne),
        born_in_year: Some(2024), minimum_age_months: Some(11), maximum_age_months: Some(13),
        entered_on: Some(date("2024-03-01")), exited_on: Some(date("2025-03-01")),
        show_only_exited: true, ..Default::default()
    };
    assert_eq!(tags(&conn, filter.clone()), vec![wanted.ear_tag]);
    let messages = filter_to_messages(&filter).join("\n");
    assert!(messages.contains("Crotalia să conțină: ro123"));
    assert!(messages.contains("01.03.2024"));
    assert!(messages.contains("01.03.2025"));
    assert!(messages.contains("ieșite"));
    assert!(tags(&conn, CowFilter { sex: Some(Sex::Male), ..filter.clone() }).is_empty());
    assert!(tags(&conn, CowFilter { breed: Some(Breed::BaltataRomaneasca), ..filter.clone() }).is_empty());
    assert!(tags(&conn, CowFilter { category: Some(Category::Lapte), ..filter.clone() }).is_empty());
    assert!(tags(&conn, CowFilter { born_in_year: Some(2023), ..filter }).is_empty());
}

#[test]
fn age_limits_include_minimum_exclude_maximum_and_use_completed_months() {
    let conn = setup();
    for (tag, birth) in [("RO1", "2024-01-31"), ("RO2", "2024-02-29"), ("RO3", "2024-03-01")] {
        insert_cow(&conn, &cow(tag, birth), 42).unwrap();
    }
    let filter = CowFilter { date: Some(date("2024-02-29")), maximum_age_months: Some(1), ..Default::default() };
    assert_eq!(tags(&conn, filter.clone()), vec!["RO1", "RO2"]);
    assert!(tags(&conn, CowFilter { maximum_age_months: Some(0), ..filter }).is_empty());
    let filter = CowFilter { date: Some(date("2024-03-30")), minimum_age_months: Some(0), maximum_age_months: Some(2), ..Default::default() };
    assert_eq!(tags(&conn, filter.clone()), vec!["RO1", "RO2", "RO3"]);
    assert_eq!(tags(&conn, CowFilter { date: Some(date("2024-03-31")), ..filter }), vec!["RO2", "RO3"]);
    let filter = CowFilter { date: Some(date("2024-03-31")), minimum_age_months: Some(1), ..Default::default() };
    assert_eq!(tags(&conn, filter), vec!["RO1", "RO2"]);
    assert!(get_cows_filtered(&conn, CowFilter { minimum_age_months: Some(12), maximum_age_months: Some(12), ..Default::default() }, 42).is_err());
    assert!(get_cows_filtered(&conn, CowFilter { minimum_age_months: Some(12), maximum_age_months: Some(2), ..Default::default() }, 42).is_err());
    assert!(get_cows_filtered(&conn, CowFilter { minimum_age_months: Some(-1), ..Default::default() }, 42).is_err());
}

#[test]
fn reference_date_controls_presence_and_search_is_literal() {
    let conn = setup();
    let mut c = cow("RO123%_456", "2024-01-01");
    c.exit_date = Some(date("2024-04-01"));
    insert_cow(&conn, &c, 42).unwrap();
    insert_cow(&conn, &cow("RO123XX456", "2024-01-01"), 42).unwrap();
    let filter = CowFilter { ear_tag_contains: Some("%_".into()), ..Default::default() };
    assert_eq!(tags(&conn, filter.clone()), vec!["RO123%_456"]);
    assert!(tags(&conn, CowFilter { date: Some(date("2024-02-29")), show_only_entered: true, ..filter.clone() }).is_empty());
    assert_eq!(tags(&conn, CowFilter { date: Some(date("2024-03-01")), show_only_entered: true, ..filter.clone() }).len(), 1);
    assert!(tags(&conn, CowFilter { date: Some(date("2024-04-01")), show_only_entered: true, ..filter.clone() }).is_empty());
    assert_eq!(tags(&conn, CowFilter { date: Some(date("2024-04-01")), show_only_exited: true, ..filter }).len(), 1);
}
