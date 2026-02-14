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

    pub fn setup_connection() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        create_tables(&conn).unwrap();
        conn
    }
    // Helper function to create a cow for testing
    pub fn new_cow(ear_tag: &str, sex: Sex, breed: Breed, birth_date: (i32, u32, u32), entry_date: (i32, u32, u32)) -> Cow {
        Cow {
            id: None,
            ear_tag: ear_tag.to_string(),
            sex,
            breed,
            category: Category::Carne, // Default for simplicity
            birth_date: NaiveDate::from_ymd_opt(birth_date.0, birth_date.1, birth_date.2).unwrap(),
            entry_date: NaiveDate::from_ymd_opt(entry_date.0, entry_date.1, entry_date.2).unwrap(),
            exit_date: None,
            birth_id: None,
        }
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

#[cfg(test)]
mod cow_query_filters_tests {
    use super::query_tests::{setup_connection, new_cow};
    use cowmanagementservice_lib::database::query::{birth_query, cow_query, insemination_query};
    use cowmanagementservice_lib::model::{
        birth::Birth,
        cow::{Breed, Sex},
        insemination::Insemination,
    };
    use cowmanagementservice_lib::database::query::cow_filter::CowFilter;
    use chrono::{NaiveDate, Local};
    use rusqlite::Connection;

    // Setup a scenario with a variety of cows for filter testing
    fn setup_filtered_cows_scenario() -> Connection {
        let conn = setup_connection();

        // --- Cows ---
        let c1 = cow_query::insert_cow(&conn, &new_cow("RO1234", Sex::Female, Breed::AmbardeenAngus, (2022, 1, 15), (2022, 2, 1))).unwrap(); // 2 births, 1 insemination
        let _c2 = cow_query::insert_cow(&conn, &new_cow("RO5678", Sex::Female, Breed::Metis, (2021, 5, 20), (2021, 6, 1))).unwrap(); // 0 births, 0 inseminations
        let c3 = cow_query::insert_cow(&conn, &new_cow("RO9012", Sex::Male, Breed::BaltataRomaneasca, (2022, 3, 10), (2022, 4, 1))).unwrap(); // 1 insemination as sire
        let _c4 = cow_query::insert_cow(&conn, &new_cow("FR3456", Sex::Female, Breed::AmbardeenAngus, (2023, 8, 1), (2023, 9, 1))).unwrap(); // Young cow
        let _c5_exited = { // Cow that has exited
            let mut cow = new_cow("EX1111", Sex::Female, Breed::Metis, (2020, 1, 1), (2020, 2, 1));
            cow.exit_date = Some(NaiveDate::from_ymd_opt(2023, 12, 31).unwrap());
            cow_query::insert_cow(&conn, &cow).unwrap()
        };


        // --- Births (for C1) ---
        birth_query::insert_birth(&conn, &Birth { id: None, mother_id: c1, date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap() }).unwrap();
        birth_query::insert_birth(&conn, &Birth { id: None, mother_id: c1, date: NaiveDate::from_ymd_opt(2025, 1, 1).unwrap() }).unwrap();

        // --- Inseminations (for C1 and C3) ---
        insemination_query::insert_insemination(&conn, &Insemination { id: None, dam_id: c1, sire_id: Some(c3), date: NaiveDate::from_ymd_opt(2024, 5, 1).unwrap() }).unwrap();

        conn
    }

    #[test]
    fn test_get_cows_filtered_empty() {
        let conn = setup_filtered_cows_scenario();
        let filter = CowFilter::default(); // No filters
        let cows = cow_query::get_cows_filtered(&conn, filter).unwrap();
        assert_eq!(cows.len(), 5); // Should return all cows
    }
    
    #[test]
    fn test_get_cows_filtered_show_only_entered() {
        let conn = setup_filtered_cows_scenario();
        
        // Reference date is today (e.g., sometime in 2026)
        let filter_present = CowFilter {
            show_only_entered: true,
            date: Some(Local::now().date_naive()), // or a fixed date like NaiveDate::from_ymd_opt(2026, 2, 14).unwrap()
            ..Default::default()
        };
        let cows_present = cow_query::get_cows_filtered(&conn, filter_present).unwrap();
        assert_eq!(cows_present.len(), 4, "Should not include the exited cow");

        // Reference date is in the past, before the cow exited
        let filter_past = CowFilter {
            show_only_entered: true,
            date: Some(NaiveDate::from_ymd_opt(2023, 1, 1).unwrap()),
            ..Default::default()
        };
        let cows_past = cow_query::get_cows_filtered(&conn, filter_past).unwrap();
        assert_eq!(cows_past.len(), 4, "Should include the cow that had not exited yet, but exclude the one not yet entered");
    }


    #[test]
    fn test_get_cows_filtered_by_sex_and_breed() {
        let conn = setup_filtered_cows_scenario();

        // Filter by Sex
        let filter_male = CowFilter { sex: Some(Sex::Male), ..Default::default() };
        let males = cow_query::get_cows_filtered(&conn, filter_male).unwrap();
        assert_eq!(males.len(), 1);
        assert_eq!(males[0].ear_tag, "RO9012");

        // Filter by Breed
        let filter_angus = CowFilter { breed: Some(Breed::AmbardeenAngus), ..Default::default() };
        let angus_cows = cow_query::get_cows_filtered(&conn, filter_angus).unwrap();
        assert_eq!(angus_cows.len(), 2);

        // Filter by both
        let filter_combo = CowFilter { sex: Some(Sex::Female), breed: Some(Breed::AmbardeenAngus), ..Default::default() };
        let female_angus = cow_query::get_cows_filtered(&conn, filter_combo).unwrap();
        assert_eq!(female_angus.len(), 2); // Both AmbardeenAngus are female in our test data
    }
    
    #[test]
    fn test_get_cows_filtered_by_eartag_digits() {
        let conn = setup_filtered_cows_scenario();
        let filter = CowFilter { last_4_digits_eartag: Some("1234".to_string()), ..Default::default() };
        let cows = cow_query::get_cows_filtered(&conn, filter).unwrap();
        assert_eq!(cows.len(), 1);
        assert_eq!(cows[0].ear_tag, "RO1234");
    }

    #[test]
    fn test_get_cows_filtered_by_year_and_age() {
        let conn = setup_filtered_cows_scenario();
        let ref_date = NaiveDate::from_ymd_opt(2024, 1, 15).unwrap();

        // Born in 2022
        let filter_year = CowFilter { born_in_year: Some(2022), ..Default::default() };
        let year_cows = cow_query::get_cows_filtered(&conn, filter_year).unwrap();
        assert_eq!(year_cows.len(), 2);

        // Age between 24 and 36 months (relative to ref_date)
        // RO1234 (born 2022-01-15) is exactly 24 months.
        // RO5678 (born 2021-05-20) is ~32 months.
        let filter_age = CowFilter {
            date: Some(ref_date),
            minimum_age_months: Some(24),
            maximum_age_months: Some(36),
            ..Default::default()
        };
        let age_cows = cow_query::get_cows_filtered(&conn, filter_age).unwrap();
        assert_eq!(age_cows.len(), 2);
        assert!(age_cows.iter().any(|c| c.ear_tag == "RO1234"));
        assert!(age_cows.iter().any(|c| c.ear_tag == "RO5678"));
    }

    #[test]
    fn test_get_cows_filtered_by_birth_count() {
        let conn = setup_filtered_cows_scenario();

        // More than 1 birth (should be just C1)
        let filter_more = CowFilter { births_more_than: Some(1), ..Default::default() };
        let cows_more = cow_query::get_cows_filtered(&conn, filter_more).unwrap();
        assert_eq!(cows_more.len(), 1);
        assert_eq!(cows_more[0].ear_tag, "RO1234");

        // Less than 1 birth (all females except C1)
        let filter_less = CowFilter { births_less_than: Some(1), ..Default::default() };
        let cows_less = cow_query::get_cows_filtered(&conn, filter_less).unwrap();
        assert_eq!(cows_less.len(), 3); // RO5678, FR3456, EX1111 (all females with 0 births)
        assert!(!cows_less.iter().any(|c| c.ear_tag == "RO1234"));
    }
    
    #[test]
    fn test_get_cows_filtered_by_insemination_count() {
        let conn = setup_filtered_cows_scenario();

        // Females with more than 0 inseminations (C1)
        let filter_females = CowFilter { sex: Some(Sex::Female), inseminations_more_than: Some(0), ..Default::default() };
        let inseminated_females = cow_query::get_cows_filtered(&conn, filter_females).unwrap();
        assert_eq!(inseminated_females.len(), 1);
        assert_eq!(inseminated_females[0].ear_tag, "RO1234");

        // Males with more than 0 inseminations (C3)
        let filter_males = CowFilter { sex: Some(Sex::Male), inseminations_more_than: Some(0), ..Default::default() };
        let inseminated_males = cow_query::get_cows_filtered(&conn, filter_males).unwrap();
        assert_eq!(inseminated_males.len(), 1);
        assert_eq!(inseminated_males[0].ear_tag, "RO9012");
        
        // Any cow with less than 1 insemination
        let filter_less = CowFilter { inseminations_less_than: Some(1), ..Default::default() };
        let cows_less = cow_query::get_cows_filtered(&conn, filter_less).unwrap();
        assert_eq!(cows_less.len(), 3); // C2, C4, C5 have 0 inseminations
    }
}
