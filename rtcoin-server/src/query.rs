//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use log::{
    info,
    warn,
};
use rusqlite;
use crate::db;

// Accepts the comm of kind Whoami and arg of
//     vec["user", (username)]
// Responds with the public key associated 
// with the account.
pub fn whoami(comm: &db::Comm, conn: &rusqlite::Connection) {
    let args = comm.args();
    let query = "SELECT pubkey FROM users WHERE name = :name";

    info!("New query: {}, {}", query, args[1]);
    let mut rowstmt = conn.prepare(&query).unwrap();

    let row = rowstmt.query_row_named(
        &[(":name", &args[1])], 
        |row| {
            Ok(row.get(0).unwrap())
        })
        .unwrap_or_else(|err| {
            warn!("Query failed: {}", err);
            panic!("{}", err);
        });

    let reply = db::Reply::Data(row);

    if let Some(tx) = &comm.origin {
        tx.send(reply)
            .unwrap_or_else(|err| {
                warn!("Failed to send reply: {}", err);
                panic!("{}", err);
            });
    }
}