//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    io::{
        BufReader,
        BufRead,
        Write,
    },
    net::Shutdown,
    os::unix::net::{
        SocketAddr, 
        UnixStream,
    },
    sync::mpsc,
};

use log::{
    info,
    error,
    debug,
};

use serde_json::{
    Value,
};

use crate::db;
use crate::db::{
    Kind,
};
use crate::json;
use crate::user;

pub const SOCK: &str = "/tmp/rtcoinserver.sock";

// Used for quickly serializing an error into bytes
// so that it may be sent across the socket. 
// Current error codes:
//      01: Worker error
//      02: Could not parse request as JSON
//      03: Invalid request
#[derive(Debug)]
pub struct ErrResp {
    code: u32,
    kind: String,
    details: String,
}

// These are fairly self-explanatory, boilerplate
// methods for structs with private fields.
impl ErrResp {
    pub fn new(code: u32, err: &str, details: &str) -> ErrResp {
        let kind = err.to_string();
        let details = details.to_string();
        ErrResp {
            code,
            kind,
            details,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!("{:#?}", self)
            .as_bytes()
            .to_owned()
    }
    pub fn code(&self) -> u32 {
        self.code
    }
    pub fn kind(&self) -> String {
        self.kind.clone()
    }
    pub fn details(&self) -> String {
        self.details.clone()
    }
}

// First handler for each new connection.
pub fn init(conn: UnixStream, pipe: mpsc::Sender<db::Comm>) {
    // Have to make the connection mutable for route().
    // Also, clone it to create a BufReader while still
    // retaining access to the stream (BufReader::new()
    // consumes the stream it's passed)
    let mut conn = conn;
    let incoming = conn.try_clone().unwrap_or_else(|err| {
        error!("Client connection error: {}", err);
        debug!("Failed to clone UnixStream: conn.rs::init()");
        panic!("{}", err);
    });
    let mut incoming = BufReader::new(incoming);

    // deserialize the request
    let mut json_in = String::new();
    incoming.read_line(&mut json_in)
        .unwrap_or_else(|err| {
            error!("Error reading client request: {}", err);
            debug!("conn.rs::init(), incoming.read_line(..), error: {}", err);
            panic!("{}", err);
        });   
    let json_in: Value = json::from_str(&json_in, Some(&mut conn)).unwrap();

    route(&mut conn, &json_in, &pipe);

    let mut buf = String::new();
    incoming.read_line(&mut buf)
        .expect("conn.rs::L85::init() - failed to read line for debug hold");
}

// This handles the routing of requests from *clients*
// Internally-generated requests will bypass this
// function and be sent directly to the Ledger Worker
// thread. If those are detected, respond to the client
// that we have received an invalid request.
fn route(conn: &mut UnixStream, json_in: &Value, pipe: &mpsc::Sender<db::Comm>) {
    let (tx, rx) = mpsc::channel::<db::Reply>();
    let comm = json::to_comm(&json_in, tx).unwrap();

    // need to flesh out the rest of these branches. 
    // it'll probably just be logging for now.
    match comm.kind() {
        Kind::Register => user::register(&json_in),
        Kind::Whoami => { },
        Kind::Rename => { },
        Kind::Send => { },
        Kind::Sign => { },
        Kind::Balance => { },
        Kind::Verify => { },
        Kind::Contest => { },
        Kind::Audit => { },
        Kind::Resolve => { },
        Kind::Second => { },
        Kind::Disconnect => {
            invalid_request(conn, "Disconnect");
            return
         },
        Kind::Query => {
            invalid_request(conn, "Query");
            return
         },
         &_ => {
             invalid_request(conn, "Unknown request type");
             return
         },
    }

    pipe.send(comm).unwrap();
    let resp: Option<db::Reply> = recv(rx.recv(), conn);

    if resp.is_none() {
        info!("Closing client connection");
        let out = ErrResp::new(01, "Worker Error", "No response from worker. Closing connection.").to_bytes();
        conn.write_all(&out).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();
    
    } else if let Some(val) = resp {
        let reply = format!("{:#?}", val);
        conn.write_all(reply.as_bytes()).unwrap();
    }
}

fn recv(recv: Result<db::Reply, mpsc::RecvError>, conn: &mut UnixStream) -> Option<db::Reply> {
    return match recv {
        Ok(val) => Some(val),
        Err(err) => {
            let err = format!("{}", err);
            
            let out = ErrResp::new(01, "Worker Error", &err);

            let out = out.to_bytes();
            conn.write_all(&out).unwrap();

            error!("Error in Ledger Worker Response: {}", err);
            None
        }
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

    String::from("Unknown Thread")
}

// Response when the connection worker receives an
// external request specifying the "Disconnect" or
// "Query" actions. Disconnect shuts down the
// ledger worker and Query performs arbitrary
// queries against the ledger database.
fn invalid_request(conn: &mut UnixStream, kind: &str) {
    let details = format!("\"{}\" is not an allowed request type", kind);
    let msg = ErrResp::new(03, "Invalid Request", &details);
    let msg = msg.to_bytes();

    error!("Received invalid request from client: {}", details);

    conn.write_all(&msg).unwrap();
    conn.shutdown(Shutdown::Both).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{
        fs,
        os::unix::net::UnixListener,
        path::Path,
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

    #[test]
    fn socket_addr_fail() {
        let sock_path = Path::new("");
        let sock = UnixListener::bind(sock_path).unwrap();

        let addy = sock.local_addr().unwrap();
        let name = addr(&addy);

        assert_eq!(name, "Unknown Thread");
    }

    #[test]
    fn msg_resp() {
        let out = ErrResp::new(00, "Test Error", "");
        let code = out.code();
        let kind = out.kind();
        
        assert_eq!(code, 00);
        assert_eq!(kind, "Test Error");
    }
}
