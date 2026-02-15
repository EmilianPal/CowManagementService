use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumString};


#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq, Clone)]
pub enum Sex {
    Male,
    Female
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq, Clone)]
pub enum Breed {
    Metis, 
    #[strum(serialize = "Bălțata Românească")]
    BaltataRomaneasca, 
    #[strum(serialize = "Ambardeen-Angus")]
    AmbardeenAngus
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq, Clone)]
pub enum Category { 
    Carne, 
    Mixt, 
    Lapte 
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Cow {
    pub id: Option<i64>,
    pub ear_tag: String,
    pub sex: Sex,
    pub breed: Breed,
    pub category: Category,
    pub birth_date: NaiveDate,
    pub entry_date: NaiveDate,
    pub exit_date: Option<NaiveDate>,
    pub birth_id: Option<i64>,
    pub birth_count: i64,
    pub insemination_count: i64,
}

impl Cow {
    fn new(eartag: String, sex: Sex, breed: Breed, category: Category, birth_date: NaiveDate, entry_date: NaiveDate, exit_date: Option<NaiveDate>, birth_id: Option<i64>) -> Self {
        Self {
            id: None,
            ear_tag: eartag,
            sex,
            breed,
            category,
            birth_date,
            entry_date,
            exit_date,
            birth_id,
            birth_count: 0,
            insemination_count: 0,
        }
    }
}