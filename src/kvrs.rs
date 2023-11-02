use clap::{Arg, Command};

fn cli() -> Command {
    Command::new("kvrs")
        .name("kvrs")
        .subcommand(
            Command::new("get")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("file").long("file").short('f')),
        )
        .subcommand(
            Command::new("set")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("value").index(2).required(true))
                .arg(Arg::new("file").long("file").short('f')),
        )
        .subcommand(
            Command::new("update")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("value").index(2).required(true))
                .arg(Arg::new("file").long("file").short('f')),
        )
        .subcommand(
            Command::new("rm")
                .arg(Arg::new("key").index(1).required(true))
                .arg(Arg::new("value").index(2).required(true))
                .arg(Arg::new("file").long("file").short('f')),
        )
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

    match operation {
        "get" => {
            let _key = args
                .get_one::<String>("key")
                .expect("Needs to specify a key: 'kvrs get \"key\"'");
            let _file = args.get_one::<String>("file");
            todo!()
        }
        "set" => {
            let _key = args
                .get_one::<String>("key")
                .expect("Needs to specify a key: 'kvrs set \"key\" \"value\"'");
            let _value = args
                .get_one::<String>("value")
                .expect("Needs to specify a value: 'kvrs set \"key\" \"value\"'");
            let _file = args.get_one::<String>("file");
            todo!()
        }
        "update" => {
            let _key = args
                .get_one::<String>("key")
                .expect("Needs to specify a key: 'kvrs update \"key\" \"value\"'");
            let _new_value = args
                .get_one::<String>("value")
                .expect("Needs to specify a value: 'kvrs update \"key\" \"value\"'");
            let _file = args.get_one::<String>("file");
            todo!()
        }
        "rm" => {
            let _key = args
                .get_one::<String>("key")
                .expect("Needs to specify a key: 'kvrs rm \"key\"'");
            let _file = args.get_one::<String>("file");
            todo!()
        }
        _ => unreachable!(),
    }
}
