use std::str::FromStr;

use anyhow::Result;
use version::Version;

use crate::CommandAndHandler;

/// The command handler of the help command.
pub fn handler(_args: Vec<String>) -> Result<()> {
    let ver: Version = FromStr::from_str(version!()).unwrap();
    println!("Version: {}", ver);
    Ok(())
}

// submit the command to CommandAndHandler Iterator.
inventory::submit! {
    CommandAndHandler::new("version", "Print the version of this message.", handler)
}
