use chrono::NaiveDate;
use crate::utils::cow_filter::CowFilter;

use crate::service::service::Service;
use crate::model::{cow::Cow, birth::Birth, insemination::Insemination};
use std::sync::Mutex;

#[tauri::command]
pub fn add_cow(cow: Cow, service: tauri::State<'_, Mutex<Service>>) -> Result<i64, String> {
    let mut service = service.lock().unwrap();
    service.add_cow(cow)
}

#[tauri::command]
pub fn update_cow(cow: Cow, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.update_cow(cow)
}

#[tauri::command]
pub fn delete_cow(cow_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.delete_cow(cow_id)
}

#[tauri::command]
pub fn add_birth(birth: Birth, service: tauri::State<'_, Mutex<Service>>) -> Result<i64, String> {
    let mut service = service.lock().unwrap();
    service.add_birth(birth)
}

#[tauri::command]
pub fn update_birth(birth: Birth, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.update_birth(birth)
}

#[tauri::command]
pub fn delete_birth(birth_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.delete_birth(birth_id)
}

#[tauri::command]
pub fn add_insemination(insemination: Insemination, service: tauri::State<'_, Mutex<Service>>) -> Result<i64, String> {
    let mut service = service.lock().unwrap();
    service.add_insemination(insemination)
}

#[tauri::command]
pub fn update_insemination(insemination: Insemination, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.update_insemination(insemination)
}

#[tauri::command]
pub fn delete_insemination(insemination_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.delete_insemination(insemination_id)
}

#[tauri::command]
pub fn assign_calf_to_birth(calf_id: i64, birth_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<bool, String> {
    let mut service = service.lock().unwrap();
    service.assign_calf_to_birth(calf_id, birth_id)
}

#[tauri::command]
pub fn undo(service: tauri::State<'_, Mutex<Service>>) -> Result<(), String> {
    let mut service = service.lock().unwrap();
    service.undo()
}

#[tauri::command]
pub fn redo(service: tauri::State<'_, Mutex<Service>>) -> Result<(), String> {
    let mut service = service.lock().unwrap();
    service.redo()
}

#[tauri::command]
pub fn get_cow(cow_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Cow, String> {
    let service = service.lock().unwrap();
    service.get_cow(cow_id)
}

#[tauri::command]
pub fn get_cows(service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Cow>, String> {
    let service = service.lock().unwrap();
    service.get_cows()
}

#[tauri::command]
pub fn get_cow_by_eartag(eartag: &str, service: tauri::State<'_, Mutex<Service>>) -> Result<Cow, String> {
    let service = service.lock().unwrap();
    service.get_cow_by_eartag(eartag)
}

#[tauri::command]
pub fn get_unassigned_calves_on_date(date: &NaiveDate, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Cow>, String> {
    let service = service.lock().unwrap();
    service.get_unassigned_calves_on_date(date)
}

#[tauri::command]
pub fn get_cows_in_the_plantation(date: &NaiveDate, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Cow>, String> {
    let service = service.lock().unwrap();
    service.get_cows_in_the_plantation(date)
}


#[tauri::command]
pub fn get_cows_born_on_a_given_birth(birth_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Cow>, String> {
    let service = service.lock().unwrap();
    service.get_cows_born_on_a_given_birth(birth_id)
}

#[tauri::command]
pub fn get_cows_filtered(filter: CowFilter, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Cow>, String> {
    let service = service.lock().unwrap();
    service.get_cows_filtered(filter)
}

#[tauri::command]
pub fn get_birth(birth_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Birth, String> {
    let service = service.lock().unwrap();
    service.get_birth(birth_id)
}

#[tauri::command]
pub fn get_births(service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Birth>, String> {
    let service = service.lock().unwrap();
    service.get_births()
}

#[tauri::command]
pub fn get_birth_by_mother_and_date(mother_id: i64, date: &str, service: tauri::State<'_, Mutex<Service>>) -> Result<Birth, String> {
    let service = service.lock().unwrap();
    service.get_birth_by_mother_and_date(mother_id, date)
}

#[tauri::command]
pub fn get_births_by_mother(mother_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Birth>, String> {
    let service = service.lock().unwrap();
    service.get_births_by_mother(mother_id)
}

#[tauri::command]
pub fn get_insemination(insemination_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Insemination, String> {
    let service = service.lock().unwrap();
    service.get_insemination(insemination_id)
}

#[tauri::command]
pub fn get_inseminations(service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Insemination>, String> {
    let service = service.lock().unwrap();
    service.get_inseminations()
}

#[tauri::command]
pub fn get_inseminations_by_dam(dam_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Insemination>, String> {
    let service = service.lock().unwrap();
    service.get_inseminations_by_dam(dam_id)
}

#[tauri::command]
pub fn get_inseminations_by_sire(sire_id: i64, service: tauri::State<'_, Mutex<Service>>) -> Result<Vec<Insemination>, String> {
    let service = service.lock().unwrap();
    service.get_inseminations_by_sire(sire_id)
}

#[tauri::command]
pub fn get_insemination_by_dam_and_date(dam_id: i64, date: &str, service: tauri::State<'_, Mutex<Service>>) -> Result<Insemination, String> {
    let service = service.lock().unwrap();
    service.get_insemination_by_dam_and_date(dam_id, date)
}

#[tauri::command]
pub fn export_to_xlsx(path: &str, filter: CowFilter, service: tauri::State<'_, Mutex<Service>>) -> Result<(), String> {
    let service = service.lock().unwrap();
    service.export_to_xlsx(path, filter)
}

