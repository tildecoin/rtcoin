// 
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

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
    balance: u32,
    last_login: chrono::DateTime<Utc>,
}

impl User {
    pub fn new(name: String) -> User {
        let pass: Vec<u8> = vec![0, 8];
        User {
            name,
            created: Utc::now(),
            pass: pass,
            balance: 1000,
            last_login: Utc::now(),
        }
    }

    pub fn balance(&self) -> u32 {
        self.balance
    }

    pub fn deposit(&mut self, dep: u32) -> Result<(), TcoinError> {
        if (u32::max_value() - self.balance) < dep {
            return Err(TcoinError::new("Deposit Overflow"));
        }

        self.balance += dep;
        Ok(())
    }

    pub fn withdraw(&mut self, amt: u32) -> Result<(), TcoinError> {
        if self.balance < amt {
            return Err(TcoinError::new("Insufficient funds"));
        }

        self.balance -= amt;
        Ok(())
    }

    pub fn send(&mut self, other: &mut User, amount: u32, msg: &str) -> Result<(), TcoinError> {
        if self.balance < amount {
            return Err(TcoinError::new("Insufficient funds"));
        } else if u32::max_value() - other.balance < amount {
            return Err(TcoinError::new("Deposit Overflow"));
        }
        
        println!("A message to you, Rudy:\n\t{}", msg);
        eprintln!("More logic to be added");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
