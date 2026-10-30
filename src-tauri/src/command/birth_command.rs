use rusqlite::Connection;
use crate::model::birth::Birth;
use crate::model::cow::Cow;
use crate::database::query::{cow_query, birth_query};
use crate::command::command::Command;
use std::any::Any;
use std::collections::HashSet;


#[derive(Debug)]
pub struct AddBirthCommand {
    pub birth: Birth,
    pub return_value: i64,
}

impl AddBirthCommand {
    pub fn new(birth: Birth) -> Self {
        Self {
            birth,
            return_value: 0,
        }
    }
}


impl Command for AddBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = birth_query::insert_birth(conn, &self.birth, self.birth.farm_id)
            .map_err(|e| e.to_string())?;
        self.birth.id = Some(self.return_value);
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        birth_query::delete_birth(conn, self.return_value, self.birth.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct DeleteBirthCommand {
    pub birth: Birth,
    pub return_value: bool,
    pub affected_cows: Vec<Cow>
}

impl DeleteBirthCommand {
    pub fn new(birth: Birth) -> Self {
        Self {
            birth,
            return_value: false,
            affected_cows: Vec::new(),
        }
    }
}


impl Command for DeleteBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        self.affected_cows = cow_query::get_cows_born_on_a_given_birth(&tx, self.birth.id.unwrap(), self.birth.farm_id)
            .map_err(|e| e.to_string())?;
        cow_query::remove_birth_from_cows(&tx, self.birth.id.unwrap(), self.birth.farm_id)
            .map_err(|e| e.to_string())?;
        self.return_value = birth_query::delete_birth(&tx, self.birth.id.unwrap(), self.birth.farm_id)
            .map_err(|e| e.to_string())?;
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        birth_query::insert_birth(&tx, &self.birth, self.birth.farm_id)
            .map_err(|e| e.to_string())?;
        for cow in &self.affected_cows {
            cow_query::update_cow(&tx, cow, self.birth.farm_id)
                .map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

#[derive(Debug)]
pub struct UpdateBirthCommand {
    pub old_birth: Birth,
    pub new_birth: Birth,
    pub return_value: bool,
}

impl UpdateBirthCommand {
    pub fn new(old_birth: Birth, new_birth: Birth) -> Self {
        Self {
            old_birth,
            new_birth,
            return_value: false,
        }
    }
}


impl Command for UpdateBirthCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        self.return_value = birth_query::update_birth(conn, &self.new_birth, self.old_birth.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        birth_query::update_birth(conn, &self.old_birth, self.new_birth.farm_id)
            .map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

/// Saves the event and an optional existing calf as one undoable transaction.
#[derive(Debug)]
pub struct SaveBirthWithCalfCommand {
    pub birth: Birth,
    old_birth: Option<Birth>,
    calf_id: Option<i64>,
    previous_calf_id: Option<i64>,
    old_calves: Vec<Cow>,
}

impl SaveBirthWithCalfCommand {
    pub fn new(birth: Birth, old_birth: Option<Birth>, calf_id: Option<i64>, previous_calf_id: Option<i64>) -> Self {
        Self { birth, old_birth, calf_id, previous_calf_id, old_calves: Vec::new() }
    }
}

impl Command for SaveBirthWithCalfCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let farm = self.birth.farm_id;
        let mother = cow_query::get_cow(&tx, self.birth.mother_id, farm).map_err(|e| e.to_string())?;
        if !matches!(mother.sex, crate::model::cow::Sex::Female)
            || self.birth.date < mother.birth_date || self.birth.date > chrono::Local::now().date_naive() {
            return Err("Data fătării sau mama selectată nu este validă.".into());
        }
        let previous = self.previous_calf_id.map(|id| cow_query::get_cow(&tx, id, farm)).transpose().map_err(|e| e.to_string())?;
        if let Some(previous) = &previous {
            if self.old_birth.is_none() || previous.birth_id != self.old_birth.as_ref().and_then(|b| b.id) {
                return Err("Asocierea vițelului nu mai corespunde. Reîncarcă datele înainte de a salva.".into());
            }
        }
        if let Some(old) = &self.old_birth {
            let calves = cow_query::get_cows_born_on_a_given_birth(&tx, old.id.unwrap(), farm).map_err(|e| e.to_string())?;
            if calves.iter().filter(|c| c.id != self.previous_calf_id || c.id == self.calf_id).any(|c| c.birth_date != self.birth.date || c.id == Some(self.birth.mother_id)) {
                return Err("Data fătării trebuie să coincidă cu nașterea vițeilor asociați.".into());
            }
        }
        let calf = self.calf_id.map(|id| cow_query::get_cow(&tx, id, farm)).transpose().map_err(|e| e.to_string())?;
        if let Some(calf) = &calf {
            if calf.id == Some(self.birth.mother_id) {
                return Err("Mama nu poate fi selectată ca vițel.".into());
            }
            if calf.birth_id.is_some() && calf.birth_id != self.old_birth.as_ref().and_then(|b| b.id) {
                return Err("Vițelul nu poate fi asociat: are deja o altă fătare de origine.".into());
            }
            if calf.birth_date != self.birth.date {
                return Err(format!("Data nașterii vițelului {} este {}, iar data fătării este {}. Cele două date trebuie să coincidă.", calf.ear_tag, calf.birth_date.format("%d.%m.%Y"), self.birth.date.format("%d.%m.%Y")));
            }
        }
        if let Some(previous) = &previous {
            if previous.id != self.calf_id {
                let mut unlinked = previous.clone();
                unlinked.birth_id = None;
                if !cow_query::update_cow(&tx, &unlinked, farm).map_err(|e| e.to_string())? {
                    return Err("Vițelul anterior nu mai există.".into());
                }
            }
        }
        let id = if self.old_birth.is_some() {
            if !birth_query::update_birth(&tx, &self.birth, farm).map_err(|e| e.to_string())? {
                return Err("Fătarea nu mai există.".into());
            }
            self.birth.id.unwrap()
        } else {
            birth_query::insert_birth(&tx, &self.birth, farm).map_err(|e| e.to_string())?
        };
        if let Some(calf) = &calf {
            let mut linked = calf.clone();
            linked.birth_id = Some(id);
            if !cow_query::update_cow(&tx, &linked, farm).map_err(|e| e.to_string())? {
                return Err("Vițelul nu mai există.".into());
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        self.birth.id = Some(id);
        self.old_calves = previous.into_iter().collect();
        if let Some(calf) = calf {
            if !self.old_calves.iter().any(|c| c.id == calf.id) { self.old_calves.push(calf); }
        }
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for calf in &self.old_calves {
            cow_query::update_cow(&tx, calf, self.birth.farm_id).map_err(|e| e.to_string())?;
        }
        if let Some(old) = &self.old_birth {
            birth_query::update_birth(&tx, old, old.farm_id).map_err(|e| e.to_string())?;
        } else {
            birth_query::delete_birth(&tx, self.birth.id.unwrap(), self.birth.farm_id).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}

/// Saves the complete set of calves for a birth in one undoable transaction.
#[derive(Debug)]
pub struct SaveBirthWithCalvesCommand {
    pub birth: Birth,
    old_birth: Option<Birth>,
    calf_ids: Vec<i64>,
    old_calves: Vec<Cow>,
}

impl SaveBirthWithCalvesCommand {
    pub fn new(birth: Birth, old_birth: Option<Birth>, calf_ids: Vec<i64>) -> Self {
        Self { birth, old_birth, calf_ids, old_calves: Vec::new() }
    }
}

impl Command for SaveBirthWithCalvesCommand {
    fn execute(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        let farm = self.birth.farm_id;
        let mother = cow_query::get_cow(&tx, self.birth.mother_id, farm).map_err(|e| e.to_string())?;
        if !matches!(mother.sex, crate::model::cow::Sex::Female)
            || self.birth.date < mother.birth_date || self.birth.date > chrono::Local::now().date_naive() {
            return Err("Data fătării sau mama selectată nu este validă.".into());
        }
        let wanted: HashSet<i64> = self.calf_ids.iter().copied().collect();
        if wanted.len() != self.calf_ids.len() {
            return Err("Un vițel apare de mai multe ori în listă.".into());
        }
        let old_id = self.old_birth.as_ref().and_then(|birth| birth.id);
        let existing = old_id.map(|id| cow_query::get_cows_born_on_a_given_birth(&tx, id, farm))
            .transpose().map_err(|e| e.to_string())?.unwrap_or_default();
        let mut selected = Vec::with_capacity(self.calf_ids.len());
        for id in &self.calf_ids {
            let calf = cow_query::get_cow(&tx, *id, farm).map_err(|e| e.to_string())?;
            if calf.id == Some(self.birth.mother_id) {
                return Err("Mama nu poate fi selectată ca vițel.".into());
            }
            if calf.birth_id.is_some() && calf.birth_id != old_id {
                return Err("Vițelul este deja asociat unei alte fătări.".into());
            }
            if calf.birth_date != self.birth.date {
                return Err(format!("Data nașterii vițelului {} trebuie să coincidă cu data fătării.", calf.ear_tag));
            }
            selected.push(calf);
        }
        let id = if self.old_birth.is_some() {
            if !birth_query::update_birth(&tx, &self.birth, farm).map_err(|e| e.to_string())? {
                return Err("Fătarea nu mai există.".into());
            }
            self.birth.id.unwrap()
        } else {
            birth_query::insert_birth(&tx, &self.birth, farm).map_err(|e| e.to_string())?
        };
        let mut old_calves = existing.clone();
        for calf in &selected {
            if !old_calves.iter().any(|old| old.id == calf.id) { old_calves.push(calf.clone()); }
        }
        for mut calf in existing {
            if !wanted.contains(&calf.id.unwrap()) {
                calf.birth_id = None;
                if !cow_query::update_cow(&tx, &calf, farm).map_err(|e| e.to_string())? {
                    return Err("Vițelul asociat nu mai există.".into());
                }
            }
        }
        for mut calf in selected {
            if calf.birth_id != Some(id) {
                calf.birth_id = Some(id);
                if !cow_query::update_cow(&tx, &calf, farm).map_err(|e| e.to_string())? {
                    return Err("Vițelul selectat nu mai există.".into());
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        self.birth.id = Some(id);
        self.old_calves = old_calves;
        Ok(())
    }

    fn undo(&mut self, conn: &mut Connection) -> Result<(), String> {
        let tx = conn.transaction().map_err(|e| e.to_string())?;
        for calf in &self.old_calves {
            cow_query::update_cow(&tx, calf, self.birth.farm_id).map_err(|e| e.to_string())?;
        }
        if let Some(old) = &self.old_birth {
            birth_query::update_birth(&tx, old, old.farm_id).map_err(|e| e.to_string())?;
        } else {
            birth_query::delete_birth(&tx, self.birth.id.unwrap(), self.birth.farm_id).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(())
    }

    fn as_any_mut(&mut self) -> &mut dyn Any { self }
}
