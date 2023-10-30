use clap::{Arg, Command};

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

    if let Some((operation, args)) = matches.subcommand() {
        match operation {
            "get" => {
                let key = args
                    .get_one::<String>("key")
                    .expect("Needs to specify a key: 'kvrs get \"key\"'");
                println!("Getting a value by the key \"{key}\"")
            }
            "set" => {
                let key = args
                    .get_one::<String>("key")
                    .expect("Needs to specify a key: 'kvrs set \"key\" \"value\"'");
                let _value = args
                    .get_one::<String>("value")
                    .expect("Needs to specify a value: 'kvrs set \"key\" \"value\"'");
                println!("Setting the key-value pair with the key \"{key}\"");
            }
            "update" => {
                let key = args
                    .get_one::<String>("key")
                    .expect("Needs to specify a key: 'kvrs update \"key\" \"value\"'");
                let _new_value = args
                    .get_one::<String>("value")
                    .expect("Needs to specify a value: 'kvrs update \"key\" \"value\"'");
                println!("Updating value by the key \"{key}\"");
            }
            "rm" => {
                let key = args
                    .get_one::<String>("key")
                    .expect("Needs to specify a key: 'kvrs rm \"key\"'");
                println!("Deleting a key-value pair by key \"{key}\"");
            }
            _ => unreachable!(),
        }
    }
}
