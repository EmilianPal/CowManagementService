#[cfg(test)]
mod query_tests {
    use cowmanagementservice_lib::database::database::create_tables;
    use cowmanagementservice_lib::database::query::{birth_query, cow_query, insemination_query};
    use cowmanagementservice_lib::model::{
        birth::Birth,
        cow::{Breed, Category, Cow, Sex},
        insemination::Insemination,
    };
    use chrono::NaiveDate;
    use rusqlite::Connection;
    use std::str::FromStr;

    fn setup_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        conn
    }

    #[test]
    fn test_insert_cow() {
        let conn = setup_connection();
        let cow = Cow {
            id: None,
            ear_tag: "12345".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let cow_id = cow_query::insert_cow(&conn, &cow).unwrap();
        let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
        assert_eq!(cow.ear_tag, fetched_cow.ear_tag);
    }

    #[test]
    fn test_update_cow() {
        let conn = setup_connection();
        let mut cow = Cow {
            id: None,
            ear_tag: "12345".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let cow_id = cow_query::insert_cow(&conn, &cow).unwrap();
        cow.id = Some(cow_id);
        cow.ear_tag = "54321".to_string();
        cow_query::update_cow(&conn, &cow).unwrap();
        let fetched_cow = cow_query::get_cow(&conn, cow_id).unwrap();
        assert_eq!(cow.ear_tag, fetched_cow.ear_tag);
    }

    #[test]
    fn test_delete_cow() {
        let conn = setup_connection();
        let cow = Cow {
            id: None,
            ear_tag: "12345".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let cow_id = cow_query::insert_cow(&conn, &cow).unwrap();
        cow_query::delete_cow(&conn, cow_id).unwrap();
        let res = cow_query::get_cow(&conn, cow_id);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_cows() {
        let conn = setup_connection();
        let cow1 = Cow {
            id: None,
            ear_tag: "1".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let cow2 = Cow {
            id: None,
            ear_tag: "2".to_string(),
            sex: Sex::Male,
            breed: Breed::BaltataRomaneasca,
            category: Category::Mixt,
            birth_date: NaiveDate::from_ymd_opt(2020, 2, 2).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 2, 2).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        cow_query::insert_cow(&conn, &cow1).unwrap();
        cow_query::insert_cow(&conn, &cow2).unwrap();
        let cows = cow_query::get_cows(&conn).unwrap();
        assert_eq!(cows.len(), 2);
    }

    #[test]
    fn test_get_cow_by_eartag() {
        let conn = setup_connection();
        let cow = Cow {
            id: None,
            ear_tag: "12345".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        cow_query::insert_cow(&conn, &cow).unwrap();
        let fetched_cow = cow_query::get_cow_by_eartag(&conn, "12345").unwrap();
        assert_eq!(cow.ear_tag, fetched_cow.ear_tag);
    }
    
    #[test]
    fn test_unassigned_calves_on_date(){
        let conn = setup_connection();
        let date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        let cow = Cow {
            id: None,
            ear_tag: "123".to_string(),
            sex: Sex::from_str("Female").unwrap(),
            breed: Breed::from_str("Metis").unwrap(),
            category: Category::from_str("Carne").unwrap(),
            birth_date: date,
            entry_date: date,
            exit_date: None,
            birth_id: None,
        };
        let cow_id = cow_query::insert_cow(&conn, &cow).unwrap();
        let calves = cow_query::get_unassigned_calves_on_date(&conn, &date).unwrap();
        assert_eq!(calves.len(), 1);
        assert_eq!(calves[0].id, Some(cow_id));
    }

    #[test]
    fn test_get_cows_in_the_plantation(){
        let conn = setup_connection();
        let date = NaiveDate::from_ymd_opt(2024, 7, 1).unwrap();
        let cow = Cow {
            id: None,
            ear_tag: "123".to_string(),
            sex: Sex::from_str("Female").unwrap(),
            breed: Breed::from_str("Metis").unwrap(),
            category: Category::from_str("Carne").unwrap(),
            birth_date: date,
            entry_date: date,
            exit_date: None,
            birth_id: None,
        };
        cow_query::insert_cow(&conn, &cow).unwrap();
        let plantation_cows = cow_query::get_cows_in_the_plantation(&conn, &date).unwrap();
        assert_eq!(plantation_cows.len(), 1);
    }
    
    #[test]
    fn test_get_cows_born_on_a_given_birth(){
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::from_str("Female").unwrap(),
            breed: Breed::from_str("Metis").unwrap(),
            category: Category::from_str("Carne").unwrap(),
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth = Birth {
            id: None,
            mother_id: mother_id,
            date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()
        };
        let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();
        let calf = Cow {
            id: None,
            ear_tag: "calf".to_string(),
            sex: Sex::from_str("Female").unwrap(),
            breed: Breed::from_str("Metis").unwrap(),
            category: Category::from_str("Carne").unwrap(),
            birth_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            exit_date: None,
            birth_id: Some(birth_id),
        };
        cow_query::insert_cow(&conn, &calf).unwrap();
        let calves = cow_query::get_cows_born_on_a_given_birth(&conn, birth_id).unwrap();
        assert_eq!(calves.len(), 1);
        assert_eq!(calves[0].ear_tag, "calf");
    }
    
    #[test]
    fn test_remove_birth_from_cows(){
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::from_str("Female").unwrap(),
            breed: Breed::from_str("Metis").unwrap(),
            category: Category::from_str("Carne").unwrap(),
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth = Birth {
            id: None,
            mother_id: mother_id,
            date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap()
        };
        let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();
        let calf = Cow {
            id: None,
            ear_tag: "calf".to_string(),
            sex: Sex::from_str("Female").unwrap(),
            breed: Breed::from_str("Metis").unwrap(),
            category: Category::from_str("Carne").unwrap(),
            birth_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2024, 7, 1).unwrap(),
            exit_date: None,
            birth_id: Some(birth_id),
        };
        let calf_id = cow_query::insert_cow(&conn, &calf).unwrap();
        cow_query::remove_birth_from_cows(&conn, birth_id).unwrap();
        let fetched_calf = cow_query::get_cow(&conn, calf_id).unwrap();
        assert!(fetched_calf.birth_id.is_none());
    }

    #[test]
    fn test_insert_birth() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        };
        let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();
        let fetched_birth = birth_query::get_birth(&conn, birth_id).unwrap();
        assert_eq!(birth.mother_id, fetched_birth.mother_id);
    }
    
    #[test]
    fn test_update_birth() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let mut birth = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        };
        let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();
        birth.id = Some(birth_id);
        birth.date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        birth_query::update_birth(&conn, &birth).unwrap();
        let fetched_birth = birth_query::get_birth(&conn, birth_id).unwrap();
        assert_eq!(birth.date, fetched_birth.date);
    }

    #[test]
    fn test_delete_birth() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        };
        let birth_id = birth_query::insert_birth(&conn, &birth).unwrap();
        birth_query::delete_birth(&conn, birth_id).unwrap();
        let res = birth_query::get_birth(&conn, birth_id);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_births() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth1 = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        };
        let birth2 = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        birth_query::insert_birth(&conn, &birth1).unwrap();
        birth_query::insert_birth(&conn, &birth2).unwrap();
        let births = birth_query::get_births(&conn).unwrap();
        assert_eq!(births.len(), 2);
    }
    
    #[test]
    fn test_get_birth_by_mother_and_date() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth_date = NaiveDate::from_ymd_opt(2022, 1, 1).unwrap();
        let birth = Birth {
            id: None,
            mother_id,
            date: birth_date,
        };
        birth_query::insert_birth(&conn, &birth).unwrap();
        let fetched_birth =
            birth_query::get_birth_by_mother_and_date(&conn, mother_id, &birth_date.to_string())
                .unwrap();
        assert_eq!(birth.mother_id, fetched_birth.mother_id);
        assert_eq!(birth.date, fetched_birth.date);
    }

    #[test]
    fn test_get_births_by_mother() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth1 = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        };
        let birth2 = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        birth_query::insert_birth(&conn, &birth1).unwrap();
        birth_query::insert_birth(&conn, &birth2).unwrap();
        let births = birth_query::get_births_by_mother(&conn, mother_id).unwrap();
        assert_eq!(births.len(), 2);
    }

    #[test]
    fn test_delete_births_by_mother() {
        let conn = setup_connection();
        let mother = Cow {
            id: None,
            ear_tag: "mother".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let mother_id = cow_query::insert_cow(&conn, &mother).unwrap();
        let birth = Birth {
            id: None,
            mother_id,
            date: NaiveDate::from_ymd_opt(2022, 1, 1).unwrap(),
        };
        birth_query::insert_birth(&conn, &birth).unwrap();
        birth_query::delete_births_by_mother(&conn, mother_id).unwrap();
        let births = birth_query::get_births_by_mother(&conn, mother_id).unwrap();
        assert!(births.is_empty());
    }

    #[test]
    fn test_insert_insemination() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };

        let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();
        let fetched_insemination =
            insemination_query::get_insemination(&conn, insemination_id).unwrap();
        assert_eq!(insemination.dam_id, fetched_insemination.dam_id);
        assert_eq!(insemination.sire_id, fetched_insemination.sire_id);
    }
    
    #[test]
    fn test_update_insemination() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

        let mut insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };

        let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();
        insemination.id = Some(insemination_id);
        insemination.date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
        insemination_query::update_insemination(&conn, &insemination).unwrap();

        let fetched_insemination =
            insemination_query::get_insemination(&conn, insemination_id).unwrap();
        assert_eq!(insemination.date, fetched_insemination.date);
    }

    #[test]
    fn test_delete_insemination() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };

        let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();
        insemination_query::delete_insemination(&conn, insemination_id).unwrap();
        let res = insemination_query::get_insemination(&conn, insemination_id);
        assert!(res.is_err());
    }

    #[test]
    fn test_get_inseminations() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();
        let insemination1 = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        let insemination2 = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        };
        insemination_query::insert_insemination(&conn, &insemination1).unwrap();
        insemination_query::insert_insemination(&conn, &insemination2).unwrap();
        let inseminations = insemination_query::get_inseminations(&conn).unwrap();
        assert_eq!(inseminations.len(), 2);
    }
    
    #[test]
    fn test_get_inseminations_by_dam() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        insemination_query::insert_insemination(&conn, &insemination).unwrap();
        let inseminations = insemination_query::get_inseminations_by_dam(&conn, dam_id).unwrap();
        assert_eq!(inseminations.len(), 1);
        assert_eq!(inseminations[0].dam_id, dam_id);
    }

    #[test]
    fn test_get_inseminations_by_sire() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        insemination_query::insert_insemination(&conn, &insemination).unwrap();
        let inseminations =
            insemination_query::get_inseminations_by_sire(&conn, sire_id).unwrap();
        assert_eq!(inseminations.len(), 1);
        assert_eq!(inseminations[0].sire_id, Some(sire_id));
    }
    
    #[test]
    fn test_get_insemination_by_dam_and_date() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();
        let insemination_date = NaiveDate::from_ymd_opt(2023, 1, 1).unwrap();
        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: insemination_date,
        };
        insemination_query::insert_insemination(&conn, &insemination).unwrap();
        let fetched_insemination = insemination_query::get_insemination_by_dam_and_date(
            &conn,
            dam_id,
            &insemination_date.to_string(),
        )
        .unwrap();
        assert_eq!(insemination.dam_id, fetched_insemination.dam_id);
        assert_eq!(insemination.date, fetched_insemination.date);
    }
    
    #[test]
    fn test_delete_insemination_by_dam() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: None,
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        insemination_query::insert_insemination(&conn, &insemination).unwrap();
        insemination_query::delete_insemination_by_dam(&conn, dam_id).unwrap();
        let inseminations = insemination_query::get_inseminations_by_dam(&conn, dam_id).unwrap();
        assert!(inseminations.is_empty());
    }

    #[test]
    fn test_remove_sire_from_inseminations() {
        let conn = setup_connection();
        let dam = Cow {
            id: None,
            ear_tag: "dam".to_string(),
            sex: Sex::Female,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let sire = Cow {
            id: None,
            ear_tag: "sire".to_string(),
            sex: Sex::Male,
            breed: Breed::Metis,
            category: Category::Carne,
            birth_date: NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(2021, 1, 1).unwrap(),
            exit_date: None,
            birth_id: None,
        };
        let dam_id = cow_query::insert_cow(&conn, &dam).unwrap();
        let sire_id = cow_query::insert_cow(&conn, &sire).unwrap();

        let insemination = Insemination {
            id: None,
            dam_id,
            sire_id: Some(sire_id),
            date: NaiveDate::from_ymd_opt(2023, 1, 1).unwrap(),
        };
        let insemination_id = insemination_query::insert_insemination(&conn, &insemination).unwrap();
        insemination_query::remove_sire_from_inseminations(&conn, sire_id).unwrap();
        let fetched_insemination =
            insemination_query::get_insemination(&conn, insemination_id).unwrap();
        assert!(fetched_insemination.sire_id.is_none());
    }
}