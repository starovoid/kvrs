mod handlers;

use crate::handlers::HANDLERS;
use clap::{Arg, ArgMatches, Command};
use libkvrs::{DataFormatError, StorageError};

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

fn process_command(operation: &str, args: &ArgMatches) -> Result<(), String> {
    let handler = match HANDLERS.get(operation) {
        Some(handler) => handler,
        None => return Err(format!("Unknown '{operation}' operation")),
    };

    handler(args.clone()).map_err(|e| format_error_message(&e))
}

fn format_error_message(err: &StorageError) -> String {
    match err {
        StorageError::IO(e) => format!("IO Error: {e}"),
        StorageError::DataFormat(format_err) => match format_err {
            DataFormatError::MissedIdentifier => format!("Missing identifier"),
            DataFormatError::IncorrectVersion(v) => format!("Used unsupported version: {v}"),
        },
    }
}
