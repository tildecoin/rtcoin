# Development Plan

This document represents the current expected blueprint
for `rtcoin` as of `23 July 2019`

*Not Finalized*

## `rtcoin-server`

### Overview

**Ledger Database**
* libsqlcipher's native `AES-256` encryption will be used.
* The database key will be provided by the instance administrator. It's suggested to use something like `pwgen` to create a strong, lengthy password or passphrase. Care should be taken to ensure the file containing the key is locked down, not group or world-readable. Treat it like a private GPG or SSL key. This method kind of worries me, I'm probably going to come up with something better.
* SQLite is accessed in serialized mode of operation
* Three tables: Ledger, Archive, Users

**Server Daemon**
* Two primary threads of execution: Ledger Worker and Connection Worker
* On first startup, will generate a key pair.
* Listen on UNIX Domain Socket
* Connection worker will spawn a new thread to handle each connection received.
* Each connection thread will communicate with the Ledger Worker via `mpsc` (channel).

**Ledger Worker**
* Receives requests on an `mpsc`, no restriction on buffer.
* Since SQLite is serializing transactions, new threads can be spawned all willy-nilly to run the transactions without sacrificing database integrity. SQLite will handle the mutexing internally.
* Requests will have been previously deserialized into a struct containing the following fields:
    * `enum` Type of request: Register, Query, WHOAMI, Rename, Send, Sign, Balance, Verify, Contest, Audit, Resolve, Second
    * `String` Arguments of request. This will be split and verified, then a SQL statement will be constructed internally.
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