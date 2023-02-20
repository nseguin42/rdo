use std::env::args;
use std::os::unix::prelude::PermissionsExt;
use std::process::Command;

use config::Config;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::runnable::Runnable;
use crate::task::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ScriptType {
    Bash,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub path: String,
    pub script_type: ScriptType,
    args: Vec<String>,
}

impl Script {
    pub fn new(name: &str, path: &str, script_type: ScriptType, args: Vec<String>) -> Script {
        let path = std::fs::canonicalize(path)
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        if !is_file(&path) {
            panic!("{} is not a file", path);
        }

        if !is_executable(&path) {
            panic!("{} is not executable", path);
        }

        Script {
            name: name.to_string(),
            path,
            script_type,
            args,
        }
    }
}

impl Runnable for Script {
    fn run(&self) -> Result<(), Error> {
        let output = Command::new("bash")
            .arg(&self.path)
            .args(&self.args)
            .output()
            .expect("failed to execute process");

        println!("status: {}", output.status);
        println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

        Ok(())
    }
}

fn is_file(path: &str) -> bool {
    std::path::Path::new(path).is_file()
}

fn is_executable(path: &str) -> bool {
    std::fs::metadata(path)
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn load_from_config(name: &str, config: &Config) -> Result<Script, Error> {
    let path = config.get_string(&format!("script.{}.path", name))?;
    let script_type = config.get::<ScriptType>(&format!("script.{}.type", name))?;
    let args = config
        .get_array(&format!("script.{}.args", name))?
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    debug!(
        "Loaded script: {} ({}), type: {:?}, args: {:?}",
        name, path, script_type, args
    );
    Ok(Script::new(name, &path, script_type, args))
}

pub fn load_all_from_config(config: &Config) -> Result<Vec<Script>, Error> {
    let mut scripts = Vec::new();
    for (name, _) in config.get_table("script")? {
        scripts.push(load_from_config(&name, config)?);
    }
    Ok(scripts)
}

impl From<Script> for Task<Script> {
    fn from(script: Script) -> Self {
        Task::new(&script.name, script.clone())
    }
}
