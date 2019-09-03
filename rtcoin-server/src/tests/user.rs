//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

extern crate test;

use std::sync::mpsc;

use crate::db;
use crate::user::*;

#[test]
fn create_user_check_name_and_balance() {
    let user = User::new("Bob Bobson");

    let name = user.name();
    let bal = user.balance();
    let bal_str = format!("{}", user.balance());

    assert_eq!(name, "Bob Bobson");
    assert_eq!(bal, 1000.0);
    assert_eq!(bal_str, "1000");
}

#[bench]
fn bench_user_make(b: &mut test::Bencher) {
    b.iter(|| create_user_check_name_and_balance())
}

#[test]
#[should_panic]
fn test_check_pass_too_short() {
    check_pass("2short").unwrap();
}

#[test]
fn test_check_pass_ok() {
    check_pass("thispasswordislong").unwrap();
}

#[bench]
fn bench_check_pass(b: &mut test::Bencher) {
    b.iter(|| check_pass("somepasswordhere"))
}

#[bench]
fn bench_register(b: &mut test::Bencher) {
    let (_, rx) = mpsc::channel::<db::Comm>();
    let db = db::DB::connect(db::PATH, "password".into(), rx);
    let (otx, _) = mpsc::channel::<db::Reply>();
    let comm = db::Comm {
        kind: Some(db::Kind::Register),
        args: Some(vec![
            "testuser".into(),
            "testpassword".into(),
            "pubkey".into(),
        ]),
        origin: Some(otx),
    };
    b.iter(|| register(comm.clone(), &db.conn))
}
