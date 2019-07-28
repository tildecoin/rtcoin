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
    let mut rowstmt = conn.prepare(&query).unwrap();

    // Basically, what I'm doing here is:
    //      * Compare the client-provided argument to
    //          each user name
    //      * Return the public key of the user
    //          that matches.
    // This is to avoid a potential SQL injection
    // from rogue clients.    
    let rows = rowstmt.query_map(NO_PARAMS, |row| {
        Ok(
            vec![
                row.get::<usize, String>(1).unwrap().clone(),
                row.get::<usize, String>(3).unwrap().clone(),
            ]
        )
    }).unwrap_or_else(|err| {
        warn!("Query failed: {}", err);
        panic!("{}", err);
    });

    let rows = rows.filter(|row| {
        row.as_ref().unwrap()[0] == args[0]
    });

    let rows = rows.map(|row| {
        let row = row.unwrap().clone();
        row[1].clone()
    }).collect::<Vec<String>>();

    // This should be a vec with a single item, but
    // may include some weird artifacts like empty
    // items.
    let reply = db::Reply::Rows(rows.clone());

    if let Some(tx) = &comm.origin {
        tx.send(reply)
            .unwrap_or_else(|err| {
                warn!("Failed to send reply: {}", err);
                panic!("{}", err);
            });
    }
    
}