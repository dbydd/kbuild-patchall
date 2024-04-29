use anyhow::Result;

use crate::{config::read_bin_config, CommandAndHandler};

/// The handler of the command.
fn handler(args: Vec<String>) -> Result<()> {
    if args.len() < 5 || (args[4] != "get_env" && args[4] != "get_cfg") {
        println!("byteos [config_file] [bin] [get_env|get_cfg] [name]");
    }

    let file_name = args[2].as_str();
    let bin = args[3].as_str();
    let ops = args[4].as_str();
    let name = args[5].as_str();

    // Convert byteos configuration to rustflags.
    // This rustflags will be passed to the rust build command.
    let binary_config = read_bin_config(&file_name, bin)?;

    let value = match ops {
        "get_env" => binary_config
            .get_envs()
            .get(name)
            .cloned()
            .ok_or(anyhow!("Can't find env {name}")),
        "get_cfg" => binary_config
            .get_configs()
            .get(name)
            .cloned()
            .ok_or(anyhow!("Can't find config {name}")),
        _ => unreachable!(),
    }?;

    println!("{value}");

    Ok(())
}

inventory::submit! {
    CommandAndHandler::new("config", "get or set(todo) config from file.", handler)
}
