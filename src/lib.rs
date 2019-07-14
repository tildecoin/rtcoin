// 
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::fmt;
use chrono::prelude::*;

// Locals Only
mod error;
use error::TcoinError;

// Leaving the fields private to prevent
// some funny business with the balances
// or the passwords.
#[derive(Debug)]
pub struct User {
    name: String,
    created: chrono::DateTime<Utc>,
    pass: Vec<u8>,
    balance: f64,
    last_login: chrono::DateTime<Utc>,
}

// The std::fmt::Display trait, so a User
// can be passed to a print!() macro. Will
// be formatted like so:
//  Name: Bob Bobson
//  Balance: 1000 tcoin
//  Last Login: chrono::DateTime<Utc>
//  Account Age: chrono::OldDuration
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let since = Utc::now().signed_duration_since(self.created);
        write!(f, " Name: {}\n Balance: {} tcoin\n Last Login: {}\n Account Age: {}", 
               self.name, 
               self.balance, 
               self.last_login.to_string(),
               since.to_string())
    }
}

impl User {
    pub fn new(name: &str) -> User {
        let pass: Vec<u8> = vec![0, 8];
        let name = name.to_string();
        User {
            name,
            created: Utc::now(),
            pass: pass,
            balance: 1000.0,
            last_login: Utc::now(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }

    // Currently, just check if the deposit will
    // overflow the u32 balance field.
    pub fn deposit(&mut self, dep: f64) -> Result<(), TcoinError> {
        if (std::f64::MAX - self.balance) < dep {
            return Err(TcoinError::new("Deposit Overflow"));
        }

        self.balance += dep;
        Ok(())
    }

    // Currently, just check if the withdrawal
    // results in a negative balance. As I'm
    // using unsigned ints, a negative balance
    // isn't allowed. Plus, a currency simulation
    // with negative balances could get a bit
    // unwieldy.
    pub fn withdraw(&mut self, amt: f64) -> Result<(), TcoinError> {
        if self.balance < amt {
            return Err(TcoinError::new("Insufficient funds"));
        }

        self.balance -= amt;
        Ok(())
    }

    // The least functional of the stub methods.
    // Checks for available balance and a deposit
    // overflow, then just prints the message to
    // stdout.
    pub fn send(&mut self, other: &mut User, amount: f64, msg: &str) -> Result<(), TcoinError> {
        if self.balance < amount {
            return Err(TcoinError::new("Insufficient funds"));
        } else if std::f64::MAX - other.balance < amount {
            return Err(TcoinError::new("Deposit Overflow"));
        }

        self.withdraw(amount)?;
        other.deposit(amount)?;
        
        println!("A message to you, Rudy:\n\t{}", msg);
        eprintln!("More logic to be added");
        Ok(())
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
    }

    #[test]
    fn deposit_test() {
        let mut user = User::new("Bob Bobson");
        
        user.deposit(100.0)
            .expect("Failed to deposit 100.0");

        
        assert_eq!(user.balance(), 1100.0);
    }

    #[test]
    fn withdrawal_test() {
        let mut user = User::new("Bob Bobson");
        
        user.withdraw(100.0)
            .expect("Failed to withdraw 100.0");
        
        assert_eq!(user.balance(), 900.0);
    }

    #[test]
    fn send_test() {
        let mut user1 = User::new("Bob Bobson");
        let mut user2 = User::new("Foo Barrington");

        user1.send(&mut user2, 100.0, "Henlo fren!");

        assert_eq!(user1.balance(), 900.0);
        assert_eq!(user2.balance(), 1100.0);
    }
}
