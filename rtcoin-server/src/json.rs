//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    io::Write,
    os::unix::net::UnixStream,
    sync::mpsc,
};

use crate::db;
use crate::db::{
    Kind,
};
use crate::err;

use serde_json::Value;
use log::error;

// Deserializes a JSON Value struct into a db::Comm,
// ready for passing to the ledger worker thread.
// Serialize/Deserialize serde traits apparently
// don't play well with enums.
pub fn to_comm(json: &Value, tx: mpsc::Sender<db::Reply>) -> Option<db::Comm> {
    let kind: db::Kind = match json["kind"].as_str()? {
        "Register" => Kind::Register,
        "Whoami" => Kind::Whoami,
        "Rename" => Kind::Rename,
        "Send" => Kind::Send,
        "Sign" => Kind::Sign,
        "Balance" => Kind::Balance,
        "Verify" => Kind::Verify,
        "Contest" => Kind::Contest,
        "Audit" => Kind::Audit,
        "Resolve" => Kind::Resolve,
        "Second" => Kind::Second,
        "Query" => Kind::Query,             // Query and Disconnect are internal
        "Disconnect" => Kind::Disconnect,   // values for miscellaneous database
        &_ => return None,                  // queries and shutting down the DB.
    };

    let args = json["args"].as_str()?
        .split_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    Some(
        db::Comm::new(
            Some(kind), 
            Some(args), 
            Some(tx),
        )
    )
}

// Takes a string, outputs JSON.
// If there's an error, sends an error down the socket.
// TODO: This shouldn't do that last bit. Leave that up to the caller.
pub fn from_str(json_in: &str, conn: Option<&mut UnixStream>) -> Option<serde_json::Value> {
    return match serde_json::from_str(&json_in) {
        Ok(val) => Some(val),
        Err(err) => {
            let err = format!("{}", err);
            let out = err::Resp::new(02, "JSON Error", &err);

            error!(
                "\nError {}:\n{}\n{}", 
                out.code(), 
                out.kind(), 
                out.details(),
            );
            
            let out = out.to_bytes();

            if let Some(conn) = conn {
                conn.write_all(&out).unwrap();
            }
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_from_string() {
        let rhs = json!({
            "kind": "value",
            "args": "value",
        });

        let lhs_proto = "{ \"kind\": \"value\", \"args\": \"value\" }";
        let lhs = from_str(lhs_proto, None).unwrap();

        assert_eq!(lhs, rhs);

        if let Some(val) = from_str("foo BAR invalid json", None) {
            panic!("That was invalid, why did it pass? {}", val);
        }

        assert_eq!(from_str("MORE INVALID json", None), None);
    }

    #[test]
        fn test_json_to_comm() {
        let test_data = json!({
            "kind":        "Disconnect",
            "args":        "Source Foo"
        });

        let (tx, _) = mpsc::channel::<db::Reply>();
        let tx2 = tx.clone();
        let tx3 = tx.clone();

        let case = if let Some(val) = to_comm(&test_data, tx) {
            val
        } else {
            panic!("to_comm() failed: case 1");
        };

        match case.kind() {
            db::Kind::Disconnect => { },
            _ => panic!("Incorrect Kind: case 1"),
        }

        let test_data = json!({
            "kind":        "Send",
            "args":        "From Foo To Bob"
        });

        let case = if let Some(val) = to_comm(&test_data, tx2) {
            val
        } else {
            panic!("to_comm() failed: case 2");
        };

        match case.kind() {
            db::Kind::Send => { },
            _ => panic!("Incorrect Kind: case 2"),
        }

        let test_data = json!({
            "kind": "FOOBAR",
            "args": "some args here"
        });

        match to_comm(&test_data, tx3) {
            None => { },
            _ => panic!("Received some, expected none: case 3"),
        }
    }

}