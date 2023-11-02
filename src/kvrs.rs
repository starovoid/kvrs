mod handlers;

use clap::{Arg, ArgMatches, Command};
use libkvrs::StorageError;
use crate::handlers::HANDLERS;

fn cli() -> Command {
    Command::new("kvrs")
        .name("kvrs")
        .subcommand(Command::new("get").arg(Arg::new("key").index(1).required(true)))
        .subcommand(
            Command::new("set")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("value").index(2).required(true)),
        )
        .subcommand(
            Command::new("update")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("value").index(2).required(true)),
        )
        .subcommand(
            Command::new("rm")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("value").index(2).required(true)),
        )
        .arg(Arg::new("file").long("file").short('f'))
}

fn main() {
    let matches = cli().get_matches();

    let command = matches.subcommand();
    let (operation, args) = match command {
        Some(cmd) => cmd,
        None => {
            println!("Command not set");
            return;
        }
    };

    match process_command(operation, args) {
        Ok(()) => {
            println!("Command finished")
        }
        Err(err) => {
            println!("Storage error: {}", err);
        }
    };
}

fn process_command(operation: &str, args: &ArgMatches) -> Result<(), StorageError> {
    let handler = match HANDLERS.get(operation) {
        Some(handler) => handler,
        None => return Err(StorageError::UnknownOperation(operation.to_string())),
    };

    handler(args.clone())
}