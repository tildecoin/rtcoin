//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    fmt,
};

// Used for quickly serializing an error into bytes
// (or string) so that it may be sent across the socket. 
// Current error codes:
//      01: Worker error
//      02: Could not parse request as JSON
//      03: Invalid request
//      04: Query Error
//      05: Channel Send Error
#[derive(Debug)]
pub struct Resp {
    code: u32,
    kind: String,
    details: String,
}

// This also implements ToString implicitly
impl fmt::Display for Resp {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            " Code: {}\n Kind: {}\n Details: {}",
            self.code(),
            self.kind(),
            self.details(),
        )
    }
}

impl Resp {
    pub fn new(code: u32, err: &str, details: &str) -> Resp {
        let kind = err.to_string();
        let details = details.to_string();
        Resp {
            code,
            kind,
            details,
        }
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        format!("{:#?}", self)
            .as_bytes()
            .to_owned()
    }
    pub fn code(&self) -> u32 {
        self.code
    }
    pub fn kind(&self) -> String {
        self.kind.clone()
    }
    pub fn details(&self) -> String {
        self.details.clone()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn basic_resp_test() {
        let err = Resp::new(05, "Test Error", "Some stuff went wrong");
        let code = err.code();
        let kind = err.kind();
        let details = err.details();

        assert_eq!(code, 05);
        assert_eq!(kind, "Test Error");
        assert_eq!(details, "Some stuff went wrong");
    }
}