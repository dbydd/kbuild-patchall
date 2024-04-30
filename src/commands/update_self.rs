use std::process::Command;

use anyhow::Result;

use crate::CommandAndHandler;

/// The command handler of the help command.
pub fn handler(_args: Vec<String>) -> Result<()> {
    let mut outputs = Command::new("cargo")
        .arg("install")
        .arg("kbuild")
        .spawn()
        .expect("can't clone from git repository");
    outputs.wait().expect("can't wait for git clone");

    Ok(())
}

// submit the command to CommandAndHandler Iterator.
inventory::submit! {
    CommandAndHandler::new("update_self", "update self(tools only)", handler)
}
