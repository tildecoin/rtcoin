use std::thread;

#[macro_use]
use clap::{crate_version, value_t};
use clap::{Arg, App, SubCommand};

fn main() {
    let args = App::new("rtcoin")
        .version(crate_version!())
        .author("Ben Morrison (gbmor) (based on tcoin by login000)")
        .about("Currency Simulation for the Tildeverse")
        .arg(Arg::with_name("messages")
             .short("m")
             .long("messages")
             .value_name("[n]")
             .help("Displays all messages, or last N messages.")
             .takes_value(true))
        .arg(Arg::with_name("send")
             .short("s")
             .long("send")
             .value_name("[username] [amount] [message]")
             .help("Send rtcoin to another user.")
             .takes_value(true))
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
    next.join();
}

// TODO: Need to come up with some kind of authentication
// system for users. And come up with a way to store and
// retrieve coin balances. The original implementation used
// flat files.

fn next_step(args: clap::ArgMatches) {
    if args.is_present("messages") {
        let mut msg_num = value_t!(args, "messages", i32).unwrap_or(0);
        if msg_num < 0 || msg_num > i32::max_value() {
            msg_num = 0;
        }

        if msg_num == 0 {
            println!("Displaying ALL messages");
        } else {
            println!("Displaying {} most recent messages", msg_num);
        }
    }
}
