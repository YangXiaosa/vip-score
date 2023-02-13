extern crate json;
extern crate chrono;

use crate::my_db;
use crate::my_http_client;
use sqlite::State;
use once_cell::sync::Lazy;
use self::chrono::Local;

//数据库中config表里的key
static BAI_DU_WANG_PAN_APP_KEY:&str = "bai_du_wang_pan_app_key";
static BAI_DU_WANG_PAN_SECRET_KEY:&str = "bai_du_wang_pan_secret_key";
static BAI_DU_WANG_PAN_REFRESH_TOKEN:&str = "bai_du_wang_pan_refresh_token";
static BAI_DU_WANG_PAN_ACCESS_TOKEN:&str = "bai_du_wang_pan_access_token";
static BAI_DU_WANG_PAN_ACCESS_TOKEN_END_TIME:&str = "bai_du_wang_pan_access_token_end_time";
static DB_BACKUP_DIR:&str = "db_backup_dir";
static ADMIN_MAIL:&str = "admin_email";
static ADMIN_EMAIL_PASSWORD:&str = "admin_email_password";
static ADMIN_EMAIL_SERVER:&str = "admin_email_server";
static MERCHANT_NAME:&str = "merchant_name";


//百度网盘刷新ACCESS_TOKEN的url
static BAI_DU_WANG_PAN_REFRESH_TOKEN_URL:&str = "https://openapi.baidu.com/oauth/2.0/token?grant_type=refresh_token";
//百度网盘预上传文件的url
pub static BAI_DU_WANG_PAN_PRE_UPLOAD_FILE_URL:&str = "http://pan.baidu.com/rest/2.0/xpan/file?method=precreate";
//百度网盘上传文件分片的url
pub static BAI_DU_WANG_PAN_UPLOAD_SHARD_URL:&str = "https://d.pcs.baidu.com/rest/2.0/pcs/superfile2";
//百度网盘文件分片合成url
pub static BAI_DU_WANG_PAN_CREATE_FILE_URL:&str = "https://pan.baidu.com/rest/2.0/xpan/file?method=create";

//数据库中的配置缓存
static mut DB_CONFIG:Lazy<json::JsonValue> = Lazy::new(|| { return json::JsonValue::new_object(); });

pub fn init_db_config() {
    update_config();
    let config_info = unsafe {format!("config is :{:?}", DB_CONFIG)};
    log::info!("config is:{}",config_info);
}

fn update_config() {
    let con = my_db::get_user_con();
    let mut statement = con.prepare("select * from config").or_else(|error| {
        log::error!("select * from config error:{}", error);
        return Err(error);
    }).unwrap();
    while let Ok(State::Row) = statement.next() {
        let key = statement.read::<String, _>("key").unwrap();
        let value = statement.read::<String, _>("value").unwrap().into();
        unsafe { DB_CONFIG[key] = value; };
    }
}

pub fn get_bai_du_wang_pan_access_token() -> &'static str{
    let access_token = get_config_str(BAI_DU_WANG_PAN_ACCESS_TOKEN);
    let token_end_time = get_config_i64(BAI_DU_WANG_PAN_ACCESS_TOKEN_END_TIME);
    //配置中包含token和token的结束时间，且还未到结束时间，返回token，否则刷新token
    if access_token != "" && token_end_time > 0 {
        let now_second = Local::now().timestamp();
        if now_second <= token_end_time - 600 {
            return access_token;
        }
    }
    log::info!("start refresh token");
    //刷新token
    return refresh_access_token();
}

fn refresh_access_token() -> &'static str{
    let refresh_token = get_config_str(BAI_DU_WANG_PAN_REFRESH_TOKEN);
    let app_key = get_config_str(BAI_DU_WANG_PAN_APP_KEY);
    let secret_key = get_config_str(BAI_DU_WANG_PAN_SECRET_KEY);
    if refresh_token == "" || app_key == "" || secret_key == "" {
        return "";
    }
    let mut url = String::from(BAI_DU_WANG_PAN_REFRESH_TOKEN_URL);
    url.push_str(format!("&refresh_token={}&client_id={}&client_secret={}", refresh_token, app_key, secret_key).as_str());
    let result = my_http_client::url_get(url);
    if !result.has_key("refresh_token") {
        return "";
    }
    let now_second = Local::now().timestamp();
    let expires_in = result["expires_in"].as_i64().unwrap_or(0);
    let access_token_end_time = now_second + expires_in;
    let refresh_token = result["refresh_token"].as_str().unwrap_or("");
    let access_token = result["access_token"].as_str().unwrap_or("");
    let con = my_db::get_user_con();
    let sql = format!("replace into config(key, value) values('{}','{}'),('{}','{}'),('{}','{}')",
                      BAI_DU_WANG_PAN_ACCESS_TOKEN_END_TIME, access_token_end_time,
                      BAI_DU_WANG_PAN_REFRESH_TOKEN, refresh_token,
                      BAI_DU_WANG_PAN_ACCESS_TOKEN, access_token);
    match con.execute(sql) {
        Ok(_) => {
            set_config_i64(BAI_DU_WANG_PAN_ACCESS_TOKEN_END_TIME, access_token_end_time);
            set_config_str(BAI_DU_WANG_PAN_REFRESH_TOKEN, refresh_token);
            set_config_str(BAI_DU_WANG_PAN_ACCESS_TOKEN, access_token);
            return get_config_str(BAI_DU_WANG_PAN_ACCESS_TOKEN);
        }
        Err(e) => {
            log::error!("insert token config error :{:?}", e);
            return "";
        }
    }
}

