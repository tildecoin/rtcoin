# rtcoin [![Build Status](https://travis-ci.com/tildecoin/rtcoin.svg?branch=master)](https://travis-ci.com/tildecoin/rtcoin) [![codecov](https://codecov.io/gh/tildecoin/rtcoin/branch/master/graph/badge.svg)](https://codecov.io/gh/tildecoin/rtcoin)

`tildecoin` is a fun currency simulation concept originally written for the [tildeverse](https://tildeverse.org).
`rtcoin` is the second-generation implementation, meant to improve upon
the first. The specifications set forth in the
draft RFC for `tildecoin`, written by [~aewens](https://github.com/aewens), will be followed:
* [tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md](https://tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md)

This project is in early development. 

## Project Structure

`rtcoin` will use a client-server architecture, per the RFC.

Initial work is being done on [`rtcoin-server`](https://github.com/tildecoin/rtcoin/tree/master/rtcoin-server),
which will handle connections to clients, client authentication, and manage the ledger as a 
table in a `SQLite v3` database. Afterwards, work will move to `rtcoin-client`, which will 
be what users interact with to display their balances, transfer `tildecoin` to other users, etc.

## Notes

* MIT Licensed
* Draft RFC: [tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md](https://tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md)
* The first tildecoin implementation: [`github.com/login000/tcoin`](https://github.com/login000/tcoin)

## Contributing

If you'd like to help out, the current build dependencies are:

* `libsqlcipher-dev`

If you have a cool idea for how something should be implemented, go for it!
Soon I'll have a roadmap document outlining, in detail, the things that need to be done.
I know from the outside the development direction is kind of opaque. I'm still figuring
a couple of things out myself. I'm building the scaffolding right now, and will make 
things more  robust as I go along.

* The ledger is using the native `AES-256` encryption in `libsqlcipher`
  - This is a superset of `libsqlite3`
* Communication between clients and server is via Unix Domain Socket
* Communication consists of a simple `JSON` schema
  - A brief example can be seen in `conn.rs::L75::json_to_comm()`
  - This function serializes a test `JSON` transmission into the `db.rs::L32::Comm` structure
  - The `db::Comm` is then passed via `mpsc` (channel) to the worker process.
* A single worker process handles all ledger transactions `FIFO`.
  - `db::Comm` is what the worker process handling the database 'speaks'
* Some transactions deemed non-destructive will be implicitly authenticated.
  - The `UID` of the communicating process will be used
  - Balances, etc are viewable at this level of authentication
  - Allows a bit of extensibility with client-side tooling
* Transactions deemed destructive will require further authentication.
  - Includes sending coin
  - Maybe a password hash
  - Maybe a key generated during account init (haven't weighed the options yet)
* All communication will need to be signed
