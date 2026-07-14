// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]


mod model;
mod database;
mod command;
mod service;
mod utils;
mod controller;

fn main() {

    let service = service::service::Service::new();

    tauri::Builder::default()
    //.manage(Mutex::new(service))

    .run(tauri::generate_context!()).expect("error while running tauri application");
    cowmanagementservice_lib::run()
}