//获取库文件备份目录
pub fn get_db_backup_dir() -> &'static str{
    let db_backup_dir = get_config_str(DB_BACKUP_DIR);
    if db_backup_dir != "" {
        return db_backup_dir;
    }
    return "./backup/BaiduSyncdisk/supermarket";
}
//获取管理员邮箱
pub fn get_admin_email() -> &'static str{
    return get_config_str(ADMIN_MAIL);
}
//获取管理员邮箱的授权密码
pub fn get_admin_email_password() -> &'static str{
    return get_config_str(ADMIN_EMAIL_PASSWORD);
}
//获取管理员邮箱的服务器
pub fn get_admin_email_server() -> &'static str{
    return get_config_str(ADMIN_EMAIL_SERVER);
}
//获取商户名称
pub fn get_merchant_name() -> &'static str{
    return get_config_str(MERCHANT_NAME);
}

fn get_config_str(key: &str) -> &str {
    return unsafe { DB_CONFIG[key].as_str().unwrap_or("") };
}

fn get_config_i64(key: &str) -> i64 {
    return unsafe { DB_CONFIG[key].as_str()
        .map_or(Ok(0),|val| val.parse())
        .unwrap_or_else(|error| {
            log::error!("get_config_i64 error :{:?}", error);
            return 0;
        }) };
}

fn set_config_str(key: &str, value: &str) {
    unsafe { DB_CONFIG[key]= value.into() };
}

fn set_config_i64(key: &str, value: i64) {
    unsafe { DB_CONFIG[key]= value.into() };
}

#[tauri::command]
pub fn search_setting() -> String {
    let mut result = json::JsonValue::new_object();
    result["baiDuWangPanAppKey"] = get_config_str(BAI_DU_WANG_PAN_APP_KEY).into();
    result["baiDuWangPanSecretKey"] = get_config_str(BAI_DU_WANG_PAN_SECRET_KEY).into();
    result["baiDuWangPanRefreshToken"] = get_config_str(BAI_DU_WANG_PAN_REFRESH_TOKEN).into();
    result["dbBackupDir"] = get_db_backup_dir().into();
    result["adminEmail"] = get_config_str(ADMIN_MAIL).into();
    result["adminEmailPassword"] = get_config_str(ADMIN_EMAIL_PASSWORD).into();
    result["adminEmailServer"] = get_config_str(ADMIN_EMAIL_SERVER).into();
    result["merchantName"] = get_config_str(MERCHANT_NAME).into();
    return json::stringify(result);
}

#[tauri::command]
pub fn save_setting(bai_du_wang_pan_app_key: &str, bai_du_wang_pan_secret_key: &str, 
    bai_du_wang_pan_refresh_token: &str, db_backup_dir: &str, admin_email: &str, 
    admin_email_password: &str, admin_email_server: &str, merchant_name: &str) -> String {
    let mut result = json::JsonValue::new_object();
    let con = my_db::get_user_con();
    let sql = format!("replace into config(key, value) values('{}','{}'),('{}','{}'),('{}','{}'),('{}','{}'),('{}','{}'),('{}','{}'),('{}','{}'),('{}','{}');"
    , BAI_DU_WANG_PAN_APP_KEY, bai_du_wang_pan_app_key, BAI_DU_WANG_PAN_SECRET_KEY, bai_du_wang_pan_secret_key
    , BAI_DU_WANG_PAN_REFRESH_TOKEN, bai_du_wang_pan_refresh_token, DB_BACKUP_DIR, db_backup_dir
    , ADMIN_MAIL, admin_email, ADMIN_EMAIL_PASSWORD, admin_email_password
    , ADMIN_EMAIL_SERVER, admin_email_server, MERCHANT_NAME, merchant_name);
    let sql_result = con.execute(&sql);
    sql_result.unwrap_or_else(|error| log::error!("save setting error, code:{}, msg:{}, sql:{}", json::stringify(error.code), json::stringify(error.message), sql));
    update_config();
    result["code"] = 0.into();
    return json::stringify(result);
}