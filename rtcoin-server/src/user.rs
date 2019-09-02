//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::fmt;

use chrono::prelude::*;
use log;
use zeroize::Zeroize;

use crate::db;

#[derive(Debug)]
pub struct User {
    name: String,
    created: String,
    pass: String,
    balance: f64,
    messages: Vec<String>,
    last_login: String,
}

type AuthResult<T> = std::result::Result<T, String>;

// The std::fmt::Display trait, so a User
// can be passed to a print!() macro. Will
// be formatted like so:
//  Name: Bob Bobson
//  Balance: 1000.0 tcoin
//  Last Login: String (RFC2822)
//  Account Age: String (Weeks, Days, Hours)
impl fmt::Display for User {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let created = DateTime::parse_from_rfc2822(&self.created).unwrap();
        let since = Utc::now().signed_duration_since(created);
        let acct_age = format!(
            "{} weeks, {} days, {} hours",
            since.num_weeks(),
            since.num_days(),
            since.num_hours()
        );

        write!(
            f,
            " Name: {}\n Balance: {} tcoin\n Last Login: {}\n Account Age: {}",
            self.name(),
            self.balance_as_string(),
            self.last_login,
            acct_age
        )
    }
}

impl User {
    pub fn new(name: &str) -> User {
        let pass = String::new();
        let name = name.to_string();
        let now = Utc::now().to_rfc2822();

        User {
            name,
            created: now.clone(),
            pass,
            balance: 1000.0,
            messages: Vec::new(),
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
}

pub fn register(comm: db::Comm, db: &rusqlite::Connection) {
    let thetime = chrono::Utc::now().to_rfc2822();
    let tx = match &comm.origin {
        Some(t) => t,
        None => return,
    };
    let query = format!("INSERT INTO users (name, pass, pubkey, balance, created, last_login) VALUES (:name, :pass, :pubkey, :balance, :created, :last_login)");
    let args = match &comm.args {
        Some(val) => val,
        None => return,
    };
    let user = args[0].clone();
    let mut pass = args[1].clone();
    let pubkey = args[2].clone();

    match check_pass(&pass) {
        Err(err) => match tx.send(db::Reply::Error(err)) {
            Ok(_) => {}
            Err(err) => log::warn!("{:?}", err),
        },
        Ok(_) => {}
    }

    let mut stmt = match db.prepare(&query) {
        Ok(st) => st,
        Err(err) => {
            let err = format!("Internal Error: {:?}", err);
            match tx.send(db::Reply::Error(err)) {
                Ok(_) => {}
                Err(err) => log::warn!("{:?}", err),
            }
            return;
        }
    };

    match stmt.execute_named(&[
        (":name", &user),
        (":pass", &pass),
        (":pubkey", &pubkey),
        (":balance", &1000.0),
        (":created", &thetime),
        (":last_login", &thetime),
    ]) {
        Ok(_) => {}
        Err(err) => {
            let err = format!("Internal Error: {:?}", err);
            match tx.send(db::Reply::Error(err)) {
                Ok(_) => {}
                Err(err) => log::warn!("{:?}", err),
            }
        }
    }

    log::info!("Registration Successful: {}", user);
    match tx.send(db::Reply::Info("Registration Successful".into())) {
        Ok(_) => {}
        Err(err) => log::warn!("{:?}", err),
    }

    pass.zeroize();
}

fn check_pass(pass: &str) -> AuthResult<()> {
    let mut pass = pass.to_string();

    if pass.len() < 12 {
        return Err("Password too short".into());
    }

    pass.zeroize();
    Ok(())
}
