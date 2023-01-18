extern crate sqlite;

use std::fs;
use std::time::SystemTime;

use crate::my_http_client;
use crate::my_config;

static DB_DIR:&str = "./data";
pub static DB_FILE:&str = "./data/user.db";

pub fn init_db() {
    let query = "CREATE TABLE IF NOT EXISTS users
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
                        CREATE TABLE IF NOT EXISTS user_operate_log
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
                        );
                        CREATE TABLE IF NOT EXISTS config
                        (
                            key text
                                constraint config_pk
                                    primary key,
                            value text default '' not null
                        );";
    fs::create_dir_all(DB_DIR).or_else(|error| {
        log::error!("create db dir error:{}", error);
        return Err(error);
    }).unwrap();
    get_con().execute(query).or_else(|error| {
        log::error!("execute create db sql error:{}", error);
        return Err(error);
    }).unwrap();
    my_config::init_db_config();
    db_backup();
    log::info!("db init success!!!");
}

pub fn get_con() -> sqlite::Connection{
    return sqlite::open(DB_FILE).unwrap();
}

pub fn db_backup() {
    //获取修改时间
    let modifi_time = fs::metadata(DB_FILE)
    .and_then(|meta| meta.modified())
    .map(|time| Some(time))
    .unwrap_or_else(|error| {
        log::error!("get db file modified sys time error:{}", error);
        return None;
    });
    if modifi_time.is_none() {
        return;
    }
    let time_mills = modifi_time.unwrap().duration_since(SystemTime::UNIX_EPOCH)
    .map(|duration| duration.as_millis())
    .unwrap_or_else(|error| {
        log::error!("get db file modified unix time error:{}", error);
        return 0;
    });
    
    //如果需要备份的文件不存在，复制一份备份
    let db_backup_dir = my_config::get_db_backup_dir();
    let result = fs::create_dir_all(db_backup_dir);
    if result.is_err() {
        log::error!("create db backup dir error:{:?}", result.err());
        return;
    }
    let mut backup_path = String::from(db_backup_dir);
    let file_name = format!("/user_bak.{}.db", time_mills);
    backup_path.push_str(&file_name);
    let result = fs::File::open(&backup_path);
    match result{
        Ok(_) => {},
        Err(error) => {
            let error_info = format!("{:?}", error);
            log::info!("sync db file, because: {}", error_info);
            let result = fs::OpenOptions::new().create_new(true).write(true).open(&backup_path)
            .and_then(|_| fs::copy(DB_FILE, &backup_path));
            if result.is_err() {
                log::error!("copy db backup file error:{:?}", result.err());
                return;
            }
            //上传到百度网盘
            my_http_client::upload_file_to_bai_du(&backup_path, &file_name);
        }
    }
}