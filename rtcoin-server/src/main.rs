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
    time::Duration,
};

mod conn;
mod crypt;
mod db;
mod user;

use db::{
    DB,
};

fn main() -> Result<(), Box<dyn Error>> {
    let (tx, rx) = mpsc::channel::<db::Comm>();
    let ledger = DB::connect("/etc/rtcoin/ledger.db", rx);

    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    ledger_worker.spawn(move || {
        if let Err(err) = ledger.worker_thread() {
            eprintln!("Ledger Worker Error: {}", err);
        };

        ledger.conn.close().unwrap();
    })?;

    let sock = Path::new("/tmp/rtcoin-serv.sock");
    if fs::metadata(sock).is_ok() { // if file exists...
        fs::remove_file(sock)?;
    }

    let lstnr = UnixListener::bind(sock)
        .expect(&format!("Could not bind to socket: {}", sock.to_str().unwrap()));
    lstnr.set_nonblocking(true)?;

    for conn in lstnr.incoming() {
        let tx = tx.clone();
        match conn {
            Ok(stream) => {
                thread::spawn(move || {
                    stream.set_write_timeout(Some(Duration::from_secs(5))).unwrap();
                    stream.set_read_timeout(Some(Duration::from_secs(5))).unwrap();
                    conn::init(stream, tx);
                });
            }
            
            Err(err) => {
                eprintln!("Connection error: {}", err);
            }
        }
    }

    // Tidy up
    fs::remove_file(sock)?;
    Ok(())
}

