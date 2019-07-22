//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    io::{
        BufReader,
        BufRead,
        Read,
        Write,
    },
    os::unix::net::{
        SocketAddr, 
        UnixStream,
    },
    sync::mpsc,
};

use serde_json::{
    Value,
};

use crate::db;
use crate::user;

pub const SOCK: &str = "/tmp/rtcoinserver.sock";

// Used for quickly serializing an error into bytes
// so that it may be sent across the socket. 
#[derive(Debug)]
pub struct MsgResp {
    code: u32,
    details: String,
    context: String,
}

impl MsgResp {
    pub fn new(code: u32, details: &str) -> MsgResp {
        let details = details.to_string();
        let context = String::new();
        MsgResp {
            code,
            details,
            context,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!("{:#?}", self)
            .as_bytes()
            .to_owned()
    }

    pub fn add_context(&mut self, msg: &str) {
        self.context = msg.to_string();
    }

    pub fn code(&self) -> u32 {
        self.code
    }

    pub fn details(&self) -> String {
        self.details.clone()
    }

    pub fn context(&self) -> String {
        self.context.clone()
    }
}

// First handler for each new connection.
pub fn init(conn: UnixStream, pipe: mpsc::Sender<db::Comm>) {
    let mut conn = conn;

    let incoming = conn.try_clone().unwrap();
    let mut incoming = BufReader::new(incoming);

    let mut json_in = String::new();
    incoming.read_line(&mut json_in)
        .expect("Error reading client request");
    
    let json_in: Value = str_to_json(&json_in, &mut conn).unwrap();

    route(&mut conn, &json_in, &pipe);

    conn.bytes().next();
}

fn route(conn: &mut UnixStream, json_in: &Value, pipe: &mpsc::Sender<db::Comm>) {
    let mut conn = conn;

    match json_in["user_init"].as_str() {
        Some(a) => {
            let resp = match user::init(&json_in) {
                user::InitCode::Success => MsgResp::new(0, "Success"),
                user::InitCode::Fail(msg) => MsgResp::new(1, &msg),
            };
            let resp = resp.to_bytes();
            conn.write(&resp)
                .unwrap();
            return
        }
        None => { },
    }

    let (tx, rx) = mpsc::channel::<db::Reply>();
    if let Some(comm) = json_to_comm(&json_in, tx) {
        eprintln!("\n{:#?}", comm);
        pipe.send(comm)
            .unwrap();
    }

    let resp: Option<db::Reply> = recv(rx.recv(), &mut conn);

    if resp.is_none() {
        eprintln!("Closing client connection");
        let out = MsgResp::new(1, "No response from worker. Closing connection.").to_bytes();
        conn.write_all(&out).unwrap();
    } else if let Some(val) = resp {
        let reply = format!("{:#?}", val);
        conn.write_all(reply.as_bytes()).unwrap();
    }

}

fn str_to_json(json_in: &str, conn: &mut UnixStream) -> Option<serde_json::Value> {
    return match serde_json::from_str(&json_in) {
        Ok(val) => Some(val),
        Err(err) => {            
            let mut out = MsgResp::new(1, "Could not parse request as JSON");
            out.add_context("conn.rs#L75");

            eprintln!(
                "\nError {}:\n{}\n{}", 
                out.code(), 
                out.context(), 
                out.details(),
            );
            
            let out = out.to_bytes();
            conn.write(&out).unwrap();
            None
        }
    }
}

fn recv(recv: Result<db::Reply, mpsc::RecvError>, conn: &mut UnixStream) -> Option<db::Reply> {
    return match recv {
        Ok(val) => Some(val),
        Err(err) => {
            let err = format!("{}", err);
            
            let mut out = MsgResp::new(1, &err);
            out.add_context("conn.rs#L96");

            let out = out.to_bytes();
            conn.write(&out).unwrap();

            eprintln!("Error in Ledger Worker Response: {}", err);
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

// Serializes a JSON Value struct into a db::Comm,
// ready for passing to the ledger worker thread.
// Serialize/Deserialize serde traits apparently
// don't play well with enums.
fn json_to_comm(json: &Value, tx: mpsc::Sender<db::Reply>) -> Option<db::Comm> {
    let kind: db::Kind = match json["kind"].as_str()? {
        "BulkQuery" => db::Kind::BulkQuery,
        "BulkInsert" => db::Kind::BulkInsert,
        "BulkUpdate" => db::Kind::BulkUpdate,
        "SingleQuery" => db::Kind::SingleQuery,
        "SingleInsert" => db::Kind::SingleInsert,
        "SingleUpdate" => db::Kind::SingleUpdate,
        &_ => return None,
    };
    
    let tmp = json["trans_data"].as_str()?.to_string();

    let trans: db::Trans = match json["trans"].as_str()? {
        "ID" => {
            let id = tmp.trim().parse::<u32>().unwrap();
            db::Trans::ID(id)
        }
        "TransType" => db::Trans::TransactionType(tmp),
        "Timestamp" => db::Trans::Timestamp(tmp),
        "Source" => db::Trans::Source(tmp),
        "Destination" => db::Trans::Destination(tmp),
        "Amount" => {
            let amt = tmp.trim().parse::<f64>().unwrap();
            db::Trans::Amount(amt)
        }
        "LedgerHash" => db::Trans::LedgerHash(tmp),
        "ReceiptID" => {
            let id = tmp.trim().parse::<u32>().unwrap();
            db::Trans::ReceiptID(id)
        }
        "ReceiptHash" => db::Trans::ReceiptHash(tmp),
        &_ => return None,
    };

    Some(db::Comm::new(kind, trans, tx))
}

#[cfg(test)]
mod test {
    use super::*;

    use std::{
        fs,
        os::unix::net::UnixListener,
        path::Path,
    };

    use serde_json::json;

    #[test]
    fn test_json_to_comm() {
        let test_data = json!({
            "kind": "BulkQuery",
            "trans": "Source",
            "trans_data": "Foo Barrington",
        });

        let (tx, _) = mpsc::channel::<db::Reply>();
        let tx2 = tx.clone();

        let case = if let Some(val) = json_to_comm(&test_data, tx) {
            val
        } else {
            panic!("json_to_comm() failed: case 1");
        };

        match case.kind() {
            db::Kind::BulkQuery => { },
            _ => panic!("Incorrect Kind: case 1"),
        }
        let _src = "Foo Barrington".to_string();
        match case.trans() {
            db::Trans::Source(_src) => { },
            _ => panic!("Incorrect Trans: case 1"),
        }

        let test_data = json!({
            "kind": "SingleQuery",
            "trans": "ID",
            "trans_data": "32",
        });

        let case = if let Some(val) = json_to_comm(&test_data, tx2) {
            val
        } else {
            panic!("json_to_comm() failed: case 2");
        };

        match case.kind() {
            db::Kind::SingleQuery => { },
            _ => panic!("Incorrect Kind: case 2"),
        }
        match case.trans() {
            db::Trans::ID(32) => { },
            _ => panic!("Incorrect Trans: case 2"),
        }
    }

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
    fn msg_resp() {
        let out = MsgResp::new(0, "Test Error");
        let code = out.code();
        let details = out.details();
        
        assert_eq!(code, 0);
        assert_eq!(details, "Test Error");
    
        let mut out = out;
        out.add_context("Context");
        let context = out.context();
        assert_eq!(context, "Context");
    }
}
