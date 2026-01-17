use sze_rs::logger::loghandle::LogHandle;
use std::{thread::{self, sleep}, time::Duration};
use sze_rs::cpu::listener::{load, temperature};
use std::sync::{Arc, Mutex};
use sze_rs::cpu::controller::freq::PolicyInfo;

fn main() {
    let log = Arc::new(Mutex::new(LogHandle::new("debug/log.log".to_string())));
    log.lock().unwrap().clear_file();

    let policy = PolicyInfo::new(Arc::clone(&log));

}