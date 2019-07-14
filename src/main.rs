use std::thread;

#[macro_use]
use clap::{crate_version, value_t};
use clap::{Arg, App, SubCommand};

fn main() {
    print!("\n");
    let args = App::new("rtcoin")
        .version(crate_version!())
        .author("Ben Morrison (gbmor) :: based on tcoin, originally by login000")
        .about("Currency Simulation for the Tildeverse")
        .subcommand(SubCommand::with_name("messages")
                    .about("Display all messages, or last N messages")
                    .arg(Arg::with_name("n")
                         .help("The number of messages to retrieve. Black or 0 means all.")
                         .required(false)))
        .subcommand(SubCommand::with_name("send")
                    .about("Send tcoin to another user. Message is optional.")
                    .arg(Arg::with_name("username")
                         .help("User to whom you will send tcoin")
                         .required(true)
                         .index(1))
                    .arg(Arg::with_name("amount")
                         .help("Amount of tcoin to send")
                         .required(true)
                         .index(2))
                    .arg(Arg::with_name("message")
                         .help("Optional message to include. Please use quotations.")
                         .required(false)
                         .index(3)))
        .subcommand(SubCommand::with_name("on")
                    .about("Log in to rtcoin"))
        .subcommand(SubCommand::with_name("off")
                    .about("Log out of rtcoin"))
        .subcommand(SubCommand::with_name("init")
                    .about("Initialize your rtcoin wallet"))
        .subcommand(SubCommand::with_name("balance")
                    .about("Retrieve your rtcoin balance"))
        .get_matches();

    let next = thread::spawn(move || {
        next_step(args);
    });
    if let Err(err) = next.join() {
        eprintln!("{:?}", err);
    }
}

// TODO: Need to come up with some kind of authentication
// system for users. And come up with a way to store and
// retrieve coin balances. The original implementation used
// flat files.

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
        println!("Sending tcoin:\n\tUser: {}\n\tAmount: {}\n\tMessage: {}", who, hwat, msg);
}
