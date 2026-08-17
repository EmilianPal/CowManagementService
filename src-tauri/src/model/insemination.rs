use chrono::NaiveDate;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Insemination{
    pub id: Option<i64>,
    pub dam_id: i64,
    pub sire_id: Option<i64>,
    pub date: NaiveDate,
    pub farm_id: i64
}