use std::env;

use anyhow::Result;
use color_print::cprintln;

#[macro_use]
extern crate version;
#[macro_use]
extern crate anyhow;

mod commands;
mod config;
mod utils;

/// To declare the command struct.
///
/// This main function will check this struct and call the correct handler.
pub struct CommandAndHandler {
    pub command: &'static str,
    pub description: &'static str,
    pub handler: fn(Vec<String>) -> Result<()>,
}

impl CommandAndHandler {
    /// Create a new CommandAndHandler struct.
    pub const fn new(
        command: &'static str,
        description: &'static str,
        handler: fn(Vec<String>) -> Result<()>,
    ) -> Self {
        CommandAndHandler {
            command,
            description,
            handler,
        }
    }
}

// Collect the command was submitted.
inventory::collect!(CommandAndHandler);

fn main() {
    if let Err(err) = exec() {
        cprintln!("<red><bold>Error: {}</bold></red>", err);
    }
}

/// This function will execute the command. And return the result.
fn exec() -> Result<()> {
    // get arugments.
    let args: Vec<String> = env::args().collect();
    // print the help message if the number of the command if less than 2.
    if args.len() < 2 {
        commands::help_handler(args)?;
        return Ok(());
    }
    // Find the correct command was called.
    for command in inventory::iter::<CommandAndHandler> {
        if args[1] == command.command {
            (command.handler)(args)?;
            return Ok(());
        }
    }
    // If the command was not found, print the help message.
    commands::help_handler(args)
}
