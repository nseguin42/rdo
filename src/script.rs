use std::io::Read;
use std::os::unix::prelude::PermissionsExt;
use std::process::Command;

use config::Config;
use serde::{Deserialize, Serialize};

use crate::error::Error;
use crate::runnable::Runnable;
use crate::runner::WithDependencies;
use crate::task::Task;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum ScriptType {
    Bash,
}

#[derive(Debug, Clone)]
pub struct Script {
    pub name: String,
    pub cmd: String,
    pub path: Option<String>,
    pub script_type: ScriptType,
    pub args: Vec<String>,
    pub dependencies: Vec<String>,
    pub enabled: bool,
}

impl Script {
    pub fn new(
        name: &str,
        cmd: Option<String>,
        path: Option<String>,
        script_type: ScriptType,
        args: Vec<String>,
        dependencies: Vec<String>,
        enabled: bool,
    ) -> Script {
        if (path.is_none() && cmd.is_none()) || (path.is_some() && cmd.is_some()) {
            panic!("Exactly one of path, cmd must be set");
        }

        let mut cmd = cmd.unwrap_or_default();
        let mut path = path;

        if path.is_some() {
            path = Option::from(
                std::fs::canonicalize(path.clone().unwrap())
                    .unwrap()
                    .to_str()
                    .unwrap()
                    .to_string(),
            );

            if !is_file(&path) {
                panic!("{} is not a file", path.unwrap());
            }

            if !is_executable(&path) {
                panic!("{} is not executable", path.unwrap());
            }

            cmd = load_script_file(&path).unwrap();
        }

        Script {
            name: name.to_string(),
            cmd,
            path,
            script_type,
            args,
            dependencies,
            enabled,
        }
    }
}

fn load_script_file(path: &Option<String>) -> Result<String, Error> {
    let mut file = std::fs::File::open(path.clone().unwrap())?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)?;
    Ok(contents)
}

impl Runnable for Script {
    fn run(&self) -> Result<(), Error> {
        let output = Command::new("bash")
            .arg("-c")
            .arg(&self.cmd)
            .arg("--")
            .args(&self.args)
            .output()
            .expect("failed to execute process");

        if output.status.code().unwrap_or(1) != 0 {
            println!("{}", output.status);
        }

        if !output.stderr.is_empty() {
            println!("stderr: {}", String::from_utf8_lossy(&output.stderr));
        }

        if !output.stdout.is_empty() {
            println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
        }

        Ok(())
    }
}

fn is_file(path: &Option<String>) -> bool {
    std::path::Path::new(path.as_ref().unwrap()).is_file()
}

fn is_executable(path: &Option<String>) -> bool {
    std::fs::metadata(path.as_ref().unwrap())
        .map(|m| m.permissions().mode() & 0o111 != 0)
        .unwrap_or(false)
}

pub fn load_from_config(name: &str, config: &Config) -> Result<Script, Error> {
    let path = config
        .get::<Option<String>>(&format!("script.{}.path", name))
        .unwrap_or_default();
    let cmd = config
        .get::<Option<String>>(&format!("script.{}.cmd", name))
        .unwrap_or_default();
    let script_type = config.get::<ScriptType>(&format!("script.{}.type", name))?;
    let args = config
        .get_array(&format!("script.{}.args", name))
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let dependencies = config
        .get_array(&format!("script.{}.dependencies", name))
        .unwrap_or_default()
        .iter()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    let enabled = config
        .get_bool(&format!("script.{}.enabled", name))
        .unwrap_or_default();

    debug!(
        "Loaded script: {} ({}), type: {:?}, args: {:?}, dependencies: {:?}",
        name,
        &path.clone().unwrap_or_default(),
        script_type,
        args,
        dependencies
    );
    Ok(Script::new(
        name,
        cmd,
        path,
        script_type,
        args,
        dependencies,
        enabled,
    ))
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
        Task::new(&script.name, script.clone(), script.enabled)
    }
}

impl WithDependencies for Script {
    fn get_dependencies(&self) -> Vec<String> {
        self.dependencies.clone()
    }
}
