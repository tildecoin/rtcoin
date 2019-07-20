//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error, 
    fs, 
    os::unix::net::UnixListener, 
    path::Path, 
    process, 
    sync::mpsc, 
    thread,
};

use ctrlc;

mod conn;
mod crypt;
mod db;
mod user;

use db::DB;

fn main() -> Result<(), Box<dyn Error>> {
    // Connect to the ledger database. Will create the DB
    // if it doesn't already exist.
    let (tx, rx) = mpsc::channel::<db::Comm>();

    // Spawn the ledger worker to listen for query requests.
    spawn_ledger_worker_with_receiver(rx)?;

    // If the socket exists already, remove it.
    let sock = Path::new("local/rtcoin-serv.sock");
    if fs::metadata(sock).is_ok() {
        fs::remove_file(sock)?;
    }

    // Handle SIGINT / ^C
    ctrlc::set_handler(move || {
        eprintln!(" Caught. Cleaning up ...");
        if fs::metadata(sock).is_ok() {
            fs::remove_file(sock).unwrap();
        }
        process::exit(0);
    })
    .expect("SIGINT handler setup failure");

    // Bind to the socket. Spawn a new connection
    // handler thread for each client connection.
    spawn_for_connections(&sock, tx);

    // Tidy up
    fs::remove_file(sock)?;
    Ok(())
}

fn spawn_ledger_worker_with_receiver(rx: mpsc::Receiver<db::Comm>) -> Result<(), Box<dyn Error>> {
    let mut ledger = DB::connect("local/rtcoinledger.db", rx);

    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    ledger_worker.spawn(move || {
        ledger.worker_thread();
    })?;

    Ok(())
}

fn spawn_for_connections(sock: &Path, tx: mpsc::Sender<db::Comm>) {
    let lstnr = UnixListener::bind(sock).unwrap_or_else(|_|{
        panic!("Could not bind to socket: {}",
        sock.to_str().unwrap())

    });

    while let Ok((conn, addr)) = lstnr.accept() {
        let trx = tx.clone();
        let new_conn = thread::Builder::new();
        let name = conn::addr(&addr);
        let new_conn = new_conn.name(name);
        new_conn
            .spawn(move || {
                conn::init(conn, trx);
            })
            .unwrap();
    }
}
