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
    pub farm_id: i64,
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