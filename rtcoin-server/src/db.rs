//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error,
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
#[derive(Debug)]
pub struct DB {
    pub conn: Connection,
    pub pipe: mpsc::Receiver::<Comm>,
}

// Represents a single request, or communication,
// intended for the database worker thread.
// Includes an outbound channel for the response.
#[derive(Debug)]
pub struct Comm {
    kind: Kind,
    trans: Trans,
    origin: mpsc::Sender::<Reply>,
}

// This identifies what should be queried for.
// The assumption is that several rows will be
// expected by the caller. The enumerated
// transaction types are subject to change as
// the design progresses.
#[derive(Debug)]
pub enum Trans {
    ID(u32),
    TransactionType(String),
    Timestamp(String),
    Source(String),
    Destination(String),
    Amount(f64),
    LedgerHash(String),
    ReceiptID(u32),
    ReceiptHash(String),
}

// Type of transaction we're doing with the
// database.
#[derive(Debug)]
pub enum Kind {
    BulkQuery,
    BulkInsert,
    BulkUpdate,
    SingleQuery,
    SingleInsert,
    SingleUpdate,
}

// Response data to the Trans enum above.
#[derive(Debug)]
pub enum Reply {
    Int(u32),
    F64(f64),
    Text(String),
    Rows(Vec<LedgerEntry>),
}

// Each row in the ledger table is serialized
// into an instance of this struct.
#[derive(Debug)]
pub struct LedgerEntry {
    pub id: u32,
    pub transaction_type: String,
    pub timestamp: String,
    pub source: String,
    pub destination: String,
    pub amount: f64,
    pub ledger_hash: String,
    pub receipt_id: u32,
    pub receipt_hash: String,
}

impl Comm {
    // Cleanly package up a new request for
    // the ledger database worker thread.
    pub fn new(kind: Kind, trans: Trans, origin: mpsc::Sender::<Reply>) -> Comm {
        Comm {
            kind,
            trans,
            origin,
        }
    }
}
impl DB {
    // Connect to the ledger database, creating it
    // if necessary.
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
                ledger_hash     TEXT, 
                receipt_id      INTEGER, 
                receipt_hash    TEXT
                )",
            NO_PARAMS,
            )
            .expect("Could not create table");

        DB {
            conn,
            pipe,
        }
    }

    // Continually read from the channel to
    // process the incoming Comms.
    pub fn worker_thread(&mut self) -> Result<(), Box<dyn Error>> {
        for comm in self.pipe.recv() {
            match comm.kind {
                Kind::BulkQuery => bulk_query(&mut self.conn, comm)?,
                _ => bulk_query(&mut self.conn, comm)?,
            }
        } 
        Ok(())
    }

    // Return the rows associated with a single
    // user, receiving and sending entries.
    pub fn rows_by_user(&self, user: &str) -> Result<Vec<LedgerEntry>, rusqlite::Error> {
        let stmt = format!(
            "SELECT * FROM ledger WHERE (destination = {} OR source = {})", 
            user, 
            user,
            );

        let stmt = self.conn.prepare(&stmt)?;
        let out = serialize_rows(stmt).unwrap();
        
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

// Returns a vector of LedgerEntry structs, each representing
// a single row returned by this query.
fn bulk_query(db: &mut Connection, comm: Comm) -> Result<(), Box<dyn Error>> {
    let trans_info = comm.trans;
    let mut stmt = "SELECT * WHERE ".to_string();
    
    match trans_info {
        Trans::ID(n) => stmt.push_str(&format!("id = {}", n)),
        Trans::TransactionType(n) => stmt.push_str(&format!("type = {}", n)),
        Trans::Timestamp(n) => stmt.push_str(&format!("timestamp = {}", n)),
        Trans::Source(n) => stmt.push_str(&format!("source = {}", n)),
        Trans::Destination(n) => stmt.push_str(&format!("destination = {}", n)),
        Trans::Amount(n) => stmt.push_str(&format!("amount = {}", n)),
        Trans::LedgerHash(n) => stmt.push_str(&format!("ledger_hash = {}", n)),
        Trans::ReceiptID(n) => stmt.push_str(&format!("receipt_id = {}", n)),
        Trans::ReceiptHash(n) => stmt.push_str(&format!("receipt_hash = {}", n)),
    }

    let src_channel = comm.origin;
    let txn = db.transaction()?;
    let stmt = txn.prepare(&stmt)?;

    let out = serialize_rows(stmt)?;
    src_channel.send(Reply::Rows(out))?;

    Ok(())
}

// Serializes the rows returned from a query into
// a Vec of the LedgerEntry struct.
fn serialize_rows(stmt: rusqlite::Statement) -> Result<Vec<LedgerEntry>, Box<dyn Error>> {
    let mut stmt = stmt;
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn db_test_placeholder() {
        assert_eq!(529, 23*23);
    }
}
