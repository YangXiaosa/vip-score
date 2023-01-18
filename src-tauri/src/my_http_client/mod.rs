extern crate json;

use curl::easy::{Easy2, Handler, WriteError, Form};
use crate::my_config;
use std::fs;
use std::path::Path;

struct Collector(Vec<u8>);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.extend_from_slice(data);
        Ok(data.len())
    }
}

pub fn url_get(url: String) -> json::JsonValue{
    let mut easy = Easy2::new(Collector(Vec::new()));
    let result = easy.get(true)
    .and_then(|_| easy.url(url.as_str()))
    .and_then(|_| easy.perform())
    .and_then(|_| easy.response_code());
    if result.is_err() {
        log::error!("url get error url:{}, e:{:?}", url, result.err());
        return json::JsonValue::new_object();
    }
    let code = result.unwrap();
    let contents = easy.get_ref();
    let content = String::from_utf8_lossy(&contents.0);
    if code != 200 {
        log::error!("url get failed, url:{}, code:{}, data:{:?}", url, code, content);
        return json::JsonValue::new_object();
    }
    log::info!("url get success, url:{}, code:{}, data:{:?}", url, code, content);
    return json::parse(&content).unwrap_or_else(|error| {
        log::error!("url get parse content error:{:?}", error);
        return json::JsonValue::new_object();
    });
}

pub fn url_post(url: String, param: &str) -> json::JsonValue{
    let param_data = param.as_bytes();
    let mut easy = Easy2::new(Collector(Vec::new()));
    let result = easy.post(true)
    .and_then(|_| easy.url(url.as_str()))
    .and_then(|_| easy.post_fields_copy(param_data))
    .and_then(|_| easy.perform())
    .and_then(|_| easy.response_code());
    if result.is_err() {
        log::error!("url post error url:{}, param:{}, e:{:?}", url, param, result.err());
        return json::JsonValue::new_object();
    }
    let code = result.unwrap();
    let contents = easy.get_ref();
    let content = String::from_utf8_lossy(&contents.0);
    if code != 200 {
        log::error!("url post failed, url:{}, param:{}, code:{}, data:{}", url, param, code, content);
        return json::JsonValue::new_object();
    }
    log::info!("url post success, url:{}, param:{} code:{}, data:{}", url, param, code, content);
    return json::parse(&content).unwrap_or_else(|error| {
        log::error!("url post parse content error:{:?}", error);
        return json::JsonValue::new_object();
    });
}

pub fn url_post_upload(url: String, form: Form) -> json::JsonValue{
    let mut easy = Easy2::new(Collector(Vec::new()));
    let result = easy.post(true)
    .and_then(|_| easy.url(url.as_str()))
    .and_then(|_| easy.httppost(form))
    .and_then(|_| easy.perform())
    .and_then(|_| easy.response_code());
    if result.is_err() {
        log::error!("url post upload error:{:?}", result.err());
        return json::JsonValue::new_object();
    }
    let code = result.unwrap();
    let contents = easy.get_ref();
    let content = String::from_utf8_lossy(&contents.0);
    if code != 200 {
        log::error!("url post upload failed, url:{}, code:{}, data:{}", url, code, content);
        return json::JsonValue::new_object();
    }
    log::info!("url post upload success, url:{}, code:{}, data:{}", url, code, content);
    return json::parse(&content).unwrap_or_else(|error| {
        log::error!("url post upload parse content error:{:?}", error);
        return json::JsonValue::new_object();
    });
}

/** 
 * 上传文件到百度网盘
 *  @param file_path 本地文件路径
 *  @param file_name 文件名
*/
pub fn upload_file_to_bai_du(file_path: &str, file_name: &str) {
    let access_token = my_config::get_bai_du_wang_pan_access_token();
    if access_token == "" {
        log::error!("access_token is empty, file_path:{}", file_path);
        return;
    }
    //预上传
    let mut pre_upload_url = String::from(my_config::BAI_DU_WANG_PAN_PRE_UPLOAD_FILE_URL);
    pre_upload_url.push_str(format!("&access_token={}", access_token).as_str());
    let result = fs::read(file_path);
    if result.is_err() {
        log::error!("upload to baidu read file:{} error:{:?}", file_path, result.err());
        return;
    }
    let file_data = result.unwrap();
    let file_len = file_data.len();
    let file_md5 = md5::compute(file_data);
    let md5_str = format!("{:x}", file_md5);
    let param = format!("path=/apps/积分系统/{}&size={}&isdir=0&autoinit=1&rtype=3&block_list=[\"{}\"]", file_name, file_len, md5_str);
    let pre_result = url_post(pre_upload_url, param.as_str());
    //上传分片
    let uploadid = pre_result["uploadid"].as_str().unwrap_or("");
    let remote_path = pre_result["path"].as_str().unwrap_or("");
    if uploadid == "" || remote_path == "" {
        log::error!("upload to baidu pre result invalid");
        return;
    }
    let mut upload_url = String::from(my_config::BAI_DU_WANG_PAN_UPLOAD_SHARD_URL);
    upload_url.push_str(&format!("?access_token={}&method=upload&type=tmpfile&path={}&uploadid={}&partseq=0", access_token, remote_path, uploadid));
    let mut form = Form::new();
    let mut part = form.part("file");
    let form_path = Path::new(file_path);
    let result = part.file(form_path).filename(file_name).add();
    if result.is_err() {
        log::error!("upload to baidu create shard form error file:{}, file_name:{}, e:{:?}", file_path, file_name, result.err());
        return;
    }
    url_post_upload(upload_url, form);
    //分片合成文件
    let mut create_file_url = String::from(my_config::BAI_DU_WANG_PAN_CREATE_FILE_URL);
    create_file_url.push_str(&format!("&access_token={}", access_token));
    let param = format!("path={}&size={}&isdir=0&rtype=3&uploadid={}&block_list=[\"{}\"]", remote_path, file_len, uploadid, md5_str);
    url_post(create_file_url, param.as_str());
}