use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Farm {
    pub id: Option<i64>,
    pub name: String,
}