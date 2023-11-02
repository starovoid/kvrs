use std::collections::HashMap;
use clap::ArgMatches;
use lazy_static::lazy_static;
use libkvrs::StorageError;

type HandlerType = fn(ArgMatches) -> Result<(), StorageError>;

lazy_static! {
    pub static ref HANDLERS: HashMap<&'static str, HandlerType> = HashMap::from([
        ("get", get_handler as HandlerType),
        ("set", set_handler as HandlerType),
        ("update", update_handler as HandlerType),
        ("remove", remove_handler as HandlerType),
    ]);
}

fn get_handler(_args: ArgMatches) -> Result<(), StorageError> {
    todo!()
}

fn set_handler(_args: ArgMatches) -> Result<(), StorageError> {
    todo!()
}

fn update_handler(_args: ArgMatches) -> Result<(), StorageError> {
    todo!()
}

fn remove_handler(_args: ArgMatches) -> Result<(), StorageError> {
    todo!()
}