//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use crate::db;
use crate::query::*;
use std::fs;
use std::sync::mpsc;

#[test]
fn expect_no_rows() {
    // Since cargo executes tests concurrently,
    // it looks like I'll have to use separate
    // temp databases for tests.
    let path = "/tmp/rtcoinserver-query-test.db";

    let (dbtx, dbrx) = mpsc::channel::<db::Comm>();
    let db = db::DB::connect(path, "test".into(), dbrx);
    let (commtx, commrx) = mpsc::channel::<db::Reply>();

    let comm = db::Comm::new(
        Some(db::Kind::Whoami),
        Some(vec!["user".into(), "BobBobson".into()]),
        Some(commtx),
    );

    whoami(comm, &db.conn);
    let resp = commrx.recv().unwrap();
    let resp = format!("{:?}", resp);

    assert!(resp.contains("Query Error"));
    dbtx.send(db::Comm::new(Some(db::Kind::Disconnect), None, None))
        .unwrap();

    fs::remove_file(path).unwrap();
}
