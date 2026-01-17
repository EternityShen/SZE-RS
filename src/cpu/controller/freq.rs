use std::{collections::HashMap, fs::OpenOptions, io::{Error, Read, Write}, path::PathBuf, sync::{Arc, Mutex}};

use crate::logger::loghandle::{LogHandle};


struct FreqInfo {
    policy_path: PathBuf,
    max_freq_file: PathBuf,
    min_freq_file: PathBuf,
    freq_list: Vec<usize>
}

pub struct PolicyInfo {
    log_handle: Arc<Mutex<LogHandle>>,
    policy_map: HashMap<i8, FreqInfo>
}

impl FreqInfo {
    fn new(policy_path: & PathBuf, log_handle: &Arc<Mutex<LogHandle>>) -> Self {
        let max_freq_file = policy_path.join("scaling_max_freq");
        let min_freq_file = policy_path.join("scaling_min_freq");
        let freq_list_file = policy_path.join("scaling_available_frequencies");
        let mut freq_list = vec![];

        let result =OpenOptions::new()
                                        .read(true)
                                        .open(freq_list_file);
        match result {
            Ok(mut f) => {
                let mut buffer = String::new();
                let result = f.read_to_string(&mut buffer);
                match result {
                    Ok(_) => {
                        let freq_list_str = buffer.split_whitespace().collect::<Vec<&str>>();
                        for freq in freq_list_str {
                            freq_list.push(freq.parse::<usize>().unwrap());
                        } 
                    }
                    Err(e) => {
                        if let Ok(mut log) = log_handle.lock() {
                            log.error(format!("无法读取频率表:{}", e));
                        }
                    }
                }
            }

            Err(e) => {
                if let Ok(mut log) = log_handle.lock() {
                    log.error(format!("无法打开频率表文件:{}", e));
                }
            }
        }
        Self { 
            policy_path: policy_path.clone(),
            max_freq_file: max_freq_file, 
            min_freq_file: min_freq_file, 
            freq_list: freq_list 
        }
    }

    fn set_max_freq(&self, freq: usize) -> Result<(), Error>{
        let result = OpenOptions::new()
                                        .write(true)
                                        .open(&self.max_freq_file);
        match result {
            Ok(mut f) => {
                f.write_all(format!("{}", freq).as_bytes())?;
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }

    fn set_min_freq(&self, freq: usize) -> Result<(), Error>{
        let result = OpenOptions::new()
                                        .write(true)
                                        .open(&self.min_freq_file);
        match result {
            Ok(mut f) => {
                f.write_all(format!("{}", freq).as_bytes())?;
                Ok(())
            }
            Err(e) => {
                Err(e)
            }
        }
    }
}

impl PolicyInfo {
    pub fn new(log_handle: Arc<Mutex<LogHandle>>) -> Self{
        let mut policy_map: HashMap<i8, FreqInfo> = HashMap::new();
        let cpufreq_path = PathBuf::from("/sys/devices/system/cpu/cpufreq/");
        let result = cpufreq_path.read_dir();
        match result {
            Ok(all_dir) => {
                for dir_result in all_dir {
                    match dir_result {
                        Ok(dir) => {
                            let freq_info = FreqInfo::new(& dir.path(), &log_handle);
                            policy_map.insert(1, freq_info);
                        }
                        Err(e) => {
                            if let Ok(mut log) = log_handle.lock() {
                                log.error(format!("无法获取policy文件夹:{}", e));
                                log.warn("程序退出".to_string());
                                panic!()
                            }
                        }
                    };
                }
                return Self { 
                    log_handle,
                    policy_map
                };
            }

            Err(e) => {
                if let Ok(mut log) = log_handle.lock() {
                    log.error(format!("无法读取/sys/devices/system/cpu/cpufreq/:{}", e));
                }
                panic!()
            }
            
        }
    }
}
