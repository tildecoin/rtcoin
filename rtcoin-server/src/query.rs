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

    let query = "SELECT * FROM users";

    // The log message is a LIE. Sort of.
    info!("New query: {} WHERE name = {}", query, args[0]);
    let _rowstmt = conn.prepare(&query).unwrap();

    // Basically, what I'm doing here is:
    //      * Compare the client-provided argument to
    //          each user name
    //      * Return the public key of the user
    //          that matches.
    // This is to avoid a potential SQL injection
    // from rogue clients.
    // Having some issues with the rusqlite library
    // and its ... wide variety of return types.
    /*
    let rows = rowstmt.query_map(NO_PARAMS, |row| {
        Ok(row)
    }).unwrap_or_else(|err| {
        warn!("Query failed: {}", err);
        panic!("{}", err);
    });

    let out = rows.filter(|row| {
        row.unwrap().get::<usize, String>(1).unwrap() == args[0]
    });

    let out = out.map(|row| row.unwrap().get(3).unwrap()).collect::<Vec<String>>();

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
    */
}