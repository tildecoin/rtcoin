//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    path::Path,
    sync::mpsc,
};

use rusqlite::{
    Connection,
    NO_PARAMS,
    OpenFlags,
};

use crate::crypt::*;

// Wrapper for the database connection and the
// communication channel.
pub struct DB {
    pub conn: Connection,
    pub pipe: mpsc::Receiver::<Comm>,
}

// Represents a single request, or communication,
// intended for the database worker thread.
// Includes an outbound channel for the response.
pub struct Comm {
    trans: Trans,
    origin: mpsc::Sender::<Reply>,
}

// This identifies what should be queried for.
// The assumption is that several rows will be
// expected by the caller.
pub enum Trans {
    ID(u32),
    TransactionType(String),
    Timestamp(String),
    Source(String),
    Destination(String),
    Amount(f64),
    LedgerHash(Vec<u8>),
    ReceiptID(u32),
    ReceiptHash(Vec<u8>),
}

// Response data to the Trans enum above.
pub enum Reply {
    Int(u32),
    F64(f64),
    Text(String),
    Data(Vec<u8>),
}

// Each row in the ledger table is serialized
// into an instance of this struct.
pub struct LedgerEntry {
    pub id: u32,
    pub transaction_type: String,
    pub timestamp: String,
    pub source: String,
    pub destination: String,
    pub amount: f64,
    pub ledger_hash: Vec<u8>,
    pub receipt_id: u32,
    pub receipt_hash: Vec<u8>,
}

impl Comm {
    pub fn new(trans: Trans, origin: mpsc::Sender::<Reply>) -> Comm {
        Comm {
            trans,
            origin,
        }
    }
}
impl DB {
    pub fn connect(path: &str, pipe: mpsc::Receiver::<Comm>) -> DB {
        let mut db_flags = OpenFlags::empty();
        db_flags.set(OpenFlags::SQLITE_OPEN_CREATE, true);        // Create DB if it doesn't exist. 
        db_flags.set(OpenFlags::SQLITE_OPEN_READ_WRITE, true);    // RW mode.
        db_flags.set(OpenFlags::SQLITE_OPEN_FULL_MUTEX, true);    // Flag to open the database in Serialized mode.
        db_flags.set(OpenFlags::SQLITE_OPEN_PRIVATE_CACHE, true); // Use private cache even if shared is enabled.
                                                                  // See: https://www.sqlite.org/c3ref/open.html

        let path = Path::new(path);
        let conn = Connection::open_with_flags(path, db_flags)
            .expect("Could not open ledger connection");
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS ledger (
                id              INTEGER PRIMARY KEY AUTOINCREMENT, 
                type            TEXT, 
                timestamp       TEXT, 
                source          TEXT, 
                destination     TEXT, 
                amount          REAL, 
                ledger_hash     BLOB, 
                receipt_id      INTEGER, 
                receipt_hash    BLOB
                )",
            NO_PARAMS,
            )
            .unwrap();

        DB {
            conn,
            pipe,
        }
    }

    pub fn worker_thread(&self) -> Result<(), String> {
        for comm in self.pipe.recv() {
            run_transaction(comm)?;         
        } 
        Ok(())
    }

    pub fn rows_by_user(&self, user: &str) -> Result<Vec<LedgerEntry>, rusqlite::Error> {
        let stmt = format!(
            "SELECT * FROM ledger WHERE (destination = {} OR source = {})", 
            user, 
            user,
            );
        let mut stmt = self.conn.prepare(&stmt)?;

        let rows = stmt.query_map(NO_PARAMS, |row| {
          Ok(LedgerEntry {
            id: row.get(0)?,
            transaction_type: row.get(1)?,
            timestamp: row.get(2)?,
            source: row.get(3)?,
            destination: row.get(4)?,
            amount: row.get(5)?,
            ledger_hash: row.get(6)?,
            receipt_id: row.get(7)?,
            receipt_hash: row.get(8)?,
          })  
        })?;
        
        let mut out: Vec<LedgerEntry> = Vec::new();
        for row in rows {
            out.push(row?);
        }
        
        Ok(out)
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

fn run_transaction(comm: Comm) -> Result<(), String>{
    // I meant to get to this and got sidetracked.
    // At least the communication between client
    // connections and the database/ledger is much
    // cleaner!
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn db_test_placeholder() {
        assert_eq!(529, 23*23);
    }
}
