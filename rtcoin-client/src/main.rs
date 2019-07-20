//
// rtcoin - Copyright (c) 2019 Ben Morrison (gbmor)
// See LICENSE file for detailed license information.
//

use std::error::Error;
use std::thread;
use std::time::Duration;

use clap::{crate_version, value_t}; // the macros
use clap::{App, Arg, SubCommand};

fn main() -> Result<(), Box<dyn Error>> {
    println!();
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

    // let this_user = User::new("Bob Bobson");

    // Obviously concurrency isn't strictly
    // necessary at this point. I'm going to
    // experiment with some design variations
    // for authentication that may use extra
    // threads.
    let next = thread::spawn(move || {
        //  next_step(this_user, args);
    });
    if let Err(err) = next.join() {
        eprintln!("{:?}", err);
    }

    Ok(())
}

// Currently, the following are just stubs meant to help me
// mentally track the program's execution. They will most
// likely not exist in the near future, replaced with
// something else.
/*
fn next_step(user: User, args: clap::ArgMatches) {
    let mut user = user;
    match args.subcommand() {
        ("messages", Some(msg_args)) => handle_messages(msg_args, &user),
        ("send", Some(send_args)) => send_tcoin(send_args, &mut user),
        ("", None) => println!("No subcommand issued. Something's wrong."),
        (&_, _) => println!("Something went wrong during argument parsing."),
    }
}

fn handle_messages(num: &clap::ArgMatches, user: &User) {
    let num = value_t!(num, "n", u32).unwrap_or(0);
    if num == 0 {
        println!("Displaying ALL messages");
        for (i, m) in user.messages().iter().enumerate() {
            println!("{}: {}", i, m);
        }
    } else {
        println!("Displaying {} most recent messages", num);
    }
}

fn send_tcoin(send_args: &clap::ArgMatches, user: &mut User) {
    println!(" Depositing ...");
    thread::sleep(Duration::from_secs(5));

    let who = send_args.value_of("username").unwrap();
    let hwat = value_t!(send_args, "amount", f64)
        .expect("Invalid transfer amount");
    let msg = send_args.value_of("message").unwrap_or("");

    user.deposit(hwat).unwrap();
    user.append_messages(msg);

    println!(
        " Sending tcoin:\n\tUser: {}\n\tAmount: {}\n\tMessage: {}",
        who, hwat, msg
    );

    println!("\n{}", user);

    println!("\n{:#?}", user);
}*/
