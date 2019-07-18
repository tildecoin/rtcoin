//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

mod crypt;
mod db;
mod user;

use user::*;

fn main() {
    let user = User::new("Bob Bobson");
    println!(" Adding new user:\n{}", user);
}
