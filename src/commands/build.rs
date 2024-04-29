use std::{env::current_dir, process::Command};

use anyhow::Result;

use crate::{config::read_bin_config, CommandAndHandler};

/// The handler of the command.
fn handler(args: Vec<String>) -> Result<()> {
    let file_name = match args.len() > 2 {
        true => &args[2],
        false => "byteos.yaml",
    };

    let bin = if args.len() > 3 { &args[3] } else { "default" };

    let mut rustflags = Vec::new();
    // default configuration.
    rustflags.push(String::from("-Cforce-frame-pointers=yes"));
    rustflags.push(String::from("-Clink-arg=-no-pie"));
    rustflags.push(String::from("-Ztls-model=local-exec"));

    // Convert byteos configuration to rustflags.
    // This rustflags will be passed to the rust build command.
    let binary_config = read_bin_config(file_name, bin)?;
    for (key, value) in binary_config.get_configs() {
        rustflags.push(format!("--cfg={}=\"{}\"", key, value));
        println!("{} = {:?}", key, value);
    }

    let mut extra_args = Vec::new();

    if !binary_config.build_std {
        extra_args.push("-Z");
        extra_args.push("build-std");
    }

    // build os
    let mut outputs = Command::new("cargo")
        .env("RUSTFLAGS", rustflags.join(" "))
        .env("ROOT_MANIFEST_DIR", current_dir().unwrap())
        .envs(binary_config.get_envs())
        .arg("build")
        .args(extra_args)
        .arg("--target")
        .arg(binary_config.target)
        .arg("--release")
        .spawn()
        .expect("can't build byteos");

    // Wait for build complete.
    let exit_status = outputs.wait().expect("can't wait for build");
    if !exit_status.success() {
        return Err(anyhow!("build bin target {bin} failed, {exit_status}"));
    }

    Ok(())
}

inventory::submit! {
    CommandAndHandler::new("build", "build the byteos through a yaml.", handler)
}
