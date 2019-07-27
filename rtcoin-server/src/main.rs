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
    time,
};

use ctrlc;
use threadpool::ThreadPool;
use num_cpus;

mod conn;
mod db;
mod user;

use db::DB;

fn main() -> Result<(), Box<dyn Error>> {
    // Create communication channel to the ledger database, then
    // spawn the ledger worker to listen for query requests.
    let (tx, rx) = mpsc::channel::<db::Comm>();
    thread::spawn(move || spawn_ledger_worker_with_receiver(rx));

    // If the socket exists already, remove it.
    let sock = Path::new(conn::SOCK);
    if fs::metadata(sock).is_ok() {
        fs::remove_file(sock)?;
    }

    // Handle SIGINT / ^C
    let sigint_tx = tx.clone();
    ctrlc::set_handler(move || {
        eprintln!(" Caught. Cleaning up ...");
        if fs::metadata(sock).is_ok() {
            fs::remove_file(sock).unwrap();
        }

        sigint_tx
            .send(db::Comm::new(
                Some(db::Kind::Disconnect),
                None,
                None
            ))
            .expect("Failed to send disconnect comm to ledger worker");
        
        // Give the database a bit to close/encrypt
        thread::sleep(time::Duration::from_millis(50));
        process::exit(0);
    })
    .expect("SIGINT handler setup failure");

    // Bind to the socket. Spawn a new connection
    // worker thread for each client connection.
    spawn_for_connections(&sock, tx);

    // Tidy up
    fs::remove_file(sock)?;
    Ok(())
}

fn spawn_ledger_worker_with_receiver(rx: mpsc::Receiver<db::Comm>) {
    // This next call opens the actual database connection.
    // It also creates the tables if they don't yet exist.
    let mut ledger = DB::connect(db::PATH, rx);

    // Naming the thread helps with debugging. It will
    // show up in panics.
    let ledger_worker = thread::Builder::new();
    let ledger_worker = ledger_worker.name("Ledger Worker".into());

    let wait = ledger_worker.spawn(move || {
        ledger.worker_thread();
        ledger.conn.close().unwrap();
    })
    .expect("Ledger worker failed to spawn");

    // Block execution until the thread we just
    // spawned returns.
    wait.join().unwrap();
}

fn spawn_for_connections(sock: &Path, tx: mpsc::Sender<db::Comm>) {
    let lstnr = UnixListener::bind(sock).unwrap_or_else(|_|{
        panic!("Could not bind to socket: {}",
        sock.to_str().unwrap())
    });

    // The thread pool will always allow at least
    // four simultaneous client connections. I chose 
    // this multiplier because the client connections 
    // will generally not exec resource intensive 
    // operations.
    let thread_num = num_cpus::get() * 4;
    let pool = ThreadPool::with_name("Client Connection".into(), thread_num);
    eprintln!("Using pool of {} threads", thread_num);

    while let Ok((conn, addr)) = lstnr.accept() {
        // This is the channel that allows
        // clients to communicate with the
        // ledger worker process.
        let trx = tx.clone();
        eprintln!("New connection: {:?}", addr);
       
        pool.execute(move || {
                conn::init(conn, trx);
            });
    }
}
