//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    fmt,
};

use chrono::prelude::*;
use log::{
    error,
};

use crate::{
    db,
};

#[derive(Debug)]
pub struct User {
    name: String,
    created: chrono::DateTime<Utc>,
    pass: Vec<u8>,
    balance: f64,
    messages: Vec<String>,
    last_login: chrono::DateTime<Utc>,
}

#[derive(Debug)]
pub enum InitCode {
    Success,
    Fail(String),
}

// The std::fmt::Display trait, so a User
// can be passed to a print!() macro. Will
// be formatted like so:
//  Name: Bob Bobson
//  Balance: 1000.0 tcoin
//  Last Login: chrono::DateTime<Utc>
//  Account Age: chrono::OldDuration
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let since = Utc::now().signed_duration_since(self.created);

        write!(
            f,
            " Name: {}\n Balance: {} tcoin\n Last Login: {}\n Account Age: {}",
            self.name(),
            self.balance_as_string(),
            self.last_login.to_string(),
            since.to_string()
        )
    }
}

impl User {
    pub fn new(name: &str) -> User {
        let pass: Vec<u8> = vec![1, 0, 1, 0, 1];
        let name = name.to_string();
        let now = Utc::now();

        User {
            name,
            created: now,
            pass,
            balance: 1000.0,
            messages: Vec::with_capacity(10),
            last_login: now,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    pub fn balance_as_string(&self) -> String {
        format!("{}", self.balance)
    }

}

pub fn register(_comm: &db::Comm) {
    // placeholder
    error!("{:#?}", InitCode::Fail(String::from("Unspecified Error")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_user_check_name_and_balance() {
        let user = User::new("Bob Bobson");

        assert_eq!(user.name(), "Bob Bobson");
        assert_eq!(user.balance(), 1000.0);

        let bal_str = user.balance_as_string();

        assert_eq!(bal_str, "1000");
    }
}
