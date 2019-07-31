//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error, 
    fs,
    os::unix::net::UnixListener, 
    path::Path, 
    process, 
    sync::mpsc, 
    thread,
};

use ctrlc;

use log::{
    error,
    info,
    warn,
};

use rpassword;
use threadpool::ThreadPool;
use num_cpus;
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
    info!("rtcoin-server is initializing.\n");

    eprintln!("\nrtcoin-server 0.1-dev");
    eprintln!("\nPlease enter the ledger password:");
    let mut db_key_in = rpassword::prompt_password_stderr("> ")
        .unwrap_or_else(|err| {
            error!("Failed to read ledger password: {}", err);
            panic!("{}", err);
        });
    eprintln!();
    let db_key = db_key_in.trim().to_string();
    db_key_in.zeroize();

    eprintln!("Continuing startup process. See log file for details.");
    eprintln!();
    // Create communication channel to the ledger database, then
    // spawn the ledger worker to listen for query requests.
    info!("Starting ledger worker...");
    let (tx, rx) = mpsc::channel::<db::Comm>();
    thread::spawn(move || spawn_ledger_worker_with_receiver(db_key, rx));

    // If the socket exists already, remove it.
    let sock = Path::new(conn::SOCK);
    if fs::metadata(sock).is_ok() {
        warn!("Socket {} already exists.", conn::SOCK);
        fs::remove_file(sock)?;
    }

    // Handle SIGINT / ^C
    let ctrlc_tx = tx.clone();
    ctrlc::set_handler(move || {
        warn!("^C / SIGINT Caught. Cleaning up ...");
        if fs::metadata(sock).is_ok() {
            info!("Removing socket file");
            fs::remove_file(sock).unwrap();
        }

        info!("SIGINT: Sending disconnect signal to ledger worker queue");
        let (reply_tx, sigint_rx) = mpsc::channel::<db::Reply>();
        let db_disconnect_signal = db::Comm::new(
                Some(db::Kind::Disconnect),
                None,
                Some(reply_tx),
            );

        match ctrlc_tx.send(db_disconnect_signal) {
            Ok(_) => { },
            Err(err) => error!("SIGINT: Failed to send disconnect signal to ledger worker: {}", err),
        }
        
        // Block to allow database to close
        match sigint_rx.recv() {
            Ok(_) => { },
            Err(_) => { },
        }
        
        info!("¡Hasta luego!");
        process::exit(0);
    })
    .expect("SIGINT handler setup failure");

    // Bind to the socket. Spawn a new connection
    // worker thread for each client connection.
    info!("Binding to socket: {}", conn::SOCK);
    spawn_for_connections(&sock, tx);

    // Tidy up
    fs::remove_file(sock)?;
    Ok(())
}

fn spawn_ledger_worker_with_receiver(db_key: String, rx: mpsc::Receiver<db::Comm>) {
    // This next call opens the actual database connection.
    // It also creates the tables if they don't yet exist.
    info!("Connecting to database: {}", db::PATH);
    let ledger = DB::connect(db::PATH, db_key.clone(), rx);
    let mut db_key = db_key;
    db_key.zeroize();

    // Naming the thread helps with debugging. It will
    // show up in panics.
    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    info!("Starting ledger worker process...");
    let worker_thread = ledger_worker.spawn(move || {
        ledger.worker_thread();
        match ledger.conn.close() {
            Err(err) => error!("Error closing database connection: {:?}", err),
            Ok(_) => info!("Database connection successfully closed"),
        }
    })
    .expect("Ledger worker failed to spawn");

    // Block execution until the thread we just
    // spawned returns.
    info!("Startup finished!");
    worker_thread.join().unwrap_or_else(|_| ());
}

fn spawn_for_connections(sock: &Path, tx: mpsc::Sender<db::Comm>) {
    let lstnr = UnixListener::bind(sock)
        .unwrap_or_else(|err|{
            let msg = format!("Could not bind to socket {} :: {}", conn::SOCK, err);
            error!("{}", msg);
            panic!("{}", msg);
        });

    // The thread pool will always allow at least
    // four simultaneous client connections. The 
    // client connections will most likely not be
    // resource hogs.
    let thread_num = num_cpus::get() * 4;
    let pool = ThreadPool::with_name("Client Connection".into(), thread_num);
    info!("Using pool of {} threads", thread_num);

    while let Ok((conn, addr)) = lstnr.accept() {
        // This is the channel that allows
        // clients to communicate with the
        // ledger worker process.
        let trx = tx.clone();
        info!("New client connection: {:?}", addr);
       
        pool.execute(move || {
                conn::init(conn, trx);
            });
    }
}
