# Development Plan

This document represents the current expected course of development
for `rtcoin` as of `23 July 2019`

*Not Finalized*

## `rtcoin-server`

### Overview

**Ledger Database**
* libsqlcipher's native `AES-256` encryption will be used.
* The database key will be provided by the instance administrator. It's suggested to use something like `pwgen` to create a strong, lengthy password or passphrase. Care should be taken to ensure the file containing the key is locked down, not group or world-readable. Treat it like a private GPG or SSL key.
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
* Requests will have been previously serialized into a struct containing the following fields:
    * `enum` Type of request: Register, Query, WHOAMI, Rename, Send, Sign, Balance, Verify, Contest, Audit, Resolve, Second
    * `String` Arguments of request. This will be split and verified, then a SQL statement will be constructed internally.
    * `mpsc` Origin channel leading to the thread associated with the requesting connection.
* No raw SQL statements will be accepted. They must always be constructed based on request. This is a security issue.

**Connection Worker**
* Spawns a new thread for each incoming client connection.
* Receives signed JSON requests via UNIX Domain Socket.
* Verifies the signature and unpacks the JSON into a request struct.
* Sends the request to the Ledger Worker.
* Packs replies from the Ledger Worker into signed JSON and sends it back to the client.