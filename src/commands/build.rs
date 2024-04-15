use std::process::Command;

use crate::{config, CommandAndHandler};

/// The handler of the command.
fn handler(args: Vec<String>) -> Result<(), String> {
    let file_name = match args.len() > 2 {
        true => &args[2],
        false => "byteos.yaml",
    };

    let bin = if args.len() > 3 { &args[3] } else { "default" };

    let byteos_config = config::read_toml(file_name)?;

    let mut rustflags = Vec::new();
    // default configuration.
    rustflags.push(String::from("-Cforce-frame-pointers=yes"));
    rustflags.push(String::from("-Clink-arg=-no-pie"));

    // Convert byteos configuration to rustflags.
    // This rustflags will be passed to the rust build command.
    let binary_config = byteos_config
        .get_bin_config(bin)
        .ok_or(format!("Can't find bin target {bin}"))?;
    for (key, value) in binary_config.configs {
        rustflags.push(format!("--cfg={}=\"{}\"", key, value));
        println!("{} = {:?}", key, value);
    }

    // build os
    let mut outputs = Command::new("cargo")
        .env("RUSTFLAGS", rustflags.join(" "))
        .arg("build")
        .arg("-Z")
        .arg("build-std")
        .arg("--target")
        .arg(binary_config.target)
        .arg("--release")
        .spawn()
        .expect("can't build byteos");

    // Wait for build complete.
    let exit_status = outputs.wait().expect("can't wait for build");
    if !exit_status.success() {
        return Err(format!("build bin target {bin} failed, {exit_status}"));
    }

    if let Some(run_command) = binary_config.run {
        let args: Vec<&str> = run_command.split(" ").collect();
        let mut cmd = Command::new(args[0]);
        if args.len() > 1 {
            cmd.args(&args[1..]);
        }
        let mut outputs = cmd.spawn().expect("can't run byteos");
        let _ = outputs.wait();
    }

    // Check target configuration. And target will be passed to cargo build command.
    // dbg!(rustflags.join(" "));
    // dbg!(rustflags);
    // dbg!(args);
    Ok(())
}

inventory::submit! {
    CommandAndHandler::new("build", "build the byteos through a yaml.", handler)
}
