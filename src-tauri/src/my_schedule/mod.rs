extern crate chrono;
extern crate cron;

use chrono::Local;
use cron::Schedule;
use std::{str::FromStr, time::Duration, thread};
use crate::my_db;
use crate::my_log;

pub fn start_schedule() {
    thread::spawn(db_schedule);
    thread::spawn(email_schedule);
}

fn db_schedule() {
    //sec min hour dayOfMonth month dayOfWeek year
    let expression = "0 */10 * * * * *";
    let schedule = Schedule::from_str(expression).unwrap();
    for datetime in schedule.upcoming(Local) {
        let wait_time = (datetime.timestamp_millis() - Local::now().timestamp_millis()) as u64;
        log::info!("next sync db file time:{}, wait_time:{}", datetime, wait_time);
        thread::sleep(Duration::from_millis(wait_time));
        my_db::db_backup();
    }
}

fn email_schedule() {
    //sec min hour dayOfMonth month dayOfWeek year
    let expression = "0 */10 * * * * *";
    let schedule = Schedule::from_str(expression).unwrap();
    for datetime in schedule.upcoming(Local) {
        let wait_time = (datetime.timestamp_millis() - Local::now().timestamp_millis()) as u64;
        log::info!("next sync db file time:{}, wait_time:{}", datetime, wait_time);
        thread::sleep(Duration::from_millis(wait_time));
        my_log::error_send_mail();
    }
}