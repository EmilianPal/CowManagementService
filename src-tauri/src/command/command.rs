use rustlite::Connection;

use rusqlite::Connection;
use std::fmt::Debug;

pub trait Command: Debug {
    pub fn execute(&mut self, conn: &mut Connection) -> Result<(), String>;
    pub fn undo(&mut self, conn: &mut Connection) -> Result<(), String>;
    pub fn redo(&mut self, conn: &mut Connection) -> Result<(), String>{
        self.execute(conn)
    }
}