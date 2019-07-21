//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

// TODO: Implement symmetric encryption of the rtcoin
// ledger database. What's currently present here are
// various functions I pulled out of the documentation
// so I can remember what to use later, when I get to
// this. I'll handle this once I get the rest of the
// database's interaction sorted.

use std::{
    fs,
};

use aes_soft::Aes256;
use block_modes::{
    BlockMode, 
    block_padding::Pkcs7,
    Cbc,
};

use hmac::{
    Hmac, 
    Mac,
};
use sha2::Sha256;

use crate::{
    db,
    db::DB,
};

type Aes256Cbc = Cbc<Aes256, Pkcs7>;
pub fn crypt() {
    let key = b"000102030405060708090a0b0c0d0e0f";
    let iv = b"f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff";

    let db = fs::read(db::PATH).unwrap();
    // encrypt
    let cipher = Aes256Cbc::new_var(&key[..], &iv[..]).unwrap();
    let ciphertext = cipher.encrypt_vec(&db);

    fs::write(db::PATH, &ciphertext).unwrap();
}

pub fn decrypt() {
    let key = b"000102030405060708090a0b0c0d0e0f";
    let iv = b"f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff";

    let db = fs::read(db::PATH).unwrap();

    let cipher = Aes256Cbc::new_var(&key[..], &iv[..]).unwrap();
    let db = cipher.decrypt_vec(&db).unwrap();

    fs::write(db::PATH, &db).unwrap();
}

type HmacSha256 = Hmac<Sha256>;
pub fn auth() {
    let mut mac = HmacSha256::new_varkey(b"dog feet").expect("Something went wrong");

    mac.input(b"dog feet smell like tortilla chips");

    // constant time equality check
    let res = mac.result();
    //let code_bytes = result.code();

    let mut mac = HmacSha256::new_varkey(b"dog feet").expect("Oh noez");

    mac.input(b"some message");

    //mac.verify(&code_bytes).unwrap();
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[ignore]
    fn encryption_decryption() {
        let before = fs::read("../local/rtcoinledger.db").unwrap();
        crypt();
        decrypt();
        let after = fs::read("../local/rtcoinledger.db").unwrap();

        assert_eq!(before, after);
    }
}