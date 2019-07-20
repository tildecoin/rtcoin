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

use crate::db::DB;

type Aes256Cbc = Cbc<Aes256, Pkcs7>;
pub fn crypt() {
    let key = vec!["000102030405060708090a0b0c0d0e0f"];
    let iv = vec!["f0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"];
    let text = b"Dog feet smell like tortilla chips";

    // encrypt
    //let cipher = Aes256Cbc::new_varkey(&key).unwrap();
    //let ciphertext = cipher.encrypt_vec(text);

    // now to decrypt
    //let cipher = Aes256Cbc::new_var(&key, &iv).unwrap();
    //let text = cipher.decrypt_vec(&ciphertext).unwrap();
}

type HmacSha256 = Hmac<Sha256>;
pub fn auth() {
    let mut mac = HmacSha256::new_varkey(b"dog feet").expect("Something went wrong");

    mac.input(b"I have a secret about dog feet");

    // constant time equality check
    let res = mac.result();
    //let code_bytes = result.code();

    let mut mac = HmacSha256::new_varkey(b"dog feet").expect("Oh noez");

    mac.input(b"some message");

    //mac.verify(&code_bytes).unwrap();
}
