use crate::command::command_manager::CommandManager;
use std::sync::Mutex;
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;


pub struct UserSession {
    pub user_id: i64,
    pub farm_id: i64,
    pub command_manager: CommandManager
}

pub struct AppState {
    pub db_pool: Pool<SqliteConnectionManager>,
    pub session: Mutex<Option<UserSession>>
}
