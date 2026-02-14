use chrono::NaiveDate;
use crate::model::cow::{Sex, Breed, Category};

use serde::{Serialize, Deserialize};

#[derive(Default, Debug, Clone, Serialize, Deserialize)]
pub struct CowFilter {
    pub date: Option<NaiveDate>,
    pub last_4_digits_eartag: Option<String>,
    pub breed: Option<Breed>,
    pub sex: Option<Sex>,
    pub born_in_year: Option<i64>,
    pub born_on: Option<NaiveDate>,
    pub minimum_age_months: Option<i64>,
    pub maximum_age_months: Option<i64>,
    pub entered_on: Option<NaiveDate>,
    pub exited_on: Option<NaiveDate>,
    pub category: Option<Category>,
    pub births_less_than: Option<i64>,
    pub births_more_than: Option<i64>,
    pub inseminations_less_than: Option<i64>,
    pub inseminations_more_than: Option<i64>,
    pub show_only_entered: bool 
}