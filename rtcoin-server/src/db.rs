//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::path::Path;

use sqlite;

mod crypt;

struct DB {
    conn: sqlite::Connection,
    key: Vec<u8>,
}

impl DB {
    fn connect() -> &'static DB {
        let db_flags = sqlite::OpenFlags::new();
        db_flags.set_create();     // Create DB if it doesn't exist. 
        db_flags.set_read_write(); // RW mode.
        db_flags.set_full_mutex(); // Flag to open the database in Serialized mode.
                                   // See: https://www.sqlite.org/threadsafe.html

        let path = Path::new("/etc/rtcoin/ledger.db");
        let conn = sqlite::open_with_flags(path, db_flags)
            .expect("Could not open ledger connection");
        
        let key: Vec<u8> = Vec::new();
        
        &DB {
            conn,
            key,
        }
    }

    fn encrypt() -> Result<(), String> {
        crypt(); 
        Ok(())
    }

    fn hmac() -> Result<(), String> {
        auth(); 
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn db_test_placeholder() {
        assert_eq!(529, 23*23);
    }
}
