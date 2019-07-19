//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    io::BufRead,
    io::BufReader,
    os::unix::net::UnixStream,
    sync::mpsc,
};

use crate::db;

// First handler for each new connection.
pub fn init(conn: UnixStream, pipe: mpsc::Sender::<db::Comm>) {
    let stream = BufReader::new(conn);
    for line in stream.lines() {
        println!("{}", line.unwrap());
    }

    let (tx, rx) = mpsc::channel::<db::Reply>();
    pipe.send(
        db::Comm::new(db::Trans::Destination("Henlo".into()), tx)
    ).unwrap();

    let resp: Option<db::Reply> = match rx.recv() {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Error in Ledger Worker Response: {}", err);
            None
        }
    };

    if let None = resp {
        eprintln!("Closing connection");
        return
    }
}
