use crate::command::command::Command;
use rusqlite::Connection;


pub struct CommandManager{
    pub undo_stack: Vec<Box<dyn Command>>,
    pub redo_stack: Vec<Box<dyn Command>>,
}

impl CommandManager {
    pub fn new() -> Self {
        Self {
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }

    pub fn execute(&mut self, mut command: Box<dyn Command>, conn: &mut Connection) -> Result<&mut dyn Command, String> { 
        command.execute(conn)?;
        self.undo_stack.push(command);
        self.redo_stack.clear();
        Ok(self.undo_stack.last_mut().unwrap().as_mut()) 
    }

    pub fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        if let Some(mut command) = self.undo_stack.pop() {
            if let Err(e) = command.undo(conn) {
                self.undo_stack.push(command);
                return Err(e);
            }

            self.redo_stack.push(command);
        }
        Ok(())
    }

    pub fn redo(&mut self, conn: &mut Connection) -> Result<(), String> {
        if let Some(mut command) = self.redo_stack.pop() {
            if let Err(e) = command.redo(conn) {
                self.redo_stack.push(command);
                return Err(e);
            }
            self.undo_stack.push(command);
        }
        Ok(())
    }
}



