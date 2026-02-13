use rusqlite::Connection;
use std::any::Any;
use std::fmt::Debug;

pub trait Command: Debug + Any {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String>;
    fn undo(&mut self, conn: &mut Connection) -> Result<(), String>;
    fn redo(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.execute(conn)
    }
}