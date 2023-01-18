use std::fs;

static LOG_PATH:&str = "./log4rs.yaml";

pub fn init_log() {
    init_log_file();
    log4rs::init_file(LOG_PATH, Default::default()).unwrap();
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
        kind: delete
# 对全局 log 进行配置
root:
  level: info
  appenders:
    - stdout
    - log_file";
    let result = fs::File::open(LOG_PATH);
    match result{
        Ok(_) => {},
        Err(_) => {
            fs::OpenOptions::new().create_new(true).write(true).open(LOG_PATH)
            .and_then(|_| fs::write(LOG_PATH, log_config))
            .unwrap();
        }
    }
}