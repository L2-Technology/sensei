// This file is Copyright its original authors, visible in version control
// history.
//
// This file is licensed under the Apache License, Version 2.0 <LICENSE-APACHE
// or http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your option.
// You may not use this file except in accordance with one or both of these
// licenses.

use bitcoin::Network;
use chrono::Utc;
use lightning::util::logger::{Logger, Record};
use lightning::util::ser::Writer;
use std::fs;

pub struct FilesystemLogger {
    data_dir: String,
    timestamp_format: String,
}
impl FilesystemLogger {
    pub fn new(data_dir: String, network: Network) -> Self {
        let logs_path = format!("{}/logs", data_dir);
        fs::create_dir_all(logs_path.clone()).unwrap();

        let timestamp_format = match network {
            Network::Bitcoin => String::from("%Y-%m-%d %H:%M:%S"),
            _ => String::from("%Y-%m-%d %H:%M:%S%.3f"),
        };

        Self {
            data_dir: logs_path,
            timestamp_format,
        }
    }
}
impl Logger for FilesystemLogger {
    fn log(&self, record: &Record) {
        let raw_log = record.args.to_string();
        let log = format!(
            "{} {:<5} [{}:{}] {}\n",
            // Note that a "real" lightning node almost certainly does *not* want subsecond
            // precision for message-receipt information as it makes log entries a target for
            // deanonymization attacks. For testing, however, its quite useful.
            Utc::now().format(&self.timestamp_format),
            record.level,
            record.module_path,
            record.line,
            raw_log
        );
        let logs_file_path = format!("{}/logs.txt", self.data_dir.clone());
        fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(logs_file_path)
            .unwrap()
            .write_all(log.as_bytes())
            .unwrap();
    }
}
