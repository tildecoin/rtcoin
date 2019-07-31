//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use crate::user::*;

#[test]
fn create_user_check_name_and_balance() {
    let user = User::new("Bob Bobson");

    let name = user.name();
    let bal = user.balance();
    let bal_str = user.balance_as_string();

    assert_eq!(name, "Bob Bobson");
    assert_eq!(bal, 1000.0);
    assert_eq!(bal_str, "1000");
}
