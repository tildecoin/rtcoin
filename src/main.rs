//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::thread;

#[macro_use]
use clap::{crate_version, value_t}; // the macros
use clap::{App, Arg, SubCommand};

// TODO: Need to come up with some kind of authentication
// system for users. And come up with a way to store and
// retrieve coin balances. The original implementation used
// flat files.

fn main() {
    print!("\n");
    let args = App::new("rtcoin")
        // This uses a macro to pull the version
        // stated in Cargo.toml, rather than having
        // it hard-coded in source or something.
        .version(crate_version!())
        .author("Ben Morrison (gbmor) :: based on tcoin, originally by login000")
        .about("Currency Simulation for the Tildeverse")
        // The number of messages to pull. Needs
        // a bit more work before this stub will
        // be API compatible with tcoin.
        .subcommand(
            SubCommand::with_name("messages")
                .about("Display all messages, or last N messages")
                .arg(
                    Arg::with_name("n")
                        .help("The number of messages to retrieve. Blank or 0 means all.")
                        .required(false),
                ),
        )
        // For the 'send' subcommand, every field is
        // its own argument. The only optional field
        // is the third: message.
        .subcommand(
            SubCommand::with_name("send")
                .about("Send tcoin to another user. Message is optional.")
                .arg(
                    Arg::with_name("username")
                        .help("User to whom you will send tcoin")
                        .required(true)
                        .index(1),
                )
                .arg(
                    Arg::with_name("amount")
                        .help("Amount of tcoin to send")
                        .required(true)
                        .index(2),
                )
                .arg(
                    Arg::with_name("message")
                        .help("Optional message to include. Please use quotations.")
                        .required(false)
                        .index(3),
                ),
        )
        // The "on"/"off" functionality present
        // in the tcoin API doesn't sit well with
        // me. I'm going to come up with an
        // alternative that doesn't present the
        // same potential issues. Here as a
        // placeholder, mostly. Like a post-it.
        .subcommand(SubCommand::with_name("on").about("Log in to rtcoin"))
        .subcommand(SubCommand::with_name("off").about("Log out of rtcoin"))
        .subcommand(SubCommand::with_name("init").about("Initialize your rtcoin wallet"))
        // Balance *should* be the default action,
        // I think. If init has happened. Else, the
        // default action should be to init.
        .subcommand(SubCommand::with_name("balance").about("Retrieve your rtcoin balance"))
        .get_matches();

    // Obviously concurrency isn't strictly
    // necessary at this point. I'm going to
    // experiment with some design variations
    // for authentication that may use extra
    // threads.
    let next = thread::spawn(move || {
        next_step(args);
    });
    if let Err(err) = next.join() {
        eprintln!("{:?}", err);
    }
}

// TODO: The following functions need to all be moved into
// other files and imported as libraries. This will allow
// rtcoin to utilize cargo's built-in unit testing framework.
// Currently, they are just stubs meant to help me mentally
// track the program's execution. They will most likely not
// exist in the near future, replaced with something else.

fn next_step(args: clap::ArgMatches) {
    match args.subcommand() {
        ("messages", Some(msg_args)) => handle_messages(msg_args),
        ("send", Some(send_args)) => send_tcoin(send_args),
        ("", None) => println!("No subcommand issued. Something's wrong."),
        (&_, _) => println!("Something went wrong during argument parsing."),
    }
}

fn handle_messages(num: &clap::ArgMatches) {
    let num = value_t!(num, "n", u32).unwrap_or(0);
    if num == 0 {
        println!("Displaying ALL messages");
    } else {
        println!("Displaying {} most recent messages", num);
    }
}

fn send_tcoin(send_args: &clap::ArgMatches) {
    let who = send_args.value_of("username").unwrap();
    let hwat = send_args.value_of("amount").unwrap();
    let msg = send_args.value_of("message").unwrap_or("");
    println!(
        "Sending tcoin:\n\tUser: {}\n\tAmount: {}\n\tMessage: {}",
        who, hwat, msg
    );
}
