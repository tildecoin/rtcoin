//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

// This file currently only contains placeholder
// functions and structures for the sake of
// organization. None of the following will be
// making it into rtcoin -- it will all be rewritten
// with actual functionality once I start on user
// methods.

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

// Leaving the fields private to prevent
// some funny business with the balances
// or the passwords.
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

// Most of the following is TERRIBLE.
// I super pinky promise I'm going to rewrite
// it soon.
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

    // Make sure the deposit is positive.
    pub fn deposit(&mut self, dep: f64) -> Result<(), &'static str> {
        if dep < 0.0 {
            return Err("Negative Deposit");
        }

        self.balance += dep;
        Ok(())
    }

    pub fn withdraw(&mut self, amt: f64) -> Result<(), &'static str> {
        if self.balance < amt {
            return Err("Insufficient funds");
        } else if amt < 0.0 {
            return Err("Negative Withdrawal");
        }

        self.balance -= amt;
        Ok(())
    }

    pub fn send(&mut self, other: &mut User, amount: f64, msg: &str) -> Result<(), &'static str> {
        self.withdraw(amount)?;
        other.deposit(amount)?;

        other.messages.push(msg.to_string());
        Ok(())
    }

    pub fn messages(&self) -> Vec<String> {
        self.messages.clone()
    }

    pub fn append_messages(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
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

    #[test]
    fn deposit() {
        let mut user = User::new("Bob Bobson");
        user.deposit(100.0).expect("Failed to deposit 100.0");

        assert_eq!(user.balance(), 1100.0);
        assert_eq!(&user.balance_as_string()[..], "1100");
    }

    #[test]
    #[should_panic]
    fn deposit_negative() {
        let dep = -32.3;
        let mut user = User::new("Bob Bobson");

        match user.deposit(dep) {
            Err(err) => panic!(err),
            Ok(_) => println!("Something went wrong, test didn't panic"),
        }
    }

    #[test]
    fn withdrawal() {
        let mut user = User::new("Bob Bobson");
        user.withdraw(100.0).expect("Failed to withdraw 100.0");

        assert_eq!(user.balance(), 900.0);
        assert_eq!(&user.balance_as_string()[..], "900");
    }

    #[test]
    #[should_panic]
    fn withdrawal_nsf() {
        let mut user = User::new("Bob Bobson");
        user.withdraw(10000.0).unwrap();
    }

    #[test]
    fn send() {
        let mut user1 = User::new("Bob Bobson");
        let mut user2 = User::new("Foo Barrington");

        user1
            .send(&mut user2, 100.0, "Henlo fren!")
            .expect("Failed to send 100.0");

        assert_eq!(user1.balance(), 900.0);
        assert_eq!(user2.balance(), 1100.0);
        assert_eq!(user2.messages[0], "Henlo fren!");

        user1
            .send(&mut user2, 23.5, "Have some moar, fren!")
            .expect("Failed to send 23.5");

        assert_eq!(user1.balance(), 876.5);
        assert_eq!(user2.balance(), 1123.5);
        assert_eq!(user2.messages[0], "Henlo fren!");
        assert_eq!(user2.messages[1], "Have some moar, fren!");
    }

    #[test]
    fn append_messages() {
        let mut user = User::new("Bob Bobson");
        let old_len = user.messages().len();

        user.append_messages("Testing 1 2 3");
        user.append_messages("Testing 3 4 5");

        let new_len = user.messages().len();

        assert_ne!(old_len, new_len);
    }

    #[test]
    fn user_messages_list() {
        let mut user = User::new("Bob Bobson");
        user.append_messages("test");

        let out = user.messages();
        assert_eq!(out[0], "test");
    }
}
