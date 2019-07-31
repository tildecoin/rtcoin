//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::fs;
use std::fs::File;

use chrono::offset::Utc;
use simplelog::*;

pub const FILE: &str = "/tmp/rtcoinserver.log";

pub fn init() {
    // If the log file exists on startup,
    // timestamp it so we get a fresh log
    // file.
    if fs::metadata(FILE).is_ok() {
        let mut newpath = FILE.to_string();
        let time = Utc::now().to_string();
        newpath.push_str(".");
        newpath.push_str(&time);
        fs::rename(FILE, newpath).unwrap();
    }

    CombinedLogger::init(
        vec![
            TermLogger::new(
                LevelFilter::Warn,
                Config::default(),
                TerminalMode::Stderr,
            ).unwrap(),
            WriteLogger::new(
                LevelFilter::Info,
                Config::default(),
                File::create(FILE).unwrap(),
            ),
        ]
    ).expect("Unable to initialize logging");
}

#[cfg(test)]
mod test {
}