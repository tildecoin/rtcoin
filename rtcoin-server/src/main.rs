//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::{
    error::Error, 
    fs,
    io,
    os::unix::net::UnixListener, 
    path::Path, 
    process, 
    sync::mpsc, 
    thread,
    time,
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
mod json;
mod logging;
mod user;

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
    let sigint_tx = tx.clone();
    ctrlc::set_handler(move || {
        warn!("^C / SIGINT Caught. Cleaning up ...");
        if fs::metadata(sock).is_ok() {
            info!("Removing socket file");
            fs::remove_file(sock).unwrap();
        }

        info!("Sending disconnect signal to ledger worker");
        sigint_tx
            .send(db::Comm::new(
                Some(db::Kind::Disconnect),
                None,
                None
            ))
            .expect("Failed to send disconnect comm to ledger worker");
        
        // Give the database a bit to close/encrypt
        thread::sleep(time::Duration::from_millis(50));
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
    let mut ledger = DB::connect(db::PATH, db_key.clone(), rx);
    let mut db_key = db_key;
    db_key.zeroize();

    // Naming the thread helps with debugging. It will
    // show up in panics.
    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    info!("Starting ledger worker process...");
    let wait = ledger_worker.spawn(move || {
        ledger.worker_thread();
        ledger.conn.close().unwrap();
    })
    .expect("Ledger worker failed to spawn");

    // Block execution until the thread we just
    // spawned returns.
    info!("Startup finished!");
    wait.join().unwrap();
}

fn spawn_for_connections(sock: &Path, tx: mpsc::Sender<db::Comm>) {
    let lstnr = UnixListener::bind(sock).unwrap_or_else(|_|{
        error!("Could not bind to socket: {}", conn::SOCK);
        panic!("Could not bind to socket: {}", conn::SOCK);
    });

    // The thread pool will always allow at least
    // four simultaneous client connections. I chose 
    // this multiplier because the client connections 
    // will generally not exec resource intensive 
    // operations.
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
