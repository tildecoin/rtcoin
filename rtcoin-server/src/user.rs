//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::fmt;

use chrono::prelude::*;
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

type AuthResult<T> = std::result::Result<T, Box<dyn std::error::Error>>;

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
            self.balance(),
            self.last_login,
            acct_age
        )
    }
}

impl User {
    pub fn new(name: &str) -> Self {
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

    pub fn set_pass(&mut self, pass: &str) {
        self.pass = pass.into();
    }

    pub fn scrub_pass(&mut self) {
        self.pass.zeroize();
    }

    pub fn get_ctime(&self) -> String {
        self.created.clone()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn balance(&self) -> f64 {
        self.balance
    }
}

// Accepts a registration request and adds a new user to the database
pub fn register(comm: db::Comm, db: &rusqlite::Connection) {
    let tx = match &comm.origin {
        Some(t) => t,
        None => return,
    };
    let query = "INSERT INTO users (name, pass, pubkey, balance, created, last_login) VALUES (:name, :pass, :pubkey, :balance, :created, :last_login)";
    let args = match &comm.args {
        Some(val) => val,
        None => return,
    };
    let mut user = User::new(&args[0]);
    let pass = args[1].clone();
    let pubkey = args[2].clone();

    if let Err(err) = check_pass(&pass) {
        if let Err(err) = tx.send(db::Reply::Error(format!("{:?}", err))) {
            log::warn!("{:?}", err);
        }
        return;
    }

    let mut pass = match bcrypt::hash(&pass, 12) {
        Ok(hash) => hash,
        Err(err) => {
            log::error!("Failed to hash password: {:?}", err);
            if let Err(err) = tx.send(db::Reply::Error("Internal Error: Password Hashing".into())) {
                log::error!("Failed to send reply to client: {:?}", err);
            }
            return;
        }
    };

    user.set_pass(&pass);

    let mut stmt = match db.prepare(query) {
        Ok(st) => st,
        Err(err) => {
            let err = format!("Internal Error: {:?}", err);
            if let Err(err) = tx.send(db::Reply::Error(err)) {
                log::warn!("{:?}", err);
            }
            return;
        }
    };

    if let Err(err) = stmt.execute_named(&[
        (":name", &user.name()),
        (":pass", &pass),
        (":pubkey", &pubkey),
        (":balance", &user.balance()),
        (":created", &user.get_ctime()),
        (":last_login", &user.get_ctime()),
    ]) {
        let err = format!("Internal Error: {:?}", err);
        if let Err(err) = tx.send(db::Reply::Error(err)) {
            log::warn!("{:?}", err);
        }
        return;
    }

    log::info!("Registration Successful: {}", user);
    if let Err(err) = tx.send(db::Reply::Info("Registration Successful".into())) {
        log::warn!("{:?}", err);
    }

    pass.zeroize();
    user.scrub_pass();
}

// Right now this just checks for a minimum password length
pub fn check_pass(pass: &str) -> AuthResult<()> {
    if pass.len() < 12 {
        return Err("Password too short".into());
    }
    Ok(())
}

// Change a username. This is incomplete. I'll need to address the
// historical transactions associated with the old user somehow.
pub fn rename(comm: db::Comm, db: &rusqlite::Connection) {
    let mut args = match comm.args {
        Some(val) => val,
        None => {
            log::error!("Received none value from client comm");
            return;
        }
    };
    let old_user = args[0].clone();
    let new_user = args[1].clone();
    let mut pass = args[2].clone();
    args[2].zeroize();

    if auth(&old_user, &pass, &db) {
        log::info!(
            "User {} authenticated for: username change to {}",
            old_user,
            new_user
        );
    } else {
        log::error!("Auth failed for user {}", old_user);
        return;
    }

    let stmt = "UPDATE users SET name = :new_user WHERE name = :old_user";
    let mut stmt = match db.prepare(stmt) {
        Ok(s) => s,
        Err(err) => {
            log::error!("Failed to prepare update username statement: {:?}", err);
            return;
        }
    };

    match stmt.execute_named(&[(":new_user", &new_user), (":old_user", &old_user)]) {
        Ok(_) => {
            if let Some(tx) = comm.origin {
                if let Err(err) = tx.send(db::Reply::Info("Username update successful".into())) {
                    log::error!("Failed to send success message: {:?}", err);
                    return;
                }
            }
        }
        Err(err) => {
            log::error!("Failed to execute update username statement: {:?}", err);
            return;
        }
    }

    pass.zeroize();
}

// Authenticates a user's provided password hash against the
// hash stored in the database.
pub fn auth(user: &str, pass: &str, db: &rusqlite::Connection) -> bool {
    let pass_verify_stmt = "SELECT pass FROM users WHERE name = :user";

    let mut stored_pass: String =
        match db.query_row_named(pass_verify_stmt, &[(":user", &user)], |row| {
            match row.get::<usize, String>(0) {
                Ok(s) => Ok(s),
                Err(err) => {
                    log::error!("Failed to get stored password hash for {}: {:?}", user, err);
                    Ok(String::new())
                }
            }
        }) {
            Ok(val) => val,
            Err(_) => return false,
        };

    let mut pass_bytes = pass.bytes().collect::<Vec<u8>>();

    match bcrypt::verify(&pass_bytes, &stored_pass) {
        Ok(boolean) => {
            pass_bytes.zeroize();
            stored_pass.zeroize();
            boolean
        }
        Err(err) => {
            pass_bytes.zeroize();
            stored_pass.zeroize();
            log::error!("Failed to verify password hash: {:?}", err);
            false
        }
    }
}

// TODO: send tildecoin from one user to another
pub fn send(comm: db::Comm, _db: &rusqlite::Connection) {
    let _args = if let Some(args) = comm.args {
        args
    } else {
        vec![]
    };

    unimplemented!();
}

// TODO: retrieve the balance for a user
pub fn balance(comm: db::Comm, _db: &rusqlite::Connection) {
    let _args = match comm.args {
        Some(val) => val,
        None => {
            log::error!("Received None for Comm args");
            return;
        }
    };

    unimplemented!();
}
