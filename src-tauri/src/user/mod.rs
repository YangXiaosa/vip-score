extern crate sqlite;
extern crate json;
extern crate chrono;

use sqlite::{State, Statement, Connection};
use chrono::{DateTime, Local};
use crate::my_db;

fn add_update_log(card: &str, name: &str, phone: &str, dress: &str, remarks: &str, operate_why: &str, mills: i64) -> bool{
    let users = get_user(card);
    if users.is_empty() {
        return false;
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
    if change == "" {
        return false;
    }
    let log_sql = format!("insert into user_operate_log(card_id, operate_type, operate_info, operate_time, operate_why, name) values('{}','{}','{}',{},'{}','{}');",
            card, "修改用户信息", change, mills, operate_why, users[0]["name"]);
    let connection = my_db::get_user_con();
    let query_result = connection.execute(&log_sql);
    match query_result {
        Result::Ok(_) => {}
        Result::Err(error) => {
            log::error!("sql error sql:{}, code:{}, msg:{}", log_sql, json::stringify(error.code), json::stringify(error.message));
        }
    }
    return true;
}

fn parse_to_user(statement: &sqlite::Statement) -> json::JsonValue {
    let mut error_info = String::from("");
    let mut data = json::JsonValue::new_object();
    data["card"] = get_filed_str(statement, "card_id", &mut error_info).into();
    data["name"] = get_filed_str(statement, "name", &mut error_info).into();
    data["score"] = get_filed_i64(statement, "score", &mut error_info).into();
    data["last_change"] = get_filed_i64(statement, "last_change", &mut error_info).into();
    data["dress"] = get_filed_str(statement, "dress", &mut error_info).into();
    data["phone"] = get_filed_str(statement, "phone", &mut error_info).into();
    data["create_time"] = get_filed_i64(statement, "create_time", &mut error_info).into();
    data["remarks"] = get_filed_str(statement, "remarks", &mut error_info).into();
    if error_info != "" {
        log::error!("parse_to_user have error, result:{:?}, e:{}", data, error_info);
    }
    return data;
}

fn get_user(card: &str) -> json::JsonValue{
    let query = String::from("SELECT * FROM users WHERE card_id = ?");
    return get_statement(&query, &my_db::get_user_con()).map_or(json::JsonValue::new_array(), |mut statement| {
        let result = statement.bind((1, card));
        if result.is_err() {
            log::error!("get user statement bind failed card_id:{}, error:{:?}", card, result.err());
            return json::JsonValue::new_array();
        }
        let mut result = json::JsonValue::new_array();
        while let Ok(State::Row) = statement.next() {
            result.push(parse_to_user(&statement)).unwrap();
        }
        return result;
    });
}

#[tauri::command]
pub fn user_info(card: &str) -> String {
    return json::stringify(get_user(card));
}

#[tauri::command]
pub fn user_add_score(card: &str, add_score: i32, operate_why: &str) -> String {
    let connection = my_db::get_user_con();
    let query = format!("update users set score = score + {}, last_change = {} where card_id = '{}'", add_score, add_score, card);
    let add_score_result = connection.execute(&query);
    let add_success = add_score_result.is_ok();
    if !add_success {
        log::error!("user_add_score failed sql:{}, error:{:?}", &query, add_score_result.err());
    }
    let users = get_user(card);
    if !users.is_empty() && add_success {
        let now: DateTime<Local> = Local::now();
        let mills: i64 = now.timestamp_millis();
        let log_sql = format!("insert into user_operate_log(card_id, operate_type, operate_info, operate_time, operate_why, name) values('{}','{}','{}',{},'{}','{}')",
                              card, "修改积分", format!("加{}分，总分:{}", add_score, users[0]["score"]), mills, operate_why, users[0]["name"]);
        connection.execute(&log_sql).unwrap_or_else(|error| log::error!("add score log failed sql:{}, error:{}", &log_sql, error));
    }
    return json::stringify(users);
}

#[tauri::command]
pub fn search_user(card: &str, name: &str, phone: &str,) -> String {
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
    return get_statement(&query, &my_db::get_user_con()).map_or(String::from("[]"), |mut statement| {
        let mut result = json::JsonValue::new_array();
        while let Ok(State::Row) = statement.next() {
            result.push(parse_to_user(&statement)).unwrap();
        }
        return json::stringify(result);
    });
}

#[tauri::command]
pub fn submit_user(is_new: &str, user_card: &str, user_name: &str, user_phone: &str, user_dress: &str, user_score: &str, user_remarks: &str, operate_why: &str) -> String {
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
        let parse_result = user_score.parse();
        if parse_result.is_err() {
            result["code"] = 500.into();
            result["msg"] = "积分必须为数字".into();
            return json::stringify(result);
        }
        score = parse_result.unwrap();
    }
    let now: DateTime<Local> = Local::now();
    let mills: i64 = now.timestamp_millis();
    let mut query = String::from("");
    if is_new == "true" {
        query = format!("INSERT INTO users (card_id, name, score, last_change, phone, dress, remarks, create_time, update_time) VALUES ('{}', '{}', {}, 0, '{}', '{}', '{}', {}, {});"
                        , user_card, user_name, score, user_phone, user_dress, user_remarks, mills, mills);
    } else {
        let have_change = add_update_log(user_card, user_name, user_phone, user_dress, user_remarks, operate_why, mills);
        if have_change {
            query = format!("update users set name='{}', phone='{}', dress='{}', remarks='{}', update_time={} where card_id='{}'"
                            , user_name, user_phone, user_dress, user_remarks, mills, user_card);
        }
    }
    if query == "" {
        result["code"] = 500.into();
        result["msg"] = "没有任何改动".into();
        return json::stringify(result);
    }
    log::info!("submit user sql : {}", query);
    let connection = my_db::get_user_con();
    let query_result = connection.execute(query);
    return match query_result {
        Result::Ok(_) => {
            log::info!("submit user success");
            result["code"] = 0.into();
            result["card"] = user_card.into();
            json::stringify(result)
        },
        Result::Err(error) => {
            let error_info = format!("{:?}", error);
            log::error!("submit user info failed, error:{}", error_info);
            result["code"] = 500.into();
            result["msg"] = "保存失败，请联系程序员".into();
            json::stringify(result)
        }
    }
}

