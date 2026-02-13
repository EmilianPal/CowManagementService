use chrono::NaiveDate;
use serde::{Serialize, Deserialize};
use strum_macros::{Display, EnumString};


#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum Sex {
    Male,
    Female
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum Breed {
    Metis, 
    #[strum(serialize = "Bălțata Românească")]
    BaltataRomaneasca, 
    #[strum(serialize = "Ambardeen-Angus")]
    AmbardeenAngus
}

#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq)]
pub enum Category { 
    Carne, 
    Mixt, 
    Lapte 
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Cow {
    pub id: Option<i64>,
    pub ear_tag: String,
    pub sex: Sex,
    pub breed: Breed,
    pub category: Category,
    pub birth_date: NaiveDate,
    pub entry_date: NaiveDate,
    pub exit_date: Option<NaiveDate>,
    pub birth_id: Option<i64>
}