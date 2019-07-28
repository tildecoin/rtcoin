//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use log::{
    error,
    info,
    warn,
};
use rusqlite;
use rusqlite::{
    NO_PARAMS,
};
use crate::db;

// Accepts the comm of kind Whoami and arg of just a 
// username. Responds with the public key associated 
// with the account.
pub fn whoami(comm: &db::Comm, conn: &rusqlite::Connection) {
    let args = comm.args();

    // This next line is insecure.
    let query = format!("SELECT * FROM users WHERE name = '{}'", args[0]);

    info!("New query: {}", query);
    let mut rowstmt = conn.prepare(&query).unwrap();

    let rows = rowstmt.query_map(NO_PARAMS, |row| {
        Ok(row.get(3).unwrap())
    }).unwrap_or_else(|err| {
        error!("Failed to query Whoami: {}", err);
        panic!("{}");
    });

    // There should only be one matching row,
    // but we still should treat it as multiple
    // and process it from there.
    let out = rows.map(|row| {
        row.unwrap()
    })
    .collect::<Vec<String>>();

    // Like here, where we just construct the
    // reply with index 0.
    let reply = db::Reply::Data(out[0].clone());

    if let Some(tx) = &comm.origin {
        tx.send(reply)
            .unwrap_or_else(|err| {
                warn!("Failed to send reply: {}", err);
                panic!("{}", err);
            });
    }
}