//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::error::Error;
use std::fmt;

// The following struct and its three implementation
// blocks are for a local error type. This allows the
// return of Result<T, TcoinError> to signal a failure.
#[derive(Debug)]
pub struct TcoinError {
    details: String,
}

impl TcoinError {
    pub fn new(msg: &str) -> TcoinError {
        TcoinError {
            details: msg.to_string(),
        }
    }
}

impl fmt::Display for TcoinError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.details)
    }
}

impl Error for TcoinError {
    fn description(&self) -> &str {
        &self.details
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn errors() {
        let err = TcoinError::new("some error");
        assert_eq!(err.description(), "some error");
    }
}
