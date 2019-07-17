//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use chrono::prelude::*;
use ryu;
use std::fmt;

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
        let pass: Vec<u8> = vec![0, 8];
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

    pub fn name(&self) -> String {
        self.name.clone()
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    // User.balance_as_string() uses the Ryu library
    // to format its output. It is significantly faster
    // than the stdlib implementation of converting an
    // f64 to a string.
    pub fn balance_as_string(&self) -> String {
        let mut buf = ryu::Buffer::new();
        buf.format(self.balance).to_string() // &str -> String
    }

    // Make sure the deposit is positive.
    pub fn deposit(&mut self, dep: f64) -> Result<(), &'static str> {
        if dep < 0.0 {
            return Err("Negative Deposit");
        }

        self.balance += dep;
        Ok(())
    }

    // Check if the withdrawal results in a negative balance.
    // A currency simulation with negative balances could get 
    // a bit unwieldy.
    // Also make sure we're withdrawing a positive number.
    pub fn withdraw(&mut self, amt: f64) -> Result<(), &'static str> {
        if self.balance < amt {
            return Err("Insufficient funds");
        } else if amt < 0.0 {
            return Err("Negative Withdrawal");
        }

        self.balance -= amt;
        Ok(())
    }

    // Acts as a wrapper for withdraw/deposit. Lets any errors 
    // with those bubble up, and appends the message to the 
    // associated User obj.
    pub fn send(&mut self, other: &mut User, amount: f64, msg: &str) -> Result<(), &'static str> {
        self.withdraw(amount)?;
        other.deposit(amount)?;

        other.messages.push(msg.to_string());

        // TODO: remove this debug print after I'm
        // certain the messages are handled properly.
        println!("A message to you, Rudy:\n\t{}", msg);
        Ok(())
    }

    pub fn messages(&self) -> Vec<String> {
        self.messages.clone()
    }

    pub fn append_messages(&mut self, msg: &str) {
        self.messages.push(msg.to_string());
    }
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

        assert_eq!(&bal_str[..], "1000.0");
    }

    #[test]
    fn deposit() {
        let mut user = User::new("Bob Bobson");

        user.deposit(100.0).expect("Failed to deposit 100.0");

        assert_eq!(user.balance(), 1100.0);
        assert_eq!(&user.balance_as_string()[..], "1100.0");
    }

    #[test]
    #[should_panic]
    fn deposit_negative() {
        let dep = -32.3;
        let mut user = User::new("Bob Bobson");

        // TODO: This isn't panicking like it should
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
        assert_eq!(&user.balance_as_string()[..], "900.0");
    }

    #[test]
    #[should_panic]
    fn withdrawal_nsf() {
        let mut user = User::new("Bob Bobson");

        // TODO: This isn't panicking like it should
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
}
