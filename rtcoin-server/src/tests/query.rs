//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::sync::mpsc;
use crate::query::*;
use crate::db;

#[test]
fn expect_no_rows() {
    let (dbtx, dbrx) = mpsc::channel::<db::Comm>();
    let db = db::DB::connect(db::PATH, String::from("test"), dbrx);
    let (commtx, commrx) = mpsc::channel::<db::Reply>();
    let comm = db::Comm::new(
            Some(db::Kind::Whoami), 
            Some(vec!["user".into(), "BobBobson".into()]), 
            Some(commtx)
        );

    whoami(comm, &db.conn);
    let resp = commrx.recv().unwrap();
    let resp = format!("{:?}", resp);

    assert!(resp.contains("Query Error"));
    dbtx.send(db::Comm::new(Some(db::Kind::Disconnect), None, None)).unwrap();
}
