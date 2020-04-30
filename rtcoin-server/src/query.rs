//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use rusqlite::NO_PARAMS;

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
    let origin_channel = comm.origin.unwrap();
    // Twice.
    let origin_channel_clone = origin_channel;

    // Prevent out-of-bounds panics
    // from malformed requests.
    let user = match args.len() {
        0 => String::from("UNKNOWN USER"),
        1 => args[0].clone(),
        _ => args[1].clone(),
    };

    let query_for_logs = format!("{}, {}", query, user);
    log::info!("New query: {}", query_for_logs);
    let mut rowstmt = conn.prepare(&query).unwrap();
    // If the query fails, we can return an err::Resp
    // as long as it's been serialized into a string.
    // Lets us continue on without adding complex
    // execution branches for handling errors.
    let row = rowstmt
        .query_row_named(&[(":name", &user)], |row| Ok(row.get(0).unwrap()))
        .unwrap_or_else(|err| {
            let err = format!("Query failed: {}", err);
            log::error!("{} :: {}", err, query_for_logs);
            err::Resp::new(4, "Query Error", &err).to_string()
        });

    let reply = if row.contains("Query Error") {
        db::Reply::Error(row)
    } else {
        db::Reply::Data(row)
    };

    if let Err(err) = origin_channel_clone.send(reply) {
        log::error!("Failed to send reply: {}", err);
    }
}

// Takes the rows returned from a query and packs them into
// a Vec of the db::LedgerEntry struct.
pub fn to_ledger_entry(mut stmt: rusqlite::Statement) -> rusqlite::Result<Vec<db::LedgerEntry>> {
    let rows = stmt.query_map(NO_PARAMS, |row| {
        Ok(db::LedgerEntry {
            id: row.get(0)?,
            transaction_type: row.get(1)?,
            timestamp: row.get(2)?,
            source: row.get(3)?,
            destination: row.get(4)?,
            amount: row.get(5)?,
            ledger_hash: row.get(6)?,
            receipt_id: row.get(7)?,
            receipt_hash: row.get(8)?,
        })
    })?;

    Ok(rows
        .map(|row| row.unwrap())
        .collect::<Vec<db::LedgerEntry>>())
}
