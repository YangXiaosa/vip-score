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

fn parse_to_user(statement: &sqlite::Statement) -> json::JsonValue {
    let mut data = json::JsonValue::new_object();
    data["card"] = statement.read::<String, _>("card_id").unwrap().into();
    data["name"] = statement.read::<String, _>("name").unwrap().into();
    data["score"] = statement.read::<i64, _>("score").unwrap().into();
    data["last_change"] = statement.read::<i64, _>("last_change").unwrap().into();
    data["dress"] = statement.read::<String, _>("dress").unwrap().into();
    data["phone"] = statement.read::<String, _>("phone").unwrap().into();
    data["create_time"] = statement.read::<i64, _>("create_time").unwrap().into();
    data["remarks"] = statement.read::<String, _>("remarks").unwrap().into();
    return data;
}

fn get_user(card: &str) -> json::JsonValue{
    let connection = sqlite::open(DB_PATH).unwrap();
    let query = "SELECT * FROM users WHERE card_id = ?";
    let mut statement = connection.prepare(query).unwrap();
    statement.bind((1, card)).unwrap();
    let mut result = json::JsonValue::new_array();
    while let Ok(State::Row) = statement.next() {
        result.push(parse_to_user(&statement)).unwrap();
    }
    return result;
}

#[tauri::command]
fn user_info(card: &str) -> String {
    return json::stringify(get_user(card));
}

#[tauri::command]
fn user_add_score(card: &str, add_score: i32, operate_why: &str) -> String {
    let connection = sqlite::open(DB_PATH).unwrap();
    let query = format!("update users set score = score + {}, last_change = {} where card_id = '{}'", add_score, add_score, card);
    connection.execute(query).unwrap();
    let users = get_user(card);
    if !users.is_empty() {
        let now: DateTime<Local> = Local::now();
        let mills: i64 = now.timestamp_millis();
        let log_sql = format!("insert into user_operate_log(card_id, operate_type, operate_info, operate_time, operate_why, name) values('{}','{}','{}',{},'{}','{}')",
                              card, "修改积分", format!("加{}分，总分:{}", add_score, users[0]["score"]), mills, operate_why, users[0]["name"]);
        connection.execute(log_sql).unwrap();
    }
    return json::stringify(users);
}

#[tauri::command]
fn search_user(card: &str, name: &str, phone: &str,) -> String {
    let mut query = String::from("SELECT * FROM users WHERE 1 = 1");
    if card != "" {
        query.push_str(&format!(" and card_id like '%{}%'", card));
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
        result.push(parse_to_user(&statement)).unwrap();
    }
    return json::stringify(result);
}

