//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use log::{
    error,
    info,
};
use rusqlite;

use crate::db;
use crate::err;

// Accepts the comm of kind Whoami and arg of
//     vec["user", (username)]
// Responds with the public key associated 
// with the account.
pub fn whoami(comm: db::Comm, conn: &rusqlite::Connection) {
    let args = comm.args();
    let query = "SELECT pubkey FROM users WHERE name = :name";
    
    // The ownership system causes the channel to 
    // be moved, so I had to clone it.
    let origin_channel = comm.origin.unwrap().clone();
    // Twice.
    let origin_channel_clone = origin_channel.clone();

    // Prevent out-of-bounds panics
    // from malformed requests.
    let user = match args.len() {
        0 => String::from("UNKNOWN USER"),
        1 => args[0].clone(),
        _ => args[1].clone(),
    };

    let query_for_logs = format!("{}, {}", query, user);
    info!("New query: {}", query_for_logs);
    let mut rowstmt = conn.prepare(&query).unwrap();

    let row = rowstmt.query_row_named(
        &[(":name", &user)], 
        |row| {
            Ok(row.get(0).unwrap())
        })
        .unwrap_or_else(|err| {
            let err = format!("Query failed: {}", err);
            error!("{} :: {}", err, query_for_logs);
            err::Resp::new(04, "Query Error", &err).to_string()
        });

    let reply = match row.contains("Query Error") {
            true => db::Reply::Error(row),
            false => db::Reply::Data(row),
        };

    if let Err(err) = origin_channel_clone.send(reply) {
        error!("Failed to send reply: {}", err);
    }
}