// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;
mod database;
mod command;
mod service;
mod utils;
mod controller;
use tauri::Manager;

fn main() {
    tauri::Builder::default()
    .setup(|app| {
        let app_handle = app.handle();
        let conn = database::database::init_db(&app_handle)
            .expect("Failed to initialize the database.");
        let service = service::service::Service::new(conn);
        app.manage(std::sync::Mutex::new(service));
        Ok(())
    })
    .invoke_handler(tauri::generate_handler![
        controller::controller::add_cow,
        controller::controller::update_cow,
        controller::controller::delete_cow,
        controller::controller::add_birth,
        controller::controller::update_birth,
        controller::controller::delete_birth,
        controller::controller::add_insemination,
        controller::controller::update_insemination,
        controller::controller::delete_insemination,
        controller::controller::assign_calf_to_birth,
        controller::controller::undo,
        controller::controller::redo,
        controller::controller::get_cow,
        controller::controller::get_cows,
        controller::controller::get_cow_by_eartag,
        controller::controller::get_unassigned_calves_on_date,
        controller::controller::get_cows_in_the_plantation,
        controller::controller::get_cows_born_on_a_given_birth,
        controller::controller::get_cows_filtered,
        controller::controller::get_birth,
        controller::controller::get_births,
        controller::controller::get_births_by_mother,
        controller::controller::get_birth_by_mother_and_date,
        controller::controller::get_insemination,
        controller::controller::get_inseminations,
        controller::controller::get_inseminations_by_dam,
        controller::controller::get_inseminations_by_sire,
        controller::controller::get_insemination_by_dam_and_date,
        controller::controller::get_insemination_by_sire_and_date,
        controller::controller::export_to_xlsx

        
    ])
    .run(tauri::generate_context!()).expect("error while running tauri application");
    cowmanagementservice_lib::run()
}
