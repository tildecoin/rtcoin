//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error,
    io::BufRead,
    io::BufReader,
    os::unix::net::{
        SocketAddr, 
        UnixStream
    },
    path::Path,
    sync::mpsc,
};

use crate::db;

// First handler for each new connection.
pub fn init(conn: UnixStream, pipe: mpsc::Sender<db::Comm>) {
    let stream = BufReader::new(conn);
    for line in stream.lines() {
        println!("{}", line.unwrap());
    }

    let (tx, rx) = mpsc::channel::<db::Reply>();
    pipe.send(db::Comm::new(
        db::Kind::BulkQuery,
        db::Trans::Destination("Henlo".into()),
        tx,
    ))
    .unwrap();

    let resp: Option<db::Reply> = match rx.recv() {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Error in Ledger Worker Response: {}", err);
            None
        }
    };

    if let None = resp {
        eprintln!("Closing connection");
        return;
    } else if let Some(val) = resp {
        println!("{:#?}", val);
    }
}

// Grabs the connection's peer address. Used to
// name the thread spawned for the connection
// so we can better pinpoint which thread caused
// a given problem during debugging.
pub fn addr(addr: &SocketAddr) -> String {
    if let Some(n) = addr.as_pathname() {
        let path = n;
        if let Some(n) = path.to_str() {
            return n.to_string();
        };
    };

    return String::from("Unknown Thread");
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{
        fs,
        os::unix::net::UnixListener,
    };

    #[test]
    fn socket_addr() {
        let sock_path = Path::new("test-sock");
        let sock = UnixListener::bind(sock_path).unwrap();

        let addy = sock.local_addr().unwrap();
        let name = addr(&addy);

        assert_eq!(name, "test-sock");

        if fs::metadata(sock_path).is_ok() {
            fs::remove_file(sock_path).unwrap();
        }
    }
}
