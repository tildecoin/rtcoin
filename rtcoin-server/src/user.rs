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
    info,
};

use crate::{
    db,
};

#[derive(Debug)]
pub struct User {
    name: String,
    created: String,
    pass: String,
    balance: f64,
    messages: Vec<String>,
    last_login: String,
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
//  Last Login: String (RFC2822)
//  Account Age: String (Weeks, Days, Hours)
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let created = DateTime::parse_from_rfc2822(&self.created).unwrap();
        let since = Utc::now().signed_duration_since(created);
        let acct_age = format!(
                "{} weeks, {} days, {} hours", 
                since.num_weeks(),
                since.num_days(),
                since.num_hours()
            );

        write!(
            f,
            " Name: {}\n Balance: {} tcoin\n Last Login: {}\n Account Age: {}",
            self.name(),
            self.balance_as_string(),
            self.last_login,
            acct_age
        )
    }
}

impl User {
    pub fn new(name: &str) -> User {
        let pass = String::new();
        let name = name.to_string();
        let now = Utc::now().to_rfc2822();

        User {
            name,
            created: now.clone(),
            pass,
            balance: 1000.0,
            messages: Vec::new(),
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
    info!("{:#?}", InitCode::Success);
}