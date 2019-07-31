//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

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