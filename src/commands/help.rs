use color_print::cprintln;

use crate::CommandAndHandler;

/// The command handler of the help command.
pub fn handler(_args: Vec<String>) -> Result<(), String> {
    println!("\nThe command below was available: \n");
    for command in inventory::iter::<CommandAndHandler> {
        cprintln!(
            "\t<green>{:20}</green> {}",
            command.command,
            command.description
        );
    }
    println!();
    Ok(())
}

// submit the command to CommandAndHandler Iterator.
inventory::submit! {
    CommandAndHandler::new("help", "Print the help message", handler)
}
