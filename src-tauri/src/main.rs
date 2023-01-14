#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

extern crate sqlite;
extern crate json;
extern crate chrono;

use sqlite::{State};
use chrono::{DateTime, Local};

static DB_PATH:&str = "./user.db";

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn user_info(card: &str) -> String {
    let connection = sqlite::open(DB_PATH).unwrap();
    let query = "SELECT * FROM users WHERE card_id = ?";
    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, card)).unwrap();
    let mut result = json::JsonValue::new_array();
    while let Ok(State::Row) = statement.next() {
        let mut data = json::JsonValue::new_object();
        data["card"] = statement.read::<String, _>("card_id").unwrap().into();
        data["name"] = statement.read::<String, _>("name").unwrap().into();
        data["score"] = statement.read::<i64, _>("score").unwrap().into();
        data["last_change"] = statement.read::<i64, _>("last_change").unwrap().into();
        data["dress"] = statement.read::<String, _>("dress").unwrap().into();
        data["phone"] = statement.read::<String, _>("phone").unwrap().into();
        data["remarks"] = statement.read::<String, _>("remarks").unwrap().into();
        result.push(data).unwrap();
    }
    return json::stringify(result);
}

#[tauri::command]
fn user_add_score(card: &str, add_score: i32) -> String {
    let query = format!("update users set score = score + {}, last_change = {} where card_id = '{}'", add_score, add_score, card);
    let connection = sqlite::open(DB_PATH).unwrap();
    connection.execute(query).unwrap();
    return user_info(card);
}

#[tauri::command]
fn search_user(card: &str, name: &str, phone: &str,) -> String {
    let mut query = String::from("SELECT * FROM users WHERE 1 = 1");
    if card != "" {
        query.push_str(&format!(" and card_id = '{}'", card));
    }
    if name != "" {
        query.push_str(&format!(" and name like '%{}%'", name));
    }
    if phone != "" {
        query.push_str(&format!(" and phone like '%{}%'", phone));
    }
    let connection = sqlite::open(DB_PATH).unwrap();
    let mut statement = connection.prepare(query).unwrap();
    let mut result = json::JsonValue::new_array();
    while let Ok(State::Row) = statement.next() {
        let mut data = json::JsonValue::new_object();
        data["card"] = statement.read::<String, _>("card_id").unwrap().into();
        data["name"] = statement.read::<String, _>("name").unwrap().into();
        data["score"] = statement.read::<i64, _>("score").unwrap().into();
        data["last_change"] = statement.read::<i64, _>("last_change").unwrap().into();
        data["dress"] = statement.read::<String, _>("dress").unwrap().into();
        data["phone"] = statement.read::<i64, _>("phone").unwrap().into();
        data["remarks"] = statement.read::<String, _>("remarks").unwrap().into();
        result.push(data).unwrap();
    }
    return json::stringify(result);
}

#[tauri::command]
fn submit_user(is_new: &str, user_card: &str, user_name: &str, user_phone: &str, user_dress: &str, user_score: &str, user_remarks: &str) -> String {
    let mut result = json::JsonValue::new_object();
    if user_card == "" {
        result["code"] = 500.into();
        result["msg"] = "卡号不能为空".into();
        return json::stringify(result);
    }
    if user_name == "" {
        result["code"] = 500.into();
        result["msg"] = "用户名不能为空".into();
        return json::stringify(result);
    }
    let now: DateTime<Local> = Local::now();
    let mills: i64 = now.timestamp_millis();
    let query;
    if is_new == "true" {
        query = format!("INSERT INTO users (card_id, name, score, last_change, phone, dress, remarks, create_time, update_time) VALUES ('{}', '{}', {}, 0, '{}', '{}', '{}', {}, {});"
                            , user_card, user_name, user_score, user_phone, user_dress, user_remarks, mills, mills);
    } else {
        query = format!("update users set score={}, name='{}', phone='{}', dress='{}', remarks='{}', update_time={} where card_id='{}'"
                            , user_score, user_name, user_phone, user_dress, user_remarks, mills, user_card);
    }
    let connection = sqlite::open(DB_PATH).unwrap();
    let query_result = connection.execute(query);
    return match query_result {
        Result::Ok(_) => {
            result["code"] = 0.into();
            result["card"] = user_card.into();
            json::stringify(result)
        },
        Result::Err(error) => {
            result["code"] = 500.into();
            result["msg"] = format!("sql error code:{}, msg:{}", json::stringify(error.code), json::stringify(error.message)).into();
            json::stringify(result)
        }
    }
}

#[tauri::command]
fn next_card() -> String {
    let connection = sqlite::open(DB_PATH).unwrap();
    let query = "SELECT card_id FROM users WHERE card_id like '0%'";
    let mut statement = connection.prepare(query).unwrap();
    let mut max_card = 0;
    while let Ok(State::Row) = statement.next() {
        let card_id = statement.read::<String, _>("card_id").unwrap();
        let card:i32 = card_id.parse().unwrap();
        if max_card < card {
            max_card = card;
        }
    }
    let next_card = max_card + 1;
    let mut result = String::from("0");
    result.push_str(next_card.to_string().as_str());
    return result;
}

#[tauri::command]
fn search_like(filed: &str, param: &str) -> String {
    let connection = sqlite::open(DB_PATH).unwrap();
    let query = format!("SELECT distinct {} FROM users WHERE {} like '%{}%'", filed, filed, param);
    let mut statement = connection.prepare(query).unwrap();
    let mut result = json::JsonValue::new_array();
    while let Ok(State::Row) = statement.next() {
        let element = statement.read::<String, _>(filed).unwrap();
        result.push(element).unwrap();
    }
    return json::stringify(result);
}

fn main() {
    init();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, user_info, user_add_score, search_user, submit_user, next_card, search_like])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn init() {
    let connection = sqlite::open(DB_PATH).unwrap();
    let query = "CREATE TABLE IF NOT EXISTS \"users\"
                        (
                            card_id text not null
                                constraint users_pk
                                    unique,
                            name text not null,
                            score int default 0 not null,
                            last_change int default 0 not null,
                            phone text default '',
                            dress text default '',
                            remarks text default '',
                            create_time bigint not null,
                            update_time bigint not null
                        )";
    connection.execute(query).unwrap();
}