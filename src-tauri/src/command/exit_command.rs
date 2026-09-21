use super::command::Command;
use chrono::NaiveDate;
use rusqlite::{params, Connection};
use std::any::Any;

#[derive(Debug)]
pub struct ExitCows {
    pub ids: Vec<i64>,
    pub farm_id: i64,
    pub date: NaiveDate,
}

impl Command for ExitCows {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        if self.ids.is_empty() || self.date > chrono::Local::now().date_naive() {
            return Err("Selectează bovinele și o dată de ieșire care nu este în viitor.".into());
        }
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for id in &self.ids {
            let changed = tx.execute(
                "UPDATE cows SET exit_date = ?1 WHERE id = ?2 AND farm_id = ?3 AND exit_date IS NULL AND entry_date <= ?1 AND birth_date <= ?1",
                params![self.date, id, self.farm_id],
            ).map_err(|e| e.to_string())?;
            if changed != 1 {
                return Err("O bovină selectată are deja o ieșire sau nu era în fermă la data aleasă. Reîncarcă lista; nu s-a modificat nicio înregistrare.".into());
            }
        }
        tx.commit().map_err(|e| e.to_string())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for id in &self.ids {
            let changed = tx.execute("UPDATE cows SET exit_date = NULL WHERE id = ?1 AND farm_id = ?2 AND exit_date = ?3", params![id, self.farm_id, self.date]).map_err(|e| e.to_string())?;
            if changed != 1 { return Err("Ieșirea nu mai corespunde înregistrării salvate.".into()); }
        }
        tx.commit().map_err(|e| e.to_string())
    }
    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn exits_are_atomic_scoped_and_reversible() {
        let mut conn = Connection::open_in_memory().unwrap();
        conn.execute_batch("CREATE TABLE cows (id INTEGER, farm_id INTEGER, birth_date TEXT, entry_date TEXT, exit_date TEXT); INSERT INTO cows VALUES (1,1,'2020-01-01','2020-02-01',NULL),(2,1,'2020-01-01','2020-02-01',NULL),(3,2,'2020-01-01','2020-02-01',NULL);").unwrap();
        let mut batch = ExitCows { ids: vec![1,3], farm_id: 1, date: NaiveDate::from_ymd_opt(2025,1,1).unwrap() };
        assert!(batch.execute(&mut conn).is_err());
        let count = |c: &Connection| c.query_row("SELECT COUNT(*) FROM cows WHERE exit_date IS NOT NULL", [], |r| r.get::<_,i64>(0)).unwrap();
        assert_eq!(count(&conn), 0);
        batch.ids = vec![1,2];
        batch.execute(&mut conn).unwrap();
        assert_eq!(count(&conn), 2);
        assert!(batch.execute(&mut conn).is_err());
        batch.undo(&mut conn).unwrap();
        assert_eq!(count(&conn), 0);
        batch.redo(&mut conn).unwrap();
        assert_eq!(count(&conn), 2);
        batch.undo(&mut conn).unwrap();
        batch.date = NaiveDate::from_ymd_opt(2019,1,1).unwrap();
        assert!(batch.execute(&mut conn).is_err());
        assert_eq!(count(&conn), 0);
    }
}
