mod user;
use user::*;

fn main() {
    let user = User::new("Bob Bobson");
    println!(" Adding new user:\n{}", user);
}
