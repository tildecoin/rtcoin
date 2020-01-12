//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

extern crate test;

use crate::err;

#[test]
fn msg_resp() {
    let resp = err::Resp::new(00, "Test Error", "Some stuff went wrong");
    let code = resp.code();
    let kind = resp.kind();
    let details = resp.details();
    assert_eq!(code, 00);
    assert_eq!(kind, "Test Error");
    assert_eq!(details, "Some stuff went wrong");
}

#[bench]
fn msg_resp_bench(b: &mut test::Bencher) {
    b.iter(msg_resp)
}

#[test]
#[should_panic]
fn log_then_panic() {
    let error = err::Resp::new(14, "Test Error", "Some stuff");
    let code = error.code();
    assert_eq!(14, code);

    err::log_then_panic("Test Panic", error);
}
