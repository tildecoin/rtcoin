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
use crate::err;
use crate::json;

pub const SOCK: &str = "/tmp/rtcoinserver.sock";

// First handler for each new connection.
pub fn init(mut conn: UnixStream, pipe: mpsc::Sender<db::Comm>) {
    // Have to make the connection mutable for route().
    // Also, clone it to create a BufReader while still
    // retaining access to the stream (BufReader::new()
    // consumes the stream it's passed)
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
// thread.
fn route(conn: &mut UnixStream, json_in: &Value, pipe: &mpsc::Sender<db::Comm>) {
    let (tx, rx) = mpsc::channel::<db::Reply>();
    let comm = json::to_comm(&json_in, tx).unwrap();

    // Filter out the queries users shouldn't make
    match comm.kind() {
        Kind::Disconnect => {
            invalid_request(conn, "Disconnect");
            return
        }
        Kind::Query => {
            invalid_request(conn, "Query");
            return
         }
         _ => { }
    }

    pipe.send(comm).unwrap();
    let resp: Option<db::Reply> = recv(rx.recv(), conn);

    if resp.is_none() {
        info!("Closing client connection");
        let out = err::Resp::new(01, "Worker Error", "No response from worker. Closing connection.").to_bytes();
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
            let out = err::Resp::new(01, "Worker Error", &err);
            let out = out.to_bytes();

            conn.write_all(&out).unwrap();
            error!("Error in Ledger Worker Response: {}", err);
            
            None
        }
    }
}

// Response when the connection worker receives an
// external request specifying the "Disconnect" or
// "Query" actions.
fn invalid_request(conn: &mut UnixStream, kind: &str) {
    let details = format!("\"{}\" is not an allowed request type", kind);
    let msg = err::Resp::new(03, "Invalid Request", &details);
    let msg = msg.to_bytes();

    error!("Received invalid request from client: {}", details);

    conn.write_all(&msg).unwrap();
    conn.shutdown(Shutdown::Both).unwrap();
}