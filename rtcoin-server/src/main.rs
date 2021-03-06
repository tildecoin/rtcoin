//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

#![feature(test)]

use std::{error::Error, fs, os::unix::net::UnixListener, path::Path, process, sync::mpsc, thread};

use threadpool::ThreadPool;
use zeroize::Zeroize;

mod conn;
mod db;
mod err;
mod json;
mod logging;
mod query;
mod user;

#[cfg(test)]
mod tests;

use db::DB;

fn main() -> Result<(), Box<dyn Error>> {
    logging::init();
    log::info!("rtcoin-server is initializing.\n");

    eprintln!("\nrtcoin-server 0.1-dev");
    eprintln!("\nPlease enter the ledger password:");
    let mut db_key_in = rpassword::prompt_password_stderr("> ").unwrap_or_else(|err| {
        err::log_then_panic("Failed to read database password", err);
        panic!();
    });
    eprintln!();
    let db_key = db_key_in.trim().to_string();
    db_key_in.zeroize();

    eprintln!("Continuing startup process. See log file for details.");
    eprintln!();
    // Create communication channel to the ledger database, then
    // spawn the ledger worker to listen for query requests.
    log::info!("Starting ledger worker...");
    let (tx, rx) = mpsc::channel::<db::Comm>();
    thread::spawn(move || spawn_ledger_worker(db_key, rx));

    // If the socket exists already, remove it.
    let sock = Path::new(conn::SOCK);
    if fs::metadata(sock).is_ok() {
        log::warn!("Socket {} already exists.", conn::SOCK);
        fs::remove_file(sock)?;
    }

    // Handle SIGINT / ^C
    let ctrlc_tx = tx.clone();
    ctrlc::set_handler(move || {
        log::warn!("^C / SIGINT Caught. Cleaning up ...");
        if fs::metadata(sock).is_ok() {
            log::info!("Removing socket file");
            fs::remove_file(sock).unwrap();
        }

        log::info!("SIGINT: Sending disconnect signal to ledger worker queue");
        let (reply_tx, sigint_rx) = mpsc::channel::<db::Reply>();
        let db_disconnect_signal = db::Comm::new(Some(db::Kind::Disconnect), None, Some(reply_tx));

        if let Err(err) = ctrlc_tx.send(db_disconnect_signal) {
            log::error!(
                "SIGINT: Failed to send disconnect signal to ledger worker: {}",
                err
            );
        }
        // Block to allow database to close
        sigint_rx.recv().unwrap_or_else(|error| {
            log::warn!("{:?}", error);
            process::exit(1);
        });
        log::info!("¡Hasta luego!");
        process::exit(0);
    })
    .unwrap_or_else(|error| {
        err::log_then_panic("SIGINT Handler Initialization", error);
        panic!();
    });

    // Bind to the socket. Spawn a new connection
    // worker thread for each client connection.
    log::info!("Binding to socket: {}", conn::SOCK);
    spawn_for_connections(&sock, tx);

    // Tidy up
    fs::remove_file(sock)?;
    Ok(())
}

fn spawn_ledger_worker(mut db_key: String, rx: mpsc::Receiver<db::Comm>) {
    // This next call opens the actual database connection.
    // It also creates the tables if they don't yet exist.
    log::info!("Connecting to database: {}", db::PATH);
    let ledger = DB::connect(db::PATH, db_key.clone(), rx);
    db_key.zeroize();

    // Naming the thread helps with debugging. It will
    // show up in panics.
    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    log::info!("Starting ledger worker process...");
    let worker_thread = ledger_worker
        .spawn(move || {
            // once the worker_thread() method returns,
            // begin cleanup. so the whole process can exit.
            let disconnect_comm = ledger.worker_thread();
            match ledger.conn.close() {
                Err(err) => log::error!("Error closing database connection: {:?}", err),
                Ok(_) => log::info!("Database connection successfully closed"),
            }

            // Once we've closed the DB connection, let the
            // SIGINT thread know so it can kill the whole
            // process.
            if let Some(tx) = disconnect_comm.origin {
                tx.send(db::Reply::Data(String::new())).expect(
                    "When notifying SIGINT handler of DB connection close, something went wrong.",
                );
            }
        })
        .unwrap_or_else(|error| {
            err::log_then_panic("Ledger worker failed to spawn", error);
            panic!(); // otherwise rustc complains about return type
        });

    // Block execution until the thread we just
    // spawned returns.
    log::info!("Startup finished!");
    worker_thread.join().unwrap_or_else(|error| {
        err::log_then_panic("Ledger Worker", error);
        panic!() // otherwise rustc complains about return type
    });
}

fn spawn_for_connections(sock: &Path, tx: mpsc::Sender<db::Comm>) {
    let lstnr = UnixListener::bind(sock).unwrap_or_else(|error| {
        err::log_then_panic("Could not bind to socket", error);
        panic!();
    });

    // The thread pool will always allow at least
    // four simultaneous client connections. The
    // client connections will most likely not be
    // resource hogs.
    let thread_num = num_cpus::get() * 4;
    let pool = ThreadPool::with_name("Client Connection".into(), thread_num);
    log::info!("Using pool of {} threads", thread_num);

    while let Ok((conn, addr)) = lstnr.accept() {
        // This is the channel that allows
        // clients to communicate with the
        // ledger worker process.
        let trx = tx.clone();
        log::info!("New client connection: {:?}", addr);
        pool.execute(move || {
            conn::init(conn, trx);
        });
    }
}
