//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use crate::logging::*;
use log::info;
use std::fs;

#[test]
fn check_init() {
    fs::write(FILE, b"Testing Rename of Old Log Files").unwrap();
    init();
    assert!(fs::metadata(FILE).is_ok());
        
    info!("test");
    let log_out = fs::read_to_string(FILE).unwrap();

    assert!(log_out.contains("test"));
    assert!(log_out.contains("INFO"));
}
