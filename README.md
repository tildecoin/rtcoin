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

### Notes

* The first tildecoin implementation: [`github.com/login000/tcoin`](https://github.com/login000/tcoin)
* Using Rust `nightly >= 1.38` for development, however, travis builds include `stable` and `beta`.
* MIT Licensed
* Contributions welcome!
* Draft RFC: [tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md](https://tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md)

### Current Build Dependencies

* libsqlcipher-dev
