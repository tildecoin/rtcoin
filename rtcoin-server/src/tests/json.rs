//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

extern crate test;

use std::sync::mpsc;

use crate::db;
use crate::json::*;

use serde_json::json;

#[test]
fn test_from_string() {
    let rhs = json!({
        "kind": "value",
        "args": "value",
    });

    let lhs_proto = "{ \"kind\": \"value\", \"args\": \"value\" }";
    let lhs = from_str(lhs_proto, None).unwrap();

    assert_eq!(lhs, rhs);

    if let Some(val) = from_str("foo BAR invalid json", None) {
        panic!("That was invalid, why did it pass? {}", val);
    }

    assert_eq!(from_str("MORE INVALID json", None), None);
}

#[bench]
fn bench_from_string(b: &mut test::Bencher) {
    b.iter(|| test_from_string())
}

#[test]
fn test_json_to_comm() {
    let test_data = json!({
        "kind":        "Disconnect",
        "args":        "Source Foo"
    });

    let (tx, _) = mpsc::channel::<db::Reply>();
    let tx2 = tx.clone();
    let tx3 = tx.clone();

    let case = if let Some(val) = to_comm(&test_data, tx) {
        val
    } else {
        panic!("to_comm() failed: case 1");
    };

    match case.kind() {
        db::Kind::Disconnect => {}
        _ => panic!("Incorrect Kind: case 1"),
    }

    let test_data = json!({
        "kind":        "Send",
        "args":        "From Foo To Bob"
    });

    let case = if let Some(val) = to_comm(&test_data, tx2) {
        val
    } else {
        panic!("to_comm() failed: case 2");
    };

    match case.kind() {
        db::Kind::Send => {}
        _ => panic!("Incorrect Kind: case 2"),
    }

    let test_data = json!({
        "kind": "FOOBAR",
        "args": "some args here"
    });

    match to_comm(&test_data, tx3) {
        None => {}
        _ => panic!("Received some, expected none: case 3"),
    }
}

#[bench]
fn bench_json_to_comm(b: &mut test::Bencher) {
    b.iter(|| test_json_to_comm())
}
