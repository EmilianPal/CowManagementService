use serde::{Deserialize, Serialize};
use strum_macros::{EnumString, Display};

#[derive(Debug, Serialize, Deserialize, Display, EnumString, PartialEq, Clone)]
pub enum Role {
    Admin,   // Can do everything, including creating other users
    Editor,  // Can add/edit livestock records, but cannot manage users
    Viewer,  // Can only read data, cannot make any changes
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: Role,
    pub farm_id: i64
}