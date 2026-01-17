use std::io::Write;
use std::{fs::File, path::PathBuf};
use std::fs::{OpenOptions};
use chrono::Local;
use std::sync::{Arc, Mutex};


enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

pub struct LogHandle {
    file: File,
    file_path: PathBuf,
    lock: Arc<Mutex<()>>,
}

fn get_log_level(level: LogLevel) -> String {
    match level {
        LogLevel::Debug => {
            return "DEBUG".to_string();
        }
        LogLevel::Error => {
            return "ERROR".to_string();
        }
        LogLevel::Info => {
            return "INFO".to_string();
        }
        LogLevel::Warn => {
            return "WARN".to_string();
        }
    }
}

fn get_log_time() -> String {
    let now = Local::now();
    let time = now.format("%Y年%m月%d日 %H:%M:%S");
    time.to_string()
}

fn write_log(log: &mut LogHandle, mess: String, level: LogLevel) {
    let result = log.lock.lock();
    match result {
        Ok(_) => {
            let time = get_log_time();
            let level = get_log_level(level);
            let message = format!("{}:{}:{}\n",time, level, mess);
            let result = log.file.write_all(message.as_bytes());
            match result {
                Ok(_) => {

                }
                Err(e1) => {
                    eprintln!("日志部分出现问题");
                    eprintln!("原始报错:{}",e1);
                    let result = OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(&log.file_path);
                    
                    match result {
                        Ok(mut f) => {
                            let result = f.write_all(message.as_bytes());
                            match result {
                                Ok(_) => {
                                    log.file = f;
                                }
                                Err(e3) => {
                                    eprintln!("尝试恢复后仍然无法写入,报错:{}",e3);
                                }
                            }
                        }
                        Err(e2) => {
                            eprintln!("尝试恢复的报错:{}",e2);
                            panic!()
                        }
                    };
                }
            }
        }
        Err(e) => {
            eprintln!("无法获取日志锁,报错:{}",e);
            panic!()
        }
    };

   
}

impl LogHandle {
    pub fn new(file_path: String) -> Self {
        let result = OpenOptions::new()
            .append(true)
            .create(true)
            .open(&file_path);
        let file = match result {
            Ok(f) => {
                f
            }
            Err(e) => {
                eprintln!("{}",e);
                panic!()
            }   
        };
        Self {
            lock: Arc::new(Mutex::new(())),
            file,
            file_path: file_path.into(),
        }
    }


    pub fn get_file_path(&self) -> &PathBuf {
        &self.file_path
    }

    pub fn clear_file(&mut self) {
        let result = OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.file_path);
        
        match result {
            Ok(f) => {
                self.file = f;
            }
            Err(e) => {
                eprintln!("无法清空日志文件: {}", e);
            }
        }
    }

    pub fn clone(&self) -> Self {
        let file = match self.file.try_clone() {
            Ok(f) => {
                f
            }
            Err(e) => {
                eprintln!("无法克隆文件句柄:{}",e);
                panic!()
            }
        };
        Self {
            lock: self.lock.clone(),
            file,
            file_path: self.file_path.clone(),
        }
    }

    pub fn info(&mut self, mess: String) {
        write_log(self, mess, LogLevel::Info);
    }

    pub fn warn(&mut self, mess: String) {
        write_log(self, mess, LogLevel::Warn);
    }

    pub fn error(&mut self, mess: String) {
        write_log(self, mess, LogLevel::Error);
    }

    pub fn debug(&mut self, mess: String) {
        write_log(self, mess, LogLevel::Debug);
    }
}