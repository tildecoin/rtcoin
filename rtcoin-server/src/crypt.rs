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

use hmac::{
    Hmac, 
    Mac,
};

use rand::{
    RngCore,
    rngs::OsRng,
};

use crypto::{
    aes,
    blockmodes,
    buffer,
    buffer::{
        ReadBuffer,
        WriteBuffer,
        BufferResult,
    },
    symmetriccipher,
};

use sha2::Sha256;

use zeroize::Zeroize;

use crate::{
    db,
    db::DB,
};

/*
pub fn crypt(key: &[u8], iv: &[u8], db_path: &str) {
    let db = fs::read(db_path).unwrap();

    let mut encryptor = aes::cbc_encryptor(
        aes::KeySize::KeySize256,
        key,
        iv,
        blockmodes::PkcsPadding);

    let mut output = Vec::<u8>::new();
    let mut read_buf = buffer::RefReadBuffer::new(&db);
    let mut buf = [0; 4096];
    let mut write_buf = buffer::RefWriteBuffer::new(&mut buf);

    loop {
        let res = encryptor.encrypt(&mut read_buf, &mut write_buf, true).unwrap();
        output.extend(write_buf.take_read_buffer().take_remaining().iter().map(|&i| i));
        match res {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }
    
    fs::write(db_path, &output).unwrap();
}

pub fn decrypt(key: &[u8], iv: &[u8], db_path: &str) {
    let db = fs::read(db_path).unwrap();

    let mut decryptor = aes::cbc_decryptor(
        aes::KeySize::KeySize256,
        key,
        iv,
        blockmodes::PkcsPadding);

    let mut output = Vec::<u8>::new();
    let mut read_buf = buffer::RefReadBuffer::new(&db);
    let mut buf = [0; 4096];
    let mut write_buf = buffer::RefWriteBuffer::new(&mut buf);

    loop {
        let res = decryptor.decrypt(&mut read_buf, &mut write_buf, true).unwrap();
        output.extend(write_buf.take_read_buffer().take_remaining().iter().map(|&i| i));
        match res {
            BufferResult::BufferUnderflow => break,
            BufferResult::BufferOverflow => { }
        }
    }

    fs::write(db_path, &output).unwrap();
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
    const TESTDBPATH: &str = "/tmp/rtcoinledger.enc";
    #[test]
    #[ignore]
    fn encryption_decryption() {
        let mut key: [u8; 32] = [0; 32];
        let mut iv: [u8; 16] = [0; 16];
        OsRng.fill_bytes(&mut key);
        OsRng.fill_bytes(&mut iv);
        
        let mut data: [u8; 32] = [0; 32];
        if fs::metadata(TESTDBPATH).is_err() {
            OsRng.fill_bytes(&mut data);
            fs::write(TESTDBPATH, &data).unwrap();
        }

        let before = fs::read(TESTDBPATH).unwrap_or(data.to_vec());
        crypt(&key, &iv, TESTDBPATH);
        decrypt(&key, &iv, TESTDBPATH);
        let after = fs::read(TESTDBPATH).unwrap();

        key.zeroize();
        iv.zeroize();
        assert_eq!(before, after);
    }
}
*/