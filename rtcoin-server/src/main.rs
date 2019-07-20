//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error, 
    fs, 
    os::unix::net::UnixListener, 
    path::Path, 
    sync::mpsc, 
    thread,
};

mod conn;
mod crypt;
mod db;
mod user;

use db::DB;

fn main() -> Result<(), Box<dyn Error>> {
    // Connect to the ledger database. Will create the DB
    // if it doesn't already exist.
    let (tx, rx) = mpsc::channel::<db::Comm>();
    let mut ledger = DB::connect("local/rtcoinledger.db", rx);

    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    // Spawn the ledger worker to listen for query requests.
    ledger_worker.spawn(move || {
        ledger.worker_thread();
    })?;

    // If the socket exists already, remove it.
    let sock = Path::new("local/rtcoin-serv.sock");
    if fs::metadata(sock).is_ok() {
        fs::remove_file(sock)?;
    }

    // Bind to the socket.
    let lstnr = UnixListener::bind(sock).expect(&format!(
        "Could not bind to socket: {}",
        sock.to_str().unwrap()
    ));

    // Spawn a new connection handler thread for
    // each client connection.
    while let Ok((conn, addr)) = lstnr.accept() {
        let trx = tx.clone();
        let new_conn = thread::Builder::new();
        let name = conn::addr(&addr);
        let new_conn = new_conn.name(name);
        new_conn.spawn(move || {
            conn::init(conn, trx);
        })?;
    }

    // Tidy up
    fs::remove_file(sock)?;
    Ok(())
}
