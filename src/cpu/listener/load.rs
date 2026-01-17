use std::io::{Read, Seek, SeekFrom};
use std::fs::{File, OpenOptions};
use std::path::PathBuf;
use crate::logger::loghandle::LogHandle;
use std::sync::{Arc, Mutex};

#[derive(Debug)]
pub enum LoadLeve {
    Low,
    Mid,
    High,
    Ehight
}

pub struct LoadListenerHandle {
    log_handle: Arc<Mutex<LogHandle>>,
    file: File,
    file_path: PathBuf,
    total: i32,
    idle: i32,
}


impl LoadListenerHandle {
    pub fn new(log_handle: Arc<Mutex<LogHandle>>) -> Self {
        if let Ok(mut log) = log_handle.lock() {
            log.info(format!("初始化负载监听器"));
        }
        let result = OpenOptions::new()
            .read(true)
            .open("/proc/stat");
        let file = match result {
            Ok(f) => {
                if let Ok(mut log) = log_handle.lock() {
                    log.info(format!("成功打开/proc/stat文件"));
                    log.info(format!("负载监听器初始化完成"));
                }
                f
            }
            Err(e) => {
                if let Ok(mut log) = log_handle.lock() {
                    log.error(format!("无法打开/proc/stat,报错:{}",e));
                    log.error(format!("程序退出"));
                }
                panic!()
            }
        };
        Self {
            log_handle,
            file: file,
            file_path: PathBuf::from("/proc/stat"),
            total: 0,
            idle: 0,
        }
    }

    fn read_load(& mut self) -> i8 {
        let result = self.file.seek(SeekFrom::Start(0));
        match result {
            Ok(_) => {
                let mut buffer = String::new();
                let mut now_total = 0;
                let result = self.file.read_to_string(&mut buffer);
                match result {
                    Ok(_) => {
                        let lines = buffer.split("\n");
                        for line in lines {
                            if line.starts_with("cpu ") {
                                let parts = line.split_whitespace().collect::<Vec<&str>>();
                                if parts.len() > 9 {
                                    for num in &parts[1..10] {
                                        now_total += num.parse::<i32>().unwrap();
                                    }
                                    let now_idle = parts[3].parse::<i32>().unwrap() + parts[4].parse::<i32>().unwrap();

                                    //println!("now_total:{}",now_total);
                                    //println!("now_idle:{}",now_idle);
                                    
                                    let new_total = now_total - self.total;
                                    let new_idle = now_idle - self.idle;

                                    //println!("new_total:{}",new_total);
                                    //println!("new_idle:{}",new_idle);

                                    let kill = new_total - new_idle;

                                    //println!("kill:{}",kill);
                                    
                                    let load = kill as f64 / new_total as f64;

                                    self.total = now_total;
                                    self.idle = now_idle;

                                    return (load as f32 * 100.0) as i8;
                                }
                            }
                        }
                        50
                    }
                    Err(e) => {
                        if let Ok(mut log) = self.log_handle.lock() {
                            log.error(format!("无法读取/proc/stat文件:{}", e));
                            log.error(format!("尝试恢复"));
                        }
                        let result = OpenOptions::new()
                            .read(true)
                            .open(&self.file_path);
                        let file = match result {
                            Ok(f) => {
                                f
                            }
                            Err(e) => {
                                if let Ok(mut log) = self.log_handle.lock() {
                                    log.error(format!("无法恢复,退出程序,报错:{}", e));
                                }
                                panic!();
                            }
                        };
                        if let Ok(mut log) = self.log_handle.lock() {
                            log.info(format!("成功恢复/proc/stat文件"));
                        }
                        self.file = file;
                        self.read_load();
                        return 50;
                    }
                }
            }
            Err(e) => {
                if let Ok(mut log) = self.log_handle.lock() {
                    log.error(format!("无法跳转/proc/stat文件:{}", e));
                    log.error(format!("尝试恢复"));
                }
                let result = OpenOptions::new()
                    .read(true)
                    .open(&self.file_path);
                let file = match result {
                    Ok(f) => {
                        f
                    }
                    Err(e) => {
                        if let Ok(mut log) = self.log_handle.lock() {
                            log.error(format!("无法恢复,退出程序,报错:{}", e));
                        }
                        panic!();
                    }
                };
                if let Ok(mut log) = self.log_handle.lock() {
                    log.info(format!("成功恢复/proc/stat文件"));
                }
                self.file = file;
                self.read_load();
                return 50;
            }
        }
    }

    pub fn get_load_level(& mut self) -> LoadLeve {
        let load = self.read_load();
        println!("load:{}",load);
        if load < 20 {
            LoadLeve::Low
        } else if load < 40 {
            LoadLeve::Mid
        } else if load < 60 {
            LoadLeve::High
        } else {
            LoadLeve::Ehight
        }
    }
}


#[test]
fn test_read_load() {
    use std::thread::sleep;
    use std::time::Duration;
    let log_handle = Arc::new(Mutex::new(LogHandle::new("debug/log.log".to_string())));
    let mut load_listener_handle = LoadListenerHandle::new(log_handle);
    loop {
        sleep(Duration::from_millis(1000));
        let load_level = load_listener_handle.get_load_level();
        println!("load_level:{:?}",load_level);
    }
}
