# Development Plan

This document represents the current expected blueprint
for `rtcoin` as of `24 July 2019`

*Not Finalized*

## `rtcoin-server`

### Overview

**Ledger Database**
* libsqlcipher's native `AES-256` encryption will be used.
* Will prompt for a database password on startup. This will present additional complexity when attempting to control `rtcoin-server` from a traditional init system. However, this is a trade-off to make for additional security.
* SQLite is accessed in serialized mode of operation
* Three tables: Ledger, Archive, Users

**Server Daemon**
* Two primary threads of execution: Ledger Worker and Connection Worker
* On first startup, will generate a key pair to verify requests and sign responses.
* Listen on UNIX Domain Socket
* Connection worker will spawn a new thread to handle each connection received.
* Each connection thread will communicate with the Ledger Worker via `mpsc` (channel).

**Ledger Worker**
* Receives requests on an `mpsc`, no restriction on buffer.
* Since SQLite is serializing transactions, new threads can be spawned to run the transactions without sacrificing database integrity. SQLite will handle the mutexing internally. However, `std::thread` spawns and controls OS threads rather than something lightweight like the `M:N` threads in `Go`. I need to consider some designs using `async` and `await`. This would allow a bit more scalability without the impact on system resources that comes with OS threads (a minor one, but more than with `M:N` threads). The caveat being that `async` and `await` are not in stable Rust yet. See [areweasyncyet.rs](https://areweasyncyet.rs) for the development status of asynchronous Rust.
* Requests will have been previously deserialized into a struct containing the following fields:
    * `enum` Type of request: Register, Whoami, Rename, Send, Sign, Balance, Verify, Contest, Audit, Resolve, Second, Query, Disconnect. Query and Disconnect will be reserved for internal requests. Client-originating requests will be unable to utilize "Query" (arbitrary query) or "Disconnect" (shut down worker thread) - they will trigger an error response.
    * `Vec<String>` Arguments of request (even indices beginning 0 represent the type of argument, odd indices represent the argument itself). A valid SQL statement will be constructed internally.
    * `mpsc` Origin channel leading to the thread associated with the requesting connection.
* No raw SQL statements will be accepted. They must always be constructed based on request. This is a security issue.

**Connection Worker**
* Spawns a new thread for each incoming client connection.
* Clones the Sender half of Ledger Worker's channel to give to each child thread.

**Connection Child Threads**
* Receives signed JSON requests via UNIX Domain Socket.
* Verifies signature of each request. If it fails, let the client know, then disconnect/die.
* Unpacks the JSON into a request struct.
    * Still working on a schema that would allow variability in requests without making it too cumbersome to deserialize into the request structure. Trying to keep the code simple.
* Sends the request to the Ledger Worker along its channel.
* Packs replies from the Ledger Worker into signed JSON and sends it back to the client.
* If an error is generated at any point, it is passed "down the line" until it reaches the child thread handling the connection. The child thread then creates an `ErrResp` structure for ease of logging and serialization into bytes representing JSON. The JSON bytes are then passed to the client.
* `ErrResp` contains the fields:
    * `u32` Code: numeric identifier for the error. Currently includes `01` - Ledger Worker Error, `02` - JSON Error, `03` - Invalid Request.
    * `String` Kind: the textual representation of the error.
    * `String` Details: further context on this specific error incident.
