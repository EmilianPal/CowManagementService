use rusqlite::Connection;
use bcrypt::{verify, hash};
use crate::auth::user::{Role, User};
use crate::model::farm::Farm;
use crate::database::query::{user_query, farm_query}; // Assuming you put the query here

pub fn authenticate_user(conn: &Connection, username: &str, password: &str) -> Result<User, String> {
    // 1. Fetch the user from the database
    let user = user_query::get_user_by_username(conn, username)?
        .ok_or("Invalid username or password")?;

    // 2. Verify the password
    let is_valid = verify(password, &user.password_hash)
        .map_err(|_| "Failed to verify password")?;

    if !is_valid {
        return Err("Invalid username or password".to_string());
    }

    Ok(user)
}

pub fn register_admin_and_farm(conn: &mut Connection,farm_name: &str, username: &str, email: &str, password: &str) -> Result<User, String> {
    if username.trim().is_empty() || password.trim().is_empty() || farm_name.trim().is_empty() {
        return Err("All fields are required".to_string());
    }
    let hashed_password = hash(password, bcrypt::DEFAULT_COST)
        .map_err(|_| "Failed to hash password")?;
    let tx = conn.transaction().map_err(|e| e.to_string())?;

    let mut new_farm = Farm {
        id: None,
        name: farm_name.to_string(),
    };


    // 4. Create the Farm
    let farm_id = farm_query::insert_farm(&tx, &new_farm)
        .map_err(|e| format!("Failed to create farm: {}", e))?;

    new_farm.id = Some(farm_id);


    // 5. Build the Admin user
    let mut new_user = User {
        id: None,
        username: username.to_string(),
        email: email.to_string(),
        password_hash: hashed_password,
        role: Role::Admin, // First user of a new farm is always Admin
        farm_id,
    };

    // 6. Insert the User into the database
    let user_id = user_query::insert_user(&tx, &new_user)
        .map_err(|e| format!("Failed to create user: {}", e))?;

    new_user.id = Some(user_id);

    // 7. Commit the transaction
    tx.commit().map_err(|e| e.to_string())?;

    Ok(new_user)
}
