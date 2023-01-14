#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

extern crate sqlite;

use sqlite::State;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn user_info(card: &str) -> String {
    let connection = sqlite::open("D:/user.db").unwrap();
    let query = "SELECT * FROM users WHERE card_id = ?";
    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, card)).unwrap();
    let card_id;let name;let score;let last_change;let dress;let phone;let remarks;
    while let Ok(State::Row) = statement.next() {
        card_id = statement.read::<String, _>("card_id").unwrap();
        name = statement.read::<String, _>("name").unwrap();
        score = statement.read::<i64, _>("score").unwrap();
        last_change = statement.read::<i64, _>("last_change").unwrap();
        dress = statement.read::<String, _>("dress").unwrap();
        phone = statement.read::<i64, _>("phone").unwrap();
        remarks = statement.read::<String, _>("remarks").unwrap();
        return format!("\"card\":\"{}\", \"name\":\"{}\",\"score\":{},\"last_change\":{}, \"phone\":{}, \"dress\":\"{}\", \"remarks\":\"{}\"", card_id, name, score, last_change, phone, dress, remarks)
    }
    return "".to_string();
}

#[tauri::command]
fn user_add_score(card: &str, add_score: i32) -> String {
    let connection = sqlite::open("D:/user.db").unwrap();
    let query = format!("update users set score = score + {}, last_change = {} where card_id = '{}'", add_score, add_score, card);
    connection.execute(query).unwrap();
    return user_info(card);
}

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, user_info, user_add_score])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}