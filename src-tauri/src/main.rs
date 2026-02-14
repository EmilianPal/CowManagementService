// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod model;
mod database;
mod service;
mod command;
mod utils;

fn main() {
    tauri::Builder::default().run(tauri::generate_context!()).expect("error while running tauri application");
    cowmanagementservice_lib::run()
}
