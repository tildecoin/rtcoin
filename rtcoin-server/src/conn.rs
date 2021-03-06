//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    io::{BufRead, BufReader, Write},
    net::Shutdown,
    os::unix::net::UnixStream,
    sync::mpsc,
};

use serde_json::Value;

use crate::db;
use crate::db::Kind;
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
        log::error!("Client connection error: {}", err);
        log::debug!("Failed to clone UnixStream: conn.rs::init()");
        panic!("{}", err);
    });
    let mut incoming = BufReader::new(incoming);

    loop {
        // deserialize the request
        let mut json_in = String::new();
        incoming.read_line(&mut json_in).unwrap_or_else(|err| {
            log::error!("Error reading client request: {}", err);
            log::debug!("conn.rs::init(), incoming.read_line(..), error: {}", err);
            panic!("{}", err);
        });
        let json_in: Value = json::from_str(&json_in, Some(&mut conn)).unwrap();

        if let "quit" = json_in["kind"].to_string().as_ref() {
            break;
        }

        route(&mut conn, &json_in, &pipe);
    }
}

// This handles the routing of requests from *clients*
// Internally-generated requests will bypass this
// function and be sent directly to the Ledger Worker
// thread.
fn route(conn: &mut UnixStream, json_in: &Value, pipe: &mpsc::Sender<db::Comm>) {
    let (tx, rx) = mpsc::channel::<db::Reply>();
    let comm = json::to_comm(&json_in, tx);

    if comm.is_none() {
        return;
    }

    let comm = comm.unwrap();

    // Filter out the queries users shouldn't make
    match comm.kind() {
        Kind::Disconnect => {
            invalid_request(conn, "Disconnect");
            return;
        }
        Kind::Query => {
            invalid_request(conn, "Query");
            return;
        }
        _ => {}
    }

    pipe.send(comm).unwrap();
    let resp: Option<db::Reply> = recv(rx.recv(), conn);

    if resp.is_none() {
        log::info!("Closing client connection");
        let out = err::Resp::new(
            1,
            "Worker Error",
            "No response from worker. Closing connection.",
        )
        .to_bytes();
        conn.write_all(&out).unwrap();
        conn.shutdown(Shutdown::Both).unwrap();
    } else if let Some(val) = resp {
        let reply = format!("{:#?}", val);
        conn.write_all(reply.as_bytes()).unwrap();
    }
}

fn recv(recv: Result<db::Reply, mpsc::RecvError>, conn: &mut UnixStream) -> Option<db::Reply> {
    match recv {
        Ok(val) => Some(val),
        Err(err) => {
            let err = format!("{}", err);
            let out = err::Resp::new(1, "Worker Error", &err);
            let out = out.to_bytes();

            conn.write_all(&out).unwrap();
            log::error!("Error in Ledger Worker Response: {}", err);
            None
        }
    }
}

// Response when the connection worker receives an
// external request specifying the "Disconnect" or
// "Query" actions.
fn invalid_request(conn: &mut UnixStream, kind: &str) {
    let details = format!("\"{}\" is not an allowed request type", kind);
    let msg = err::Resp::new(3, "Invalid Request", &details);
    let msg = msg.to_bytes();

    log::error!("Received invalid request from client: {}", details);

    conn.write_all(&msg).unwrap();
    conn.shutdown(Shutdown::Both).unwrap();
}
