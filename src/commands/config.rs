use std::process::{Command, Stdio};

use anyhow::Result;

use crate::{config::read_bin_config, CommandAndHandler};

/// The handler of the command.
fn handler(args: Vec<String>) -> Result<()> {
    let commands = ["get_env", "get_cfg", "get_triple", "get_meta"];
    if args.len() < 5 || (!commands.contains(&args[4].as_str())) {
        println!("kbuild [config_file] [bin] [{}] [name]", commands.join("|"));
        return Ok(());
    }

    let file_name = args[2].as_str();
    let bin = args[3].as_str();
    let ops = args[4].as_str();
    let name = args[5].as_str();

    // Convert kernel configuration to rustflags.
    // This rustflags will be passed to the rust build command.
    let binary_config = read_bin_config(file_name, bin)?;

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
        "get_triple" => {
            // -Z unstable-options --print target-spec-json --target riscv64gc-unknown-none-elf
            let task = Command::new("rustc")
                .args(vec!["+nightly", "-Z", "unstable-options"])
                .args(vec!["--print", "target-spec-json"])
                .args(vec!["--target", &binary_config.target])
                .stdout(Stdio::piped())
                .spawn()?;
            let outputs = task.wait_with_output()?;
            let str = String::from_utf8(outputs.stdout)?;
            let triple = json::parse(&str)?;
            triple[name]
                .as_str()
                .ok_or(anyhow!("can't get {name} from triple"))
                .map(|x| x.to_string())
        }
        "get_meta" => binary_config
            .get_meta()
            .get(name)
            .cloned()
            .ok_or(anyhow!("can't find meta {name}")),
        _ => unreachable!(),
    }?;

    println!("{value}");

    Ok(())
}

inventory::submit! {
    CommandAndHandler::new("config", "get or set(todo) config from file.", handler)
}
