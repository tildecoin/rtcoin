//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error,
    fs,
    io::BufRead,
    io::BufReader,
    os::unix::net::UnixStream,
    os::unix::net::UnixListener,
    path::Path,
    thread,
};

mod crypt;
mod db;
mod user;

fn main() -> Result<(), Box<dyn Error>> {
    let sock = Path::new("/tmp/rtcoin-serv.sock");
    if fs::metadata(sock).is_ok() {
        fs::remove_file(sock)?;
    }

    let lstnr = UnixListener::bind(sock)
        .expect("Could not bind to socket /tmp/rtcoin-serv.sock");

    for conn in lstnr.incoming() {
        match conn {
            Ok(c) => {
                thread::spawn(move || {
                    init_conn(c);
                });
            },
            Err(err) => eprintln!("Connection error: {}", err),
        }
    }

    fs::remove_file(sock)?;
    Ok(())
}

fn init_conn(conn: UnixStream) {
    let stream = BufReader::new(conn);
    for line in stream.lines() {
        println!("{}", line.unwrap());
    }
}
