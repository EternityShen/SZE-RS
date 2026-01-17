use std::{fs::{File, OpenOptions, read_to_string}, io::Read};
use crate::logger::loghandle::{LogHandle};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::io::{Seek,SeekFrom};

pub struct TemperatureListenerHandle {
    log_handle: Arc<Mutex<LogHandle>>,
    file: File,
    file_path: PathBuf,
}

#[derive(Debug)]
pub enum TemperatureLevel {
    High,
    Low,
    Mid
}

impl TemperatureListenerHandle {
    pub fn new(log_handle: Arc<Mutex<LogHandle>>) -> Self {
        if let Ok(mut log) = log_handle.lock() {
            log.info(format!("初始化温度监听器"));
        }
        let temperature_path = PathBuf::from("/sys/class/thermal");
        let result = temperature_path.read_dir();
        match result {
            Ok(all_dir) => {
                if let Ok(mut log) = log_handle.lock() {
                    log.info(format!("成功读取/sys/class/thermal目录"));
                }
                for dir in all_dir {
                    match dir {
                        Ok(en) => {
                            let path = en.path();
                            if path.is_dir() {
                                if path.display().to_string().contains("thermal_zone") {
                                    let type_path = path.join("type");
                                    let result= read_to_string(type_path);
                                    match result {
                                        Ok(str) => {
                                            if let Ok(mut log) = log_handle.lock() {
                                                log.info(format!("成功读取type文件:{}",str));
                                            }
                                            if str.contains("mtktscpu")
                                            || str.contains("soc_max")
                                            || str.contains("cpu-1-") {
                                             let temperature_path = path.join("temp");
                                                let result = OpenOptions::new()
                                                                            .read(true)
                                                                            .open(&temperature_path);
                                                match result {
                                                    Ok(f) => {
                                                        if let Ok(mut log) = log_handle.lock() {
                                                            log.info(format!("成功打开temp文件"));
                                                            log.info(format!("成功读取temp文件:{}",str));
                                                            log.info(format!("温度监听器初始化完成"));
                                                        }
                                                        return Self { 
                                                            log_handle: log_handle,
                                                            file: f, 
                                                            file_path: temperature_path
                                                        };
                                                    }
                                                    Err(e) => {
                                                        if let Ok(mut log) = log_handle.lock() {
                                                            log.error(format!("无法打开temp文件:{}",e));
                                                            log.error(format!("程序退出"));
                                                        }
                                                        panic!()
                                                    }
                                                };
                                            }
                                        }
                                        Err(e) => {
                                            if let Ok(mut log) = log_handle.lock() {
                                                log.error(format!("无法读取type文件:{}",e));
                                                log.error(format!("程序退出"));
                                            }
                                            panic!()
                                        }
                                    }
                                }
                            }
                        }
                        Err(e) => {
                            if let Ok(mut log) = log_handle.lock() {
                                log.error(format!("无法读取目录:{}",e));
                                log.error(format!("程序退出"));
                            }
                            panic!()
                        }
                    };
                }
                panic!()
            }

            Err(e) => {
                if let Ok(mut log) = log_handle.lock() {
                    log.error(format!("无法读取/sys/class/thermal目录:{}",e));
                }
                panic!()
            }
        }
    }

    pub fn read_temperature(& mut self) -> i8 {
        let result = self.file.seek(SeekFrom::Start(0));
        match result {
            Ok(_) => {
                let mut buffer = String::new();
                let result = self.file.read_to_string(&mut buffer);
                match result {
                    Ok(_) => {
                        let temp = buffer.trim().parse::<i32>().unwrap();
                        (temp / 1000) as i8

                    }
                    Err(e) => {
                        if let Ok(mut log) = self.log_handle.lock() {
                            log.error(format!("无法读取temp文件:{}",e));
                            log.warn(format!("尝试重新打开temp文件"));
                        }
                        let result = OpenOptions::new()
                                                                    .read(true)
                                                                    .open(&self.file_path);
                        match result {
                            Ok(f) => {
                                if let Ok(mut log) = self.log_handle.lock() {
                                    log.info(format!("成功打开temp文件"));
                                    log.warn(format!("temp文件已重新打开"));
                                }
                                self.file = f;
                                return self.read_temperature();
                            }
                            Err(e) => {
                                if let Ok(mut log) = self.log_handle.lock() {
                                    log.error(format!("无法打开temp文件:{}",e));
                                    log.error(format!("程序退出"));
                                }
                                panic!()
                            }
                        }
                    }
                }
            }

            Err(e) => {
                if let Ok(mut log) = self.log_handle.lock() {
                    log.error(format!("无法读取temp文件:{}",e));
                    log.error(format!("程序退出"));
                }
                panic!()
            }
        }
    }

    pub fn get_temperature_level(& mut self) -> TemperatureLevel {
        let temperature = self.read_temperature();
        println!("temperature:{}",temperature);
        if temperature < 40 {
            TemperatureLevel::Low
        } else if temperature < 60 {
            TemperatureLevel::Mid
        } else {
            TemperatureLevel::High
        }
    }

}