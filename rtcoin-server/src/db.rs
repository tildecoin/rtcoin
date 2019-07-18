//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::path::Path;

use sqlite;

mod crypt;

struct DB {
    conn: sqlite::Connection,
    user_src_stmt: sqlite::Statement;
    user_dest_stmt: sqlite::Statement;
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
        
        conn.execute("CREATE TABLE IF NOT EXISTS ledger (id INTEGER PRIMARY KEY AUTOINCREMENT, type TEXT, timestamp TEXT, source TEXT, destination TEXT, amount REAL, ledger_hash BLOB, receipt_id INTEGER, receipt_hash BLOB)")
            .unwrap();

        let key: Vec<u8> = Vec::new();

        let user_dest_stmt = conn.prepare("SELECT * FROM ledger WHERE dest = ?");
        let user_src_stmt = conn.prepare("SELECT * FROM ledger WHERE source = ?");
        
        &DB {
            conn,
            user_src_stmt,
            user_dest_stmt,
            key,
        }
    }

    fn retrieve_dest(&self, user: &str) -> sqlite::Statement {
        let mut dest_rows = self.user_dest_stmt.clone();
        dest_rows.bind(user).unwrap()
    }

    fn retrieve_src(&self, user: &str) -> sqlite::Statement {
        let mut src_rows = self.user_src_stmt.clone();
        src_rows.bind(user).unwrap()
    }

    fn encrypt(&self) -> Result<(), String> {
        crypt(); 
        Ok(())
    }

    fn hmac(&self) -> Result<(), String> {
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