#[tauri::command]
fn submit_user(is_new: &str, user_card: &str, user_name: &str, user_phone: &str, user_dress: &str, user_score: &str, user_remarks: &str, operate_why: &str) -> String {
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
    let mut score = 0;
    if user_score != "" {
        score = user_score.parse().unwrap();
    }
    let now: DateTime<Local> = Local::now();
    let mills: i64 = now.timestamp_millis();
    let mut query = String::from("");
    if is_new == "true" {
        query = format!("INSERT INTO users (card_id, name, score, last_change, phone, dress, remarks, create_time, update_time) VALUES ('{}', '{}', {}, 0, '{}', '{}', '{}', {}, {});"
                            , user_card, user_name, score, user_phone, user_dress, user_remarks, mills, mills);
    } else {
        let have_change = add_update_log(user_card, user_name, user_phone, user_dress, score, user_remarks, operate_why, mills);
        if have_change {
            query = format!("update users set score={}, name='{}', phone='{}', dress='{}', remarks='{}', update_time={} where card_id='{}'"
                            , score, user_name, user_phone, user_dress, user_remarks, mills, user_card);
        }
    }
    if query == "" {
        result["code"] = 500.into();
        result["msg"] = "没有任何改动".into();
        return json::stringify(result);
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

fn add_update_log(card: &str, name: &str, phone: &str, dress: &str, score: i32, remarks: &str, operate_why: &str, mills: i64) -> bool{
    let users = get_user(card);
    if users.is_empty() {
        return false;
    }
    let mut log_sql = String::from("");
    if users[0]["score"] != score{
        log_sql.push_str(format!("insert into user_operate_log(card_id, operate_type, operate_info, operate_time, operate_why, name) values('{}','{}','{}',{},'{}','{}');",
                              card, "修改积分", format!("总分：{} -> {}", users[0]["score"], score), mills, operate_why, users[0]["name"]).as_str());

    }
    let mut change = String::from("");
    if users[0]["name"] != name {
        change.push_str(format!("姓名：{} -> {} ", users[0]["name"], name).as_str());
    }
    if users[0]["phone"] != phone {
        change.push_str(format!("电话：{} -> {} ", users[0]["phone"], phone).as_str());
    }
    if users[0]["dress"] != dress {
        change.push_str(format!("地址：{} -> {} ", users[0]["dress"], dress).as_str());
    }
    if users[0]["remarks"] != remarks {
        change.push_str(format!("备注：{} -> {} ", users[0]["remarks"], remarks).as_str());
    }
    if change != "" {
        log_sql.push_str(format!("insert into user_operate_log(card_id, operate_type, operate_info, operate_time, operate_why, name) values('{}','{}','{}',{},'{}','{}');",
                                 card, "修改用户信息", change, mills, operate_why, users[0]["name"]).as_str());
    }
    let have_change = log_sql != "";
    if have_change {
        let connection = sqlite::open(DB_PATH).unwrap();
        let query_result = connection.execute(log_sql);
        match query_result {
            Result::Ok(_) => {}
            Result::Err(error) => {
                println!("sql error code:{}, msg:{}", json::stringify(error.code), json::stringify(error.message));
            }
        }
    }
    return have_change;
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
    let query = format!("SELECT distinct {} FROM users WHERE {} like '%{}%' limit 10", filed, filed, param);
    let mut statement = connection.prepare(query).unwrap();
    let mut result = json::JsonValue::new_array();
    while let Ok(State::Row) = statement.next() {
        let element = statement.read::<String, _>(filed).unwrap();
        result.push(element).unwrap();
    }
    return json::stringify(result);
}

#[tauri::command]
fn search_log(log_card: &str, log_start: i64, log_end: i64) -> String {
    let connection = sqlite::open(DB_PATH).unwrap();
    let mut query = String::from("SELECT * FROM user_operate_log WHERE 1 = 1");
    if log_card != "" {
        query.push_str(format!(" and card_id = '{}'", log_card).as_str());
    }
    if log_start > 0 {
        query.push_str(format!(" and operate_time >= {}", log_start).as_str());
    }
    if log_end > 0 {
        query.push_str(format!(" and operate_time <= {}", log_end).as_str());
    }
    query.push_str(" order by operate_time desc;");
    let mut result = json::JsonValue::new_array();
    let mut statement = connection.prepare(query).unwrap();
    while let Ok(State::Row) = statement.next() {
        let mut data = json::JsonValue::new_object();
        data["card"] = statement.read::<String, _>("card_id").unwrap().into();
        data["name"] = statement.read::<String, _>("name").unwrap().into();
        data["operate_time"] = statement.read::<i64, _>("operate_time").unwrap().into();
        data["operate_type"] = statement.read::<String, _>("operate_type").unwrap().into();
        data["operate_info"] = statement.read::<String, _>("operate_info").unwrap().into();
        data["operate_why"] = statement.read::<String, _>("operate_why").unwrap().into();
        result.push(data).unwrap();
    }
    return json::stringify(result);
}

fn main() {
    init();
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![greet, user_info, user_add_score, search_user, submit_user, next_card, search_like, search_log])
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
                        );
                        CREATE TABLE IF NOT EXISTS \"user_operate_log\"
                        (
                            id integer not null
                                constraint user_operate_log_pk
                                    primary key autoincrement,
                            card_id text not null,
                            operate_type text not null,
                            operate_info text not null,
                            operate_time bigint not null,
                            name text default '' not null,
                            operate_why text default ''
                        )";
    connection.execute(query).unwrap();

}