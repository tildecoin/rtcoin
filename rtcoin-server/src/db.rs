//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    path::Path,
};

use sqlite;

use crate::crypt::*;

pub struct DB {
    conn: sqlite::Connection,
    user_src_stmt: String,
    user_dest_stmt: String,
}

impl DB {
    pub fn connect(path: &str) -> DB {
        let db_flags = sqlite::OpenFlags::new();
        db_flags.set_create();     // Create DB if it doesn't exist. 
        db_flags.set_read_write(); // RW mode.
        db_flags.set_full_mutex(); // Flag to open the database in Serialized mode.
                                   // See: https://www.sqlite.org/threadsafe.html

        let path = Path::new(path);
        let conn = sqlite::Connection::open_with_flags(path, db_flags)
            .expect("Could not open ledger connection");
        
        // Schema:
        //  ID    | TYPE | TIMESTAMP(UTC) | SOURCE USER | DEST USER | AMOUNT | LEDGER HASH | RECEIPT ID | RECEIPT HASH
        //  int     text   text             text          text        real     blob          int          blob
        //  auto+                                                            |
        //                                                                   | Ledger hash is a hash of the contents
        //                                                                   | up to this entry
        conn.execute("CREATE TABLE IF NOT EXISTS ledger (id INTEGER PRIMARY KEY AUTOINCREMENT, type TEXT, timestamp TEXT, source TEXT, destination TEXT, amount REAL, ledger_hash BLOB, receipt_id INTEGER, receipt_hash BLOB)")
            .unwrap();

        let user_dest_stmt = "SELECT * FROM ledger WHERE destination = ?".to_string();
        let user_src_stmt = "SELECT * FROM ledger WHERE source = ?".to_string();
        
        DB {
            conn,
            user_src_stmt,
            user_dest_stmt,
        }
    }

    pub fn rows_by_dest_user(&self, user: &str) -> sqlite::Statement {
        let mut dest_rows = self.conn.prepare(&self.user_dest_stmt.clone()).unwrap();
        dest_rows.bind(1, user).unwrap();
        dest_rows
    }

    pub fn rows_by_src_user(&self, user: &str) -> sqlite::Statement {
        let mut src_rows = self.conn.prepare(&self.user_src_stmt.clone()).unwrap();
        src_rows.bind(1, user).unwrap();
        src_rows
    }

    pub fn encrypt(&self) -> Result<(), String> {
        crypt(); 
        Ok(())
    }

    pub fn hmac(&self) -> Result<(), String> {
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
