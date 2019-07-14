// 
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

// Locals Only
mod error;
use error::TcoinError;

// Leaving the fields private to prevent
// some funny business with the balances
// or the passwords.
#[derive(Debug)]
pub struct User {
    created: String, // TODO: use the chrono crate for time stuff
    pass: Vec<u8>,
    balance: u32,
    last_login: String,
}

impl User {
    pub fn new() -> User {
        let pass: Vec<u8> = vec![0, 8];
        User {
            created: "Placeholder".to_string(),
            pass: pass,
            balance: 1000,
            last_login: "Placeholder".to_string(),
        }
    }

    pub fn balance(&self) -> u32 {
        self.balance
    }

    pub fn deposit(&mut self, dep: u32) -> Result<(), TcoinError> {
        if (dep + self.balance) > u32::max_value() {
            return Err(TcoinError::new("Deposit Overflow"));
        }
        self.balance += dep;
        return Ok(())
    }

    pub fn withdraw(&mut self, amt: u32) -> Result<(), TcoinError> {
        if self.balance < amt {
            return Err(TcoinError::new("Insufficient funds"));
        }
        self.balance -= amt;
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
