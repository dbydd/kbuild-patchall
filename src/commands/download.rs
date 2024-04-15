use std::{env::current_dir, fs, path::PathBuf, process::Command, sync::Mutex};

use color_print::cprintln;
use yaml_rust::{Yaml, YamlLoader};

use crate::{utils, CommandAndHandler};

pub struct CloneItem {
    path: PathBuf,
    url: String,
    commit: Option<String>,
}

static CLONE_LIST: Mutex<Vec<CloneItem>> = Mutex::new(Vec::new());

fn clone_from_git(path: &PathBuf, url: &str, commit: Option<&str>) -> Result<(), String> {
    // Printing the clone information
    cprintln!(
        "<green>Cloning {}:{} -> {}</green>",
        url,
        commit.unwrap_or("latest"),
        path.display()
    );

    // Clone the origin repository into path
    let current_dir = current_dir().map_err(|x| x.to_string())?;
    let mut outputs = Command::new("git")
        .arg("clone")
        .arg(url)
        .arg(path.to_str().expect("This is not a valid path"))
        .spawn()
        .expect("can't clone from git repository");
    outputs.wait().expect("can't wait for git clone");

    // Change the commit if necessary
    if let Some(commit) = commit {
        let mut outputs = Command::new("git")
            .current_dir(current_dir.join(path))
            .arg("reset")
            .arg("--hard")
            .arg(commit)
            .spawn()
            .expect(&format!("can't switch crate {url} to commit {commit}"));
        outputs.wait().expect("can't wait for switch to commit");
    }
    Ok(())
}

fn checkout_from_yaml(path: &PathBuf, tree: &Yaml) -> Result<(), String> {
    if tree.is_badvalue() || tree.is_null() {
        return Ok(());
    }
    if !tree["git"].is_badvalue() {
        CLONE_LIST.lock().unwrap().push(CloneItem {
            path: path.clone(),
            url: tree["git"].as_str().unwrap().to_string(),
            commit: tree["commit"].as_str().map(String::from),
        });
    }
    for (key, value) in tree.as_hash().expect("can't find a hashable table") {
        if key.is_badvalue() {
            return Err(format!("bad key format: {key:?}"));
        }
        let key = key.as_str().unwrap();
        if key == "commit" || key == "git" {
            continue;
        }
        checkout_from_yaml(&path.join(key), value)?;
    }
    Ok(())
}

/// The handler of the command.
fn handler(args: Vec<String>) -> Result<(), String> {
    let file_name = match args.len() > 2 {
        true => &args[2],
        false => "byteos.yaml",
    };

    // clear the clone list.
    CLONE_LIST.lock().unwrap().clear();

    let content = fs::read_to_string(file_name).expect("can't find at config file");
    let docs = YamlLoader::load_from_str(&content).expect("The file is not a valid YAML file");
    let byteos_doc = &docs[0];
    // dbg!(byteos_doc);
    let doc_hash = byteos_doc
        .as_hash()
        .expect("can't find any crate tree definitions");
    for (key, value) in doc_hash {
        // println!("key: {key:?}");
        checkout_from_yaml(&PathBuf::from(key.as_str().unwrap()), value)?;
    }
    CLONE_LIST.lock().unwrap().iter().for_each(|clone_item| {
        cprintln!(
            "<bold>{} -> {}:{}</bold>",
            clone_item.path.display(),
            clone_item.url,
            clone_item
                .commit
                .as_ref()
                .map(String::as_str)
                .unwrap_or("latest")
        );
    });
    let confirmed = utils::confirm("Ensure to download?", true);
    println!("confirmed: {}", confirmed);

    if confirmed {
        // TODO: Clone git repositories
        CLONE_LIST.lock().unwrap().iter().for_each(|clone_item| {
            clone_from_git(
                &clone_item.path,
                &clone_item.url,
                clone_item.commit.as_ref().map(String::as_str),
            )
            .unwrap();
        });
    }
    Ok(())
}

inventory::submit! {
    CommandAndHandler::new("download", "Donwload the byteos through a yaml.", handler)
}
