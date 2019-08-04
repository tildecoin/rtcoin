//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use crate::db::*;
use crate::query;

use std::{
    fs,
    sync::mpsc,
    thread,
};

// This test needs to be broken up
#[test]
fn worker_thread_spawn_send_recv_query_rows() {
    let path = "/tmp/rtcoinserver-test.db";
    let (worker_tx, pipe) = mpsc::channel::<Comm>();
    let db = DB::connect(path, "test".into(), pipe);

    assert!(fs::metadata(path).is_ok());

    let kind = Kind::Balance;
    let args: Vec<String> = vec!["Bob".into()];
    let (tx_case1, _rx_case1) = mpsc::channel::<Reply>();
    let comm = Comm::new(Some(kind), Some(args), Some(tx_case1));

    let stmt = "SELECT * FROM ledger WHERE Source = 'Bob'";
    let stmt = db.conn.prepare(stmt).unwrap();

    if let Err(_) = query::to_ledger_entry(stmt) {
        panic!("failure in query_to_ledger_rows()");
    }
        
    // Above, comm takes ownership of the previous
    // instances of kind and trans. Need to duplicate
    // to test bulk_query(). Also, Clone isn't implemented
    // on db::Comm yet.
    let kind = Kind::Query;
    let args: Vec<String> = vec!["src".into(), "Bob".into()];
    let (tx_case2, _rx_case2) = mpsc::channel::<Reply>();
    let comm2 = Comm::new(Some(kind), Some(args), Some(tx_case2));

    thread::spawn(move || {
        db.worker_thread();
    });
        
    worker_tx.send(comm).unwrap();
    worker_tx.send(comm2).unwrap();

    // the worker passes the comm packet to bulk_query(),
    // which hands it off to serialize_rows() before sending
    // it back down the channel to be received here.
    //rx_case1.recv().unwrap();
    //rx_case2.recv().unwrap();

    worker_tx.send(Comm::new(Some(Kind::Disconnect), None, None)).unwrap();
    
    if fs::metadata(path).is_ok() {
        fs::remove_file(path).unwrap();
    }
}

#[test]
fn comm_kind() {
    let (tx, _) = mpsc::channel::<Reply>();
    let kind = Kind::Query;
    let args: Vec<String> = vec!["Source".into(),"Bob".into()];
    let comm = Comm::new(Some(kind), Some(args), Some(tx));

    match comm.kind() {
        Kind::Query => { },
        _ => panic!("Incorrect Kind"),
    }

    let arg1 = comm.args()[0].clone();
    let arg2 = comm.args()[1].clone();
    match &arg1[..] {
        "Source" => { },
        _ => panic!("Incorrect arguments"),
    }
    match &arg2[..] {
        "Bob" => { },
        _ => panic!("Incorrect arguments"),
    }
}
