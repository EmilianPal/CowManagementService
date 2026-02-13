use rusqlite::Connection;
use crate::command::command_manager::CommandManager;




pub struct Service{
    pub conn: Connection,
    pub command_manager: CommandManager

}

impl Service {
    pub fn new(conn: Connection) -> Self {
        Self {
            conn,
            command_manager: CommandManager::new(),
        }
    }
    
}