//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error,
    path::Path, 
    sync::mpsc,
};

use log::{
    error,
    info,
};

use rusqlite::{
    Connection, 
    OpenFlags, 
    NO_PARAMS,
};

use zeroize::Zeroize;

pub const PATH: &str = "/tmp/rtcoinledger.db";

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
#[derive(Debug)]
pub struct Comm {
    kind: Option<Kind>,
    args: Option<Vec<String>>,
    origin: Option<mpsc::Sender<Reply>>,
}

// Type of transaction we're doing with the
// database.
#[derive(Debug)]
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
// to delineate columns.
#[derive(Debug, Clone)]
pub enum Reply {
    Data(String),
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

impl Comm {
    // Cleanly package up a new request for
    // the ledger database worker thread.
    pub fn new(kind: Option<Kind>, args: Option<Vec<String>>, origin: Option<mpsc::Sender<Reply>>) -> Comm {
        Comm {
            kind,
            args,
            origin,
        }
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
    pub fn connect(path: &str, db_key: String, pipe: mpsc::Receiver<Comm>) -> DB {
        let mut db_flags = OpenFlags::empty();
        db_flags.set(OpenFlags::SQLITE_OPEN_CREATE, true);        // Create DB if it doesn't exist.
        db_flags.set(OpenFlags::SQLITE_OPEN_READ_WRITE, true);    // RW mode.
        db_flags.set(OpenFlags::SQLITE_OPEN_FULL_MUTEX, true);    // Flag to open the database in Serialized mode.
        db_flags.set(OpenFlags::SQLITE_OPEN_PRIVATE_CACHE, true); // Use private cache even if shared is enabled.
                                                                  // See: https://www.sqlite.org/c3ref/open.html
        let path = Path::new(path);
        let conn =
            Connection::open_with_flags(path, db_flags)
                .unwrap_or_else(|err| {
                    error!("Could not open ledger connection: {}", err);
                    panic!("{}", err);
                });

        // This PRAGMA is what either enables
        // encryption on a new database or allows 
        // the decryption of an existing database.
        let mut pragma = format!("PRAGMA key = '{}'", db_key);
        let mut db_key = db_key;
        db_key.zeroize();

        conn.execute(&pragma, NO_PARAMS)
            .unwrap_or_else(|err| {
                error!("When authenticating with database: {}", err);
                panic!("{}", err);
            });

        pragma.zeroize();

        // This has a dual purpose: First, create the three
        // tables on first startup. If subsequent startups
        // fail to execute these statements, the key is
        // incorrect.
        startup_check_tables(&conn);

        DB { 
            conn, 
            pipe, 
        }
    }

    // Continually read from the channel to
    // process the incoming Comms.
    pub fn worker_thread(&mut self) {
        while let Ok(comm) = self.pipe.recv() {
            info!("Ledger Worker :: Received {:?}", comm);
            match comm.kind {
                Some(Kind::Disconnect) => return,
                _ => continue,
            }
        }
    }
}

// Just pulled out these statements to clean up DB::connect()
fn startup_check_tables(conn: &rusqlite::Connection) {
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
        .expect("Could not create ledger table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS archive (
                id              INTEGER PRIMARY KEY AUTOINCREMENT,
                type            TEXT,
                timestamp       TEXT,
                state           TEXT,
                merkle_hash     TEXT,
                hash            TEXT,
                filename        TEXT
            )",
            NO_PARAMS,
        )
        .expect("Could not create archive table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id          INTEGER PRIMARY KEY AUTOINCREMENT,
                name        TEXT,
                pass        TEXT,
                balance     REAL,
                messages    TEXT,
                created     TEXT,
                last_login  TEXT
            )",
            NO_PARAMS,
        )
        .expect("Could not create users table");
}

// Deserializes the rows returned from a query into
// a Vec of the LedgerEntry struct.
fn deserialize_rows(stmt: rusqlite::Statement) -> Result<Vec<LedgerEntry>, Box<dyn Error>> {
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

    Ok(
        rows.map(|row| {
            row.unwrap()
        })
        .collect::<Vec<LedgerEntry>>()
    )
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{
        fs,
        thread,
    };

    #[test]
    fn worker_thread_spawn_send_recv_serialize_rows() {
        let path = "./test-db";
        let (worker_tx, pipe) = mpsc::channel::<Comm>();
        let test_key = "something something password";
        let mut db = DB::connect(path, test_key.into(), pipe);

        assert!(fs::metadata(path).is_ok());

        let kind = Kind::Balance;
        let args: Vec<String> = vec!["Bob".into()];
        let (tx_case1, _rx_case1) = mpsc::channel::<Reply>();
        let comm = Comm::new(Some(kind), Some(args), Some(tx_case1));

        let stmt = "SELECT * FROM ledger WHERE Source = 'Bob'";
        let stmt = db.conn.prepare(stmt).unwrap();

        if let Err(_) = deserialize_rows(stmt) {
            panic!("failure in serialize_rows()");
        }
        
        // Above, comm takes ownership of the previous
        // instances of kind and trans. Need to duplicate
        // to test bulk_query(). Also, Clone isn't implemented
        // on db::Comm yet.
        let kind = Kind::Query;
        let args: Vec<String> = vec!["src".into(), "Bob".into()];
        let (tx_case2, _rx_case2) = mpsc::channel::<Reply>();
        let comm2 = Comm::new(Some(kind), Some(args), Some(tx_case2));

        thread::spawn(move || {
            db.worker_thread();
        });
        
        worker_tx.send(comm).unwrap();
        worker_tx.send(comm2).unwrap();

        // the worker passes the comm packet to bulk_query(),
        // which hands it off to serialize_rows() before sending
        // it back down the channel to be received here.
        //rx_case1.recv().unwrap();
        //rx_case2.recv().unwrap();

        if fs::metadata(path).is_ok() {
            fs::remove_file(path).unwrap();
        }
    }

    #[test]
    fn comm_kind() {
        let (tx, _) = mpsc::channel::<Reply>();
        let kind = Kind::Query;
        let args: Vec<String> = vec!["Source".into(),"Bob".into()];
        let comm = Comm::new(Some(kind), Some(args), Some(tx));

        match comm.kind() {
            Kind::Query => { },
            _ => panic!("Incorrect Kind"),
        }

        let arg1 = comm.args()[0].clone();
        let arg2 = comm.args()[1].clone();
        match &arg1[..] {
            "Source" => { },
            _ => panic!("Incorrect arguments"),
        }
        match &arg2[..] {
            "Bob" => { },
            _ => panic!("Incorrect arguments"),
        }
    }
}
