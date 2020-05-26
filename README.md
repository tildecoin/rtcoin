# rtcoin [![Build Status](https://travis-ci.com/tildecoin/rtcoin.svg?branch=master)](https://travis-ci.com/tildecoin/rtcoin) [![codecov](https://codecov.io/gh/tildecoin/rtcoin/branch/master/graph/badge.svg)](https://codecov.io/gh/tildecoin/rtcoin)

`tildecoin` is a fun currency simulation concept being written for the [tildeverse](https://tildeverse.org).
`rtcoin` is the second-generation implementation, meant to improve upon
the first. The specifications set forth in the
draft RFC for `tildecoin`, written by [~aewens](https://github.com/aewens), will be followed:
* [tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md](https://tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md)

This project is in early development. Right now, I'm focusing on `rtcoin-server`. I'm going to
be rewriting it fairly soon, after reexamining how I want to construct this. For example, instead
of a threadpool, I'll be accepting connections asynchronously, and will be listening via TCP rather
than via domain socket.

## Notes

* MIT Licensed
* Draft RFC: [tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md](https://tildegit.org/aewens/rfcs/src/branch/master/draft-tilde-coin.md)
* The first tildecoin implementation: [`github.com/login000/tcoin`](https://github.com/login000/tcoin)

## Contributing

If you'd like to help out, the current build dependencies are:

* `libsqlcipher-dev`

If you're interested, I'm working on the development plan here: [PLAN.md](https://github.com/tildecoin/rtcoin/blob/master/PLAN.md). I'll update this when it's finished.

`rtcoin` uses a client-server architecture, per the RFC.

Initial work is being done on [`rtcoin-server`](https://github.com/tildecoin/rtcoin/tree/master/rtcoin-server),
which will handle connections to clients, client authentication, and manage the ledger as a 
table in a `SQLite v3` database. Afterwards, work will move to `rtcoin-client`, which will 
be what users interact with to display their balances, transfer `tildecoin` to other users, etc.

## Updates

**2019.11.23**

Minor changes to the dev plan.

