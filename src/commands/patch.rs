use std::{env::current_dir, fs, process::{Command, Stdio}};

use anyhow::Result;
use color_print::cprintln;
use serde::{Deserialize, Serialize};
use toml::Table;

use crate::CommandAndHandler;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoPackage {
    name: String,
    source: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CargoLock {
    package: Vec<CargoPackage>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PatchPackage {
    name: String,
    git: String,
    rev: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PatchedPackage {
    name: String,
    git: String,
    local: String,
}

fn get_patch_table() -> Result<Vec<CargoPackage>> {
    let cargo_lock: CargoLock = toml::from_str(&fs::read_to_string("Cargo.lock")?)?;
    let patch_table = cargo_lock
        .package
        .into_iter()
        .filter(|x| match x.source {
            Some(ref source) => source.starts_with("git+https://github.com"),
            None => false,
        })
        .collect();
    Ok(patch_table)
}

fn get_patched_table() -> Result<Vec<PatchedPackage>> {
    let mut res = Vec::new();
    let mut cargo_toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    if let Some(patch_table) = cargo_toml.get_mut("patch") {
        let patch_table = patch_table.as_table().unwrap();
        patch_table.into_iter().for_each(|(git, value)| {
            value.as_table().unwrap().keys().for_each(|name| {
                res.push(PatchedPackage {
                    name: name.to_string(),
                    git: git.to_string(),
                    local: format!("crates/{}", name),
                });
            });
        })
    }
    Ok(res)
}

pub fn check_patch_table() -> Result<()> {
    println!("Patch table avaliable in the below");
    get_patch_table()?.iter().for_each(|x| {
        cprintln!(
            "    <green><bold>{:20}</bold></green> {}",
            x.name,
            x.source.as_ref().unwrap()
        );
    });
    println!();
    println!("Patched table in the below");
    get_patched_table()?.iter().for_each(|x| {
        cprintln!(
            "    <green><bold>{:20}</bold></green> {:22} -> {}",
            x.name,
            x.local,
            x.git
        );
    });
    Ok(())
}

fn git_https_to_ssh(git: &str) -> Result<String> {
    const GIT_HEAD: &str = "https://github.com/";
    assert!(git.starts_with(GIT_HEAD));
    let ssh_url = String::from("git@github.com:") + &git[GIT_HEAD.len()..];
    println!("ssh url: {}", ssh_url);
    Ok(ssh_url)
}

pub fn do_patch(name: &str, git: &str, commit: &str) -> Result<()> {
    cprintln!(
        "<green>Patching {}:{} -> crates/{}</green>",
        git,
        commit,
        name
    );

    let mut outputs = Command::new("git")
        .arg("clone")
        .arg(git_https_to_ssh(git)?)
        .arg(format!("crates/{}", name))
        .spawn()?;
    outputs.wait()?;

    outputs = Command::new("git")
        .arg("reset")
        .arg("--hard")
        .arg(commit)
        .current_dir(current_dir()?.join(format!("crates/{}", name)))
        .spawn()?;
    outputs.wait()?;

    let mut cargo_toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
    if !cargo_toml.contains_key("patch") {
        cargo_toml.insert(String::from("patch"), toml::Value::Table(Table::new()));
    }
    let patch_table = cargo_toml.get_mut("patch").unwrap().as_table_mut().unwrap();
    if !patch_table.contains_key(git) {
        patch_table.insert(git.to_string(), toml::Value::Table(Table::new()));
    }
    let git_table = patch_table.get_mut(git).unwrap().as_table_mut().unwrap();
    git_table.insert(name.to_string(), toml::Value::Table(Table::new()));

    let detail_table = git_table.get_mut(name).unwrap().as_table_mut().unwrap();
    detail_table.insert(
        String::from("path"),
        toml::Value::String(format!("crates/{}", name)),
    );
    fs::write("Cargo.toml", toml::to_string(&cargo_toml)?)?;
    Ok(())
}

/// The command handler of the help command.
pub fn handler(args: Vec<String>) -> Result<()> {
    if args.len() == 2 {
        cprintln!("Patch commands availible below");
        cprintln!(
            "    <green>{:20}</green> {}",
            "list",
            "list patch available"
        );
        cprintln!(
            "    <green>{:20}</green> {}",
            "add",
            "Download and patch into Cargo.toml"
        );
        cprintln!(
            "    <green>{:20}</green> {}",
            "remove",
            "remove patch from Cargo.toml and delete folder"
        );
        return Ok(());
    }
    let mut spawn = Command::new("cargo")
        .arg("check")
        .spawn()
        .expect("can't spawn a command");
    spawn.wait().expect("can't wait for a command end");

    match args[2].as_str() {
        "list" => check_patch_table()?,
        "add" => {
            let patch_name = args[3].clone();

            // Check if the patch name is available
            let patch_table = get_patch_table()?;
            let patch = patch_table
                .iter()
                .find(|x| x.name == patch_name)
                .ok_or(anyhow!("Can't find matched patch name"))?;

            // Get the patch info from the specific package
            let urls: Vec<&str> = patch.source.as_ref().unwrap().split("#").collect();
            if urls.len() < 2 {
                return Err(anyhow!("This is not a valid patch source"));
            }
            let commit = urls[1];
            let git_url_end = urls[0].find('?').unwrap_or(urls[0].len());
            let git_url = &urls[0][4..git_url_end];

            // TODO: Check if the rev exists. use rev instead of the hash commit.
            // Do the patch
            do_patch(&patch_name, git_url, commit)?;
        }
        "remove" => {
            let patch_name = args[3].clone();

            // Check if the patch name is available
            let patched = get_patched_table()?
                .iter()
                .find(|x| x.name == patch_name)
                .ok_or(anyhow!(
                    "can't find any matched package named {}",
                    patch_name
                ))?.clone();
            let process = Command::new("git")
                .arg("status")
                .arg("-s")
                .current_dir(current_dir()?.join(format!("crates/{}", patch_name)))
                .stdout(Stdio::piped())
                .spawn()?;
            let outputs = process.wait_with_output()?;
            if outputs.stdout.len() != 0 {
                println!();
                println!("{}", String::from_utf8(outputs.stdout.clone())?);
                return Err(anyhow!("You have some files unhandle, please ensure this package don't have any item to handle"));
            }

            // remove patch from disk
            fs::remove_dir_all(format!("crates/{}", patch_name))?;

            // remove patch from Cargo.toml
            let mut cargo_toml: Table = toml::from_str(&fs::read_to_string("Cargo.toml")?)?;
            let patch_table = cargo_toml.get_mut("patch").unwrap().as_table_mut().unwrap();
            let git_table = patch_table.get_mut(&patched.git).unwrap().as_table_mut().unwrap();
            if git_table.len() == 1 {
                patch_table.remove(&patched.git);
            } else {
                git_table.remove(&patched.name);
            }
            fs::write("Cargo.toml", toml::to_string(&cargo_toml)?)?;
        }
        _ => {}
    }
    Ok(())
}

// submit the command to CommandAndHandler Iterator.
inventory::submit! {
    CommandAndHandler::new("patch", "Downlaod crate from git and patch in Cargo.toml.", handler)
}
