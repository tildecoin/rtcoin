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

use serde_json::{
    Result,
    Value,
};

use crate::db;

// First handler for each new connection.
pub fn init(conn: UnixStream, pipe: mpsc::Sender<db::Comm>) {
    let mut stream = BufReader::new(conn);
    let mut json_in: Vec<u8> = Vec::new();
    stream.read_until("\0".as_bytes()[0], &mut json_in)
        .expect("Error reading client request");
    
    let json_in = String::from_utf8_lossy(&json_in);
    let json_in: Value = serde_json::from_str(&json_in).unwrap();

    let (tx, rx) = mpsc::channel::<db::Reply>();
    if let Some(comm) = json_to_comm(json_in, tx) {
        pipe.send(comm)
            .unwrap();
    }

    let resp: Option<db::Reply> = match rx.recv() {
        Ok(val) => Some(val),
        Err(err) => {
            eprintln!("Error in Ledger Worker Response: {}", err);
            None
        }
    };

    if resp.is_none() {
        eprintln!("Closing connection");
    } else if let Some(val) = resp {
        println!("{:#?}", val);
    }

    stream.lines().next();
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

fn json_to_comm(json: Value, tx: mpsc::Sender<db::Reply>) -> Option<db::Comm> {
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

    let mut trans: db::Trans = match json["trans"].as_str()? {
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

        let lhs = if let Some(val) = json_to_comm(test_data, tx) {
            val
        } else {
            panic!("json_to_comm() failed");
        };

        let _rhs = db::Comm::new(
            db::Kind::BulkQuery, 
            db::Trans::Source("Foo Barrington".into()), 
            tx2,
        );

        match lhs.kind() {
            db::Kind::BulkQuery => { },
            _ => panic!("Incorrect query kind"),
        }
        let _src = "Foo Barrington".to_string();
        match lhs.trans() {
            db::Trans::Source(_src) => { },
            _ => panic!("Incorrect transaction detail"),
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
}
