#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

extern crate sqlite;
extern crate curl;
extern crate once_cell;

mod user;
mod my_db;
mod my_log;
mod my_schedule;
mod my_config;
mod my_http_client;

use std::thread;

fn main() {
    my_log::init_log();
    my_db::init_db();
    thread::spawn(my_schedule::start_schedule);
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![user::user_info, user::user_add_score, user::search_user, user::submit_user, user::next_card, user::search_like, user::search_log])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}