////
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{path::Path, sync::mpsc};

use log::info;

use rusqlite::{Connection, OpenFlags, NO_PARAMS};

use zeroize::Zeroize;

use crate::{err, query, user};

pub const PATH: &str = "/tmp/rtcoinserver.db";

// Wrapper for the database connection and the
// communication channel.
#[derive(Debug)]
pub struct DB {
    pub conn: Connection,
    pub pipe: mpsc::Receiver<Comm>,
}

// Represents a single request, or communication,
// intended for the database worker thread.
// Includes an outbound channel for the response.
#[derive(Debug, Clone)]
pub struct Comm {
    pub kind: Option<Kind>,
    pub args: Option<Vec<String>>,
    pub origin: Option<mpsc::Sender<Reply>>,
}

// Type of transaction we're doing with the
// database.
#[derive(Debug, Clone)]
pub enum Kind {
    Register,
    Query,
    Whoami,
    Rename,
    Send,
    Sign,
    Balance,
    Verify,
    Contest,
    Audit,
    Resolve,
    Second,
    Disconnect,
    Empty,
}

// When rows are serialized into plain text
// and packed into this enum, they are tab-separated
// to delineate columns. To make the reply route
// monomorphic, err::Resp will be converted to
// a string and packed into Reply::Data()
#[derive(Debug, Clone)]
pub enum Reply {
    Data(String),
    Error(String),
    Info(String),
    Rows(Vec<String>),
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

// Same, but for archive table rows.
#[derive(Debug)]
pub struct ArchiveEntry {
    pub id: u32,
    pub transaction_type: String,
    pub timestamp: String,
    pub state: String,
    pub merkle_hash: Vec<u8>,
    pub hash: String,
    pub filename: String,
}

#[derive(Debug)]
pub struct UserEntry {
    pub id: u32,
    pub name: String,
    pub pass: String,
    pub pubkey: String,
    pub balance: f64,
    pub messages: Vec<String>,
    pub created: String,
    pub last_login: String,
}

impl Comm {
    // Cleanly package up a new request for
    // the ledger database worker thread.
    pub fn new(
        kind: Option<Kind>,
        args: Option<Vec<String>>,
        origin: Option<mpsc::Sender<Reply>>,
    ) -> Comm {
        Comm { kind, args, origin }
    }

    pub fn kind(&self) -> &Kind {
        match &self.kind {
            Some(kind) => return &kind,
            None => return &Kind::Empty,
        }
    }
    pub fn args(&self) -> Vec<String> {
        match &self.args {
            Some(args) => return args.clone(),
            None => return Vec::<String>::new(),
        }
    }
}

impl DB {
    // Connect to the ledger database, creating it
    // if necessary.
    pub fn connect(path: &str, mut db_key: String, pipe: mpsc::Receiver<Comm>) -> DB {
        let mut db_flags = OpenFlags::empty();
        db_flags.set(OpenFlags::SQLITE_OPEN_CREATE, true); // Create DB if it doesn't exist.
        db_flags.set(OpenFlags::SQLITE_OPEN_READ_WRITE, true); // RW mode.
        db_flags.set(OpenFlags::SQLITE_OPEN_FULL_MUTEX, true); // Flag to open the database in Serialized mode.
        db_flags.set(OpenFlags::SQLITE_OPEN_PRIVATE_CACHE, true); // Use private cache even if shared is enabled.
                                                                  // See: https://www.sqlite.org/c3ref/open.html
        let path = Path::new(path);
        let conn = Connection::open_with_flags(path, db_flags).unwrap_or_else(|error| {
            err::log_then_panic("Could not open database connection", error);
            panic!();
        });

        // This PRAGMA is what either enables
        // encryption on a new database or allows
        // the decryption of an existing database.
        let mut pragma = format!("PRAGMA key = '{}'", db_key);
        db_key.zeroize();

        conn.execute(&pragma, NO_PARAMS).unwrap_or_else(|error| {
            err::log_then_panic("Database authentication failure", error);
            panic!();
        });

        pragma.zeroize();

        // This has a dual purpose: First, create the three
        // tables on first startup. If subsequent startups
        // fail to execute these statements, the key is
        // incorrect.
        startup_check_tables(&conn);

        DB { conn, pipe }
    }

    // Continually read from the channel to
    // process the incoming Comms.
    pub fn worker_thread(&self) -> Comm {
        while let Ok(comm) = self.pipe.recv() {
            info!("Ledger Worker :: Received {:?}", comm);
            match comm.kind {
                Some(Kind::Register) => user::register(&comm, &self.conn),
                Some(Kind::Whoami) => query::whoami(comm, &self.conn),
                Some(Kind::Rename) => {}
                Some(Kind::Send) => {}
                Some(Kind::Sign) => {}
                Some(Kind::Balance) => {}
                Some(Kind::Verify) => {}
                Some(Kind::Contest) => {}
                Some(Kind::Audit) => {}
                Some(Kind::Resolve) => {}
                Some(Kind::Second) => {}
                Some(Kind::Query) => {}
                Some(Kind::Disconnect) => return comm.clone(),
                _ => continue,
            };
        }
        Comm {
            kind: None,
            args: None,
            origin: None,
        }
    }
}

// Just pulled out these statements to clean up DB::connect()
fn startup_check_tables(conn: &rusqlite::Connection) {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS ledger (
                id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL, 
                type            TEXT NOT NULL, 
                timestamp       TEXT NOT NULL, 
                source          TEXT NOT NULL, 
                destination     TEXT NOT NULL, 
                amount          REAL NOT NULL, 
                ledger_hash     TEXT NOT NULL, 
                receipt_id      INTEGER NOT NULL, 
                receipt_hash    TEXT NOT NULL
            )",
        NO_PARAMS,
    )
    .expect("Could not create ledger table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS archive (
                id              INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                type            TEXT NOT NULL,
                timestamp       TEXT NOT NULL,
                state           TEXT NOT NULL,
                merkle_hash     TEXT NOT NULL,
                hash            TEXT NOT NULL,
                filename        TEXT NOT NULL
            )",
        NO_PARAMS,
    )
    .expect("Could not create archive table");

    conn.execute(
        "CREATE TABLE IF NOT EXISTS users (
                id          INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
                name        TEXT NOT NULL,
                pass        TEXT NOT NULL,
                pubkey      TEXT NOT NULL,
                balance     REAL NOT NULL,
                messages    TEXT,
                created     TEXT NOT NULL,
                last_login  TEXT NOT NULL
            )",
        NO_PARAMS,
    )
    .expect("Could not create users table");
}
