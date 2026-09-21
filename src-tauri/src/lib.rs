pub mod auth;
pub mod command;
pub mod controller;
pub mod database;
pub mod model;
pub mod service;
pub mod utils;

use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init()) 
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let app_handle = app.handle();
            let db_pool = crate::database::database::init_db(&app_handle)
                .expect("Baza de date nu a putut fi inițializată.");
            app.manage(crate::auth::session::AppState {
                db_pool,
                session: std::sync::Mutex::new(None),
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            crate::controller::controller::get_session,
            crate::controller::controller::get_history_state,
            crate::controller::controller::register_admin,
            crate::controller::controller::login_user,
            crate::controller::controller::logout,
            crate::controller::controller::add_cow,
            crate::controller::controller::update_cow,
            crate::controller::controller::exit_cows,
            crate::controller::controller::delete_cow,
            crate::controller::controller::add_birth,
            crate::controller::controller::update_birth,
            crate::controller::controller::delete_birth,
            crate::controller::controller::add_insemination,
            crate::controller::controller::update_insemination,
            crate::controller::controller::delete_insemination,
            crate::controller::controller::assign_calf_to_birth,
            crate::controller::controller::undo,
            crate::controller::controller::redo,
            crate::controller::controller::get_cow,
            crate::controller::controller::get_cows,
            crate::controller::controller::get_cow_by_eartag,
            crate::controller::controller::get_unassigned_calves_on_date,
            crate::controller::controller::get_cows_in_the_plantation,
            crate::controller::controller::get_cows_born_on_a_given_birth,
            crate::controller::controller::get_cows_filtered,
            crate::controller::controller::get_birth,
            crate::controller::controller::get_births,
            crate::controller::controller::get_births_by_mother,
            crate::controller::controller::get_birth_by_mother_and_date,
            crate::controller::controller::get_insemination,
            crate::controller::controller::get_inseminations,
            crate::controller::controller::get_inseminations_by_dam,
            crate::controller::controller::get_inseminations_by_sire,
            crate::controller::controller::get_insemination_by_dam_and_date,
            crate::controller::controller::get_insemination_by_sire_and_date,
            crate::controller::controller::export_to_xlsx,
            crate::controller::controller::export_report
        ])
        .run(tauri::generate_context!())
        .expect("Aplicația nu a putut fi pornită.");
}
