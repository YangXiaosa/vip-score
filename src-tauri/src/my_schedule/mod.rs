extern crate chrono;
extern crate cron;

use chrono::Local;
use cron::Schedule;
use std::{str::FromStr, time::Duration, thread};
use crate::my_db;

pub fn start_schedule() {
    //sec min hour dayOfMonth month dayOfWeek year
    let expression = "1 */10 * * * * *";
    let schedule = Schedule::from_str(expression).unwrap();
    for datetime in schedule.upcoming(Local) {
        let wait_time = (datetime.timestamp_millis() - Local::now().timestamp_millis()) as u64;
        log::info!("next sync db file time:{}, wait_time:{}", datetime, wait_time);
        thread::sleep(Duration::from_millis(wait_time));
        my_db::db_backup();
    }
}