#[tauri::command]
pub fn next_card() -> String {
    let query = String::from("SELECT card_id FROM users WHERE card_id like '0%'");
    return get_statement(&query, &my_db::get_user_con()).map_or_else(|| Local::now().timestamp().to_string(), |mut statement| {
        let mut max_card = 0;
        while let Ok(State::Row) = statement.next() {
            let result = statement.read::<String, _>("card_id");
            if result.is_err() {
                log::error!("next_card get card_id filed error:{:?}", result.err());
                continue;
            }
            let card_id = result.unwrap();
            let card = card_id.parse().unwrap_or_else(|error| { 
                log::error!("next_card card_id parse failed card_id:{}, error:{}", card_id, error); 
                return 0;
            });
            if max_card < card {
                max_card = card;
            }
        }
        let next_card = max_card + 1;
        let mut result = String::from("0");
        result.push_str(next_card.to_string().as_str());
        return result;
    });
}

#[tauri::command]
pub fn search_like(filed: &str, param: &str) -> String {
    let query = format!("SELECT distinct {} FROM users WHERE {} like '%{}%' limit 10", filed, filed, param);
    return get_statement(&query, &my_db::get_user_con()).map_or(String::from("[]"), |mut statement| {
        let mut result = json::JsonValue::new_array();
        while let Ok(State::Row) = statement.next() {
            let element = statement.read::<String, _>(filed).unwrap();
            result.push(element).unwrap();
        }
        return json::stringify(result);
    });
}

#[tauri::command]
pub fn search_log(log_card: &str, log_start: i64, log_end: i64, log_count: i64) -> String {
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
    query.push_str(format!(" order by operate_time desc limit {};", log_count).as_str());
    return get_statement(&query, &my_db::get_user_con()).map_or(String::from("[]"), |mut statement| {
        let mut result = json::JsonValue::new_array();
        let mut error_info = String::from("");
        while let Ok(State::Row) = statement.next() {
            let mut data = json::JsonValue::new_object();
            data["card"] = get_filed_str(&statement, "card_id", & mut error_info).into();
            data["name"] = get_filed_str(&statement, "name", & mut error_info).into();
            data["operate_time"] = get_filed_i64(&statement, "operate_time", & mut error_info).into();
            data["operate_type"] = get_filed_str(&statement, "operate_type", & mut error_info).into();
            data["operate_info"] = get_filed_str(&statement, "operate_info", & mut error_info).into();
            data["operate_why"] = get_filed_str(&statement, "operate_why", & mut error_info).into();
            result.push(data).unwrap();
        }
        if error_info != "" {
            log::error!("search_log prase db filed have error, result:{:?}, e:{}", result, error_info);
        }
        return json::stringify(result);
    });
}

fn get_statement<'a>(sql: &'a String, connection: &'a Connection) -> Option<Statement<'a>>{
    let result = connection.prepare(sql);
    if result.is_err() {
        log::error!("get statement failed slq:{}, error:{:?}", sql, result.err());
        return None;
    }
    return Some(result.unwrap());
}

fn get_filed_str(statement: &sqlite::Statement, filed_name: &str, error_info: & mut String) -> String{
    return statement.read::<String, _>(filed_name).unwrap_or_else(|error| {
        error_info.push_str(format!("{},", error).as_str());
        return String::from("");
    })
}

fn get_filed_i64(statement: &sqlite::Statement, filed_name: &str, error_info: & mut String) -> i64 {
    return statement.read::<i64, _>(filed_name).unwrap_or_else(|error| {
        error_info.push_str(format!("{},", error).as_str());
        return 0;
    })
}