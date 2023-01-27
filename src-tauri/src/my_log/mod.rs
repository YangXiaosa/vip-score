
use std::fs;
use crate::my_email;
use std::io::{self, BufRead};
use crate::my_db;
use sqlite::{State};

static LOG_YAML_PATH:&str = "./log4rs.yaml";

static ERROR_LOG_PATH:&str = "./log/error.log";

pub fn init_log() {
    init_log_file();
    log4rs::init_file(LOG_YAML_PATH, Default::default()).unwrap();
    log::info!("log4rs init success !!!");
}

fn init_log_file() {
    let log_config =
        "# appender 负责将日志收集到控制台或文件, 可配置多个
appenders:
  stdout:
    kind: console
  log_file:
    kind: rolling_file
    path: log/log.log
    append: true
    encoder:
      kind: pattern
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 10 mb
      roller:
        kind: fixed_window
        pattern: log/old_log/log.{}.log
        count: 100
        base: 1
  error_file:
    kind: rolling_file
    path: log/error.log
    filters:
    - kind: threshold
      level: error
    encoder:
      kind: pattern
    policy:
      kind: compound
      trigger:
        kind: size
        limit: 1 mb
      roller:
        kind: fixed_window
        pattern: log/old_error/error.{}.log
        count: 300
        base: 1
# 对全局 log 进行配置
root:
  level: info
  appenders:
    - stdout
    - log_file
    - error_file";
    let result = fs::File::open(LOG_YAML_PATH);
    match result{
        Ok(_) => {},
        Err(_) => {
            fs::OpenOptions::new().create_new(true).write(true).open(LOG_YAML_PATH)
            .and_then(|_| fs::write(LOG_YAML_PATH, log_config))
            .unwrap();
        }
    }
}

pub fn error_send_mail() {
    //校验一下错误日志是否有改动
    let con = my_db::get_no_bak_con();
    let result = con.prepare("select `value` from public_record where `key`='last_log_time'");
    if result.is_err() {
      log::error!("select last_log_time failed, error:{:?}", &result.err());
      return;
    }
    let mut statement = result.unwrap();
    let mut last_log_time = String::from("");
    while let Ok(State::Row) = statement.next() {
      last_log_time  = statement.read::<String, _>("value").unwrap_or(last_log_time);
    }
    let open_result = fs::read(ERROR_LOG_PATH);
    let mut error_info = String::from("");
    match open_result {
        Ok(bytes) => {
          let mut cursor = io::Cursor::new(bytes);
          let mut buf = String::new();
          let mut num_bytes = 0;
          let mut log_time = String::from("");
          while true{
            let read_result = cursor.read_line(&mut buf);
            if read_result.is_err() {
                log::error!("read error log file failed, num_bytes:{} error:{:?}", num_bytes, &read_result.err());
                return;
            }
            num_bytes = read_result.unwrap();
            if num_bytes > 0 {
              log_time = buf.split_once(" ").map(|tuple| String::from(tuple.0)).unwrap_or(String::from(""));
              if log_time == last_log_time {
                error_info.clear();
                continue;
              }
              error_info.push_str(buf.as_str());
              buf.clear();
            } else {
              break;
            }
          }
          if error_info != "" {
            my_email::send_mail("积分系统-监控", &error_info);
            let sql = format!("replace into public_record(key, value) values('{}', '{}')", "last_log_time", log_time);
            let query_result = con.execute(&sql);
            match query_result {
                Result::Ok(_) => {}
                Result::Err(error) => {
                    println!("sql error sql:{}, code:{}, msg:{}", sql, json::stringify(error.code), json::stringify(error.message));
                }
            }
          }
        }
        Err(error) => { log::error!("open error log file failed, e:{}", error) }
    }
}