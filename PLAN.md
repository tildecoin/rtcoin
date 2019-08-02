# Development Plan

This document represents the current expected blueprint
for `rtcoin` as of `02 August 2019`

*Not Finalized*

## `rtcoin-server`

### Overview

**Ledger Database**
* libsqlcipher's native `AES-256` encryption will be used.
* Will prompt for a database password on startup. This will present additional complexity when attempting to control `rtcoin-server` from a traditional init system. However, this is a trade-off to make for additional security when compared to a stored password.
* SQLite is accessed in serialized mode of operation
* Three tables: Ledger, Archive, Users

**Server Daemon**
* Three primary threads: Init, Ledger Worker, and Connection Worker.
* There is a fourth, minor thread listening for `SIGINT`
* On first startup, will generate a key pair to verify requests and sign responses.
* Listen on UNIX Domain Socket
* Connection worker will spawn a new thread out of a pool of `num_cpu::get() * 4` to handle each connection received.
* `std::thread` spawns and controls OS threads rather than something lightweight like the `M:N` threads in `Go`. I need to consider some designs using `async` and `await`. This would allow a bit more scalability without the impact on system resources that comes with OS threads (a minor one, but more than with `M:N` threads). The caveat being that `async` and `await` are not in stable Rust yet. See [areweasyncyet.rs](https://areweasyncyet.rs) for the development status of asynchronous Rust.
* Each connection thread will communicate with the Ledger Worker via `mpsc` (channel).

**Init**
* Initializes logging
* Authenticates with database
* Spawns ledger worker, connection worker, and `SIGINT` handler threads.

**Ledger Worker**
* Receives requests on an `mpsc`, no restriction on buffer.
* As requests are handled `FIFO`, there will be no issues with multiple SQL transactions occuring at the same time. If a `SIGINT` is received, a disconnect signal will be sent to the transaction queue and executed in order, allowing pending transactions to complete before shutting down.
* Requests will have been previously deserialized into a struct containing the following fields:
    * `enum` Type of request: Register, Whoami, Rename, Send, Sign, Balance, Verify, Contest, Audit, Resolve, Second, Query, Disconnect. Query and Disconnect will be reserved for internal requests. Client-originating requests will be unable to utilize "Query" (arbitrary query) or "Disconnect" (shut down worker thread) - they will trigger an error response.
    * `Vec<String>` Arguments of request (even indices beginning 0 represent the type of argument, odd indices represent the argument itself). A valid SQL statement will be constructed internally.
    * `mpsc` Origin channel leading to the thread associated with the requesting connection.
* No raw SQL statements will be accepted from client connections. They must always be constructed based on request.

**Connection Worker**
* Binds to a UNIX Domain Socket
* Spawns a new thread for each incoming client connection out of a pool of capacity `num_cpu::get() * 4`
* Clones the Sender half of Ledger Worker's channel to give to each child thread.

**Connection Child Threads**
* Receives signed JSON requests via UNIX Domain Socket.
    * kind: the `enum` type of request mentioned in the Ledger Worker section.
    * args: arguments of the request. The argument string will be interpreted differently based on the kind of request.
    * `{ "kind": "whoami", "args": "foo_barrington" }`
* Verifies signature of each request. If it fails, let the client know, then disconnect/die.
* Unpacks the JSON into a request struct.
* Sends the request to the Ledger Worker along its channel.
* Packs replies from the Ledger Worker into signed JSON and sends it back to the client.
* JSON for a response to a given client is serialized from the following:
* `enum db::Reply` contains:
    * `Data(String)`,
    * `Error(err::Resp as String)`,
    * `Rows(Vec<String>)` (columns for each row are tab-delineated)
* `err::Resp` contains the fields:
    * `u32` code: numeric identifier for the error. The errors are enumerated in `err.rs`.
    * `String` kind: the classification of the error.
    * `String` details: further context on this specific error incident.
