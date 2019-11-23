# Development Plan

This document represents the current expected blueprint
for `rtcoin` as of `23 November 2019`

*Not Finalized*

## `rtcoin-server`

### Overview

**Ledger Database**
* libsqlcipher's native `AES-256` encryption will be used.
* Will prompt for a database password on startup.
* SQLite is accessed in serialized mode of operation
* Three tables: Ledger, Archive, Users

**Server Daemon**
* Three primary threads: Init, Ledger Worker, and Connection Worker.
* There is a fourth thread listening for `SIGINT`
* On first startup, will generate a key pair to verify requests and sign responses.
* Listen on UNIX Domain Socket
* Connection worker will spawn a new thread out of a pool of `num_cpu * 4` to handle each connection received.
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
    * `Vec<String>` Arguments of request (even indices beginning 0 represent the type of argument, odd indices represent the argument itself). A valid SQL statement will be constructed internally. This will probably change to a `Vec<(String, String)>` for argument/value pairs.
    * `mpsc` Origin channel leading to the thread associated with the requesting connection.
* No raw SQL statements will be accepted from client connections. They must always be constructed by the server based on the client-originating request.

**Connection Worker**
* Binds to a UNIX Domain Socket
* Spawns a new thread for each incoming client connection out of a pool of capacity `num_cpu::get() * 4`. This isn't designed with a high number of concurrent connections in mind.
* Clones the Sender half of Ledger Worker's channel to give to each child thread.

**Connection Child Threads**
* Receives signed JSON requests via UNIX Domain Socket.
    * kind: the `enum` type of request mentioned in the Ledger Worker section.
    * args: arguments of the request. The argument string will be interpreted differently based on the kind of request.
    * `{ "kind": "whoami", "args": "foo_barrington" }`
    * `{ "kind": "register", "args": "some_username\tsome_password_hash\tpubkey_goes_here" }`
    * `args` will be tab-delineated.
* Verifies signature of each request. If it fails, let the client know, then disconnect/die.
* Unpacks the JSON into a request struct.
* Sends the request to the Ledger Worker along its channel.
* Packs replies from the Ledger Worker into signed JSON and sends it back to the client.
* JSON for a response to a given client is serialized from the following:
* `enum db::Reply` contains:
    * `Data(String)`,
    * `Error(err::Resp as String)`,
    * `Rows(Vec<String>)` (columns for each row are tab-delineated)
* `struct err::Resp` contains the fields:
    * `u32` code: numeric identifier for the error. The errors are enumerated in `err.rs`.
    * `String` kind: the classification of the error.
    * `String` details: further context on this specific error incident.

## `rtcoin-client`
* On startup, check for key pair to be used to sign/verify communications with the server. If the key pair doesn't exist, generate one.
* Any time a password must be sent along with a request, a `SHA-256` hash of the password will be sent.