use std::io::Read;
use std::os::unix::prelude::PermissionsExt;
use std::process::Stdio;

use async_trait::async_trait;
use config::Config;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, Lines};
use tokio::process::{ChildStderr, ChildStdin, ChildStdout, Command};
use tokio::sync::mpsc::Sender;
use tokio::sync::watch::Receiver as WatchReceiver;

use crate::runnable::Runnable;
use crate::utils::error::Error;
use crate::utils::graph_binding::GraphLike;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Hash, Default)]
pub enum ScriptType {
    #[default]
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
        script_type: Option<ScriptType>,
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
            script_type: script_type.unwrap_or_default(),
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

#[async_trait]
impl Runnable for Script {
    async fn run(
        &self,
        mut stdin_rx: WatchReceiver<String>,
        output_tx: Sender<String>,
    ) -> Result<(), Error> {
        debug!("Starting script: {}", self.name);
        let mut child = Command::new("sh")
            .arg("-c")
            .arg(&self.cmd)
            .arg("--")
            .args(&self.args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .unwrap();

        let mut stdout = BufReader::new(child.stdout.take().unwrap()).lines();
        let mut stderr = BufReader::new(child.stderr.take().unwrap()).lines();
        let stdin = child.stdin.take().unwrap();

        tokio::select! {
            e = handle_io(&mut stdin_rx, stdin, &mut stdout, &mut stderr, output_tx) => {
                e
            }
            _ = child.wait() => {
                debug!("Script {} finished", self.name);
                Ok(())
            }
        }
    }
}

/// Pass stdin into the script and return stdout/stderr through output_tx.
async fn handle_io(
    stdin_rx: &mut WatchReceiver<String>,
    stdin: ChildStdin,
    stdout: &mut Lines<BufReader<ChildStdout>>,
    stderr: &mut Lines<BufReader<ChildStderr>>,
    output_tx: Sender<String>,
) -> Result<(), Error> {
    tokio::select! {
        r = handle_stdin(stdin_rx, stdin) => {
            r
        }
        r = handle_stdout(stdout, &output_tx) => {
            r
        }
        r = handle_stderr(stderr, &output_tx) => {
            r
        }
    }
}

async fn handle_stdin(
    stdin_rx: &mut WatchReceiver<String>,
    mut stdin: ChildStdin,
) -> Result<(), Error> {
    loop {
        let changed = stdin_rx.changed().await;
        match changed {
            Ok(_) => {}
            Err(e) => {
                error!("Script stdin closed: {}", e);
                return Err(Error::StdinClosed);
            }
        }

        let line = stdin_rx.borrow().clone();
        let result = stdin.write_all(line.as_bytes()).await;
        if result.is_err() {
            return Err(Error::from(result.err().unwrap()));
        }
    }
}

async fn handle_stdout(
    stdout: &mut Lines<BufReader<ChildStdout>>,
    output_tx: &Sender<String>,
) -> Result<(), Error> {
    loop {
        let line = stdout.next_line().await?;

        match line {
            Some(line) => {
                output_tx
                    .send_timeout(line, std::time::Duration::from_millis(100))
                    .await
                    .unwrap_or_else(|e| {
                        error!("Script stdout send error: {}", e);
                    });
            }
            None => {
                return Ok(());
            }
        }
    }
}

async fn handle_stderr(
    stderr: &mut Lines<BufReader<ChildStderr>>,
    output_tx: &Sender<String>,
) -> Result<(), Error> {
    loop {
        let line = stderr.next_line().await?;

        match line {
            Some(line) => {
                output_tx
                    .send_timeout(line, std::time::Duration::from_millis(100))
                    .await
                    .unwrap_or_else(|e| {
                        error!("Script stderr send error: {}", e);
                    });
            }
            None => {
                return Ok(());
            }
        }
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

pub fn load_script_from_config(name: &str, config: &Config) -> Result<Script, Error> {
    let path = config
        .get::<Option<String>>(&format!("script.{}.path", name))
        .unwrap_or_default();
    let cmd = config
        .get::<Option<String>>(&format!("script.{}.cmd", name))
        .unwrap_or_default();
    let script_type = config
        .get::<Option<ScriptType>>(&format!("script.{}.type", name))
        .unwrap_or_default();

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

pub fn load_scripts_from_config(
    config: &Config,
    scripts_to_add: Vec<String>,
) -> Result<Vec<Script>, Error> {
    let mut scripts = Vec::new();
    for (name, _) in config.get_table("script")? {
        if scripts_to_add.contains(&name) {
            scripts.push(load_script_from_config(&name, config)?);
        }
    }
    Ok(scripts)
}

pub fn load_all_scripts_from_config(config: &Config) -> Result<Vec<Script>, Error> {
    config
        .get_table("script")?
        .keys()
        .map(|name| load_script_from_config(name.as_str(), config))
        .collect()
}

impl<'a> GraphLike<'a, String> for Script {
    fn get_key(&'a self) -> &'a String {
        &self.name
    }

    fn get_children_keys(&'a self) -> Vec<&'a String> {
        self.dependencies.iter().collect()
    }
}
