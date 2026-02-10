use chrono::NaiveDate;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Birth{
    pub id: Option<i64>,
    pub mother_id: i64,
    pub date: NaiveDate,
}