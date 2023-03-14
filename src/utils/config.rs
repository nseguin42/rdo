use std::path::PathBuf;
use config::Config;

use crate::utils::error::Error;

#[allow(dead_code)]
pub enum ConfigType {
    Production,
    Test,
    Default,
}

impl ToString for ConfigType {
    fn to_string(&self) -> String {
        match self {
            ConfigType::Production => "config".to_string(),
            ConfigType::Test => "config.test".to_string(),
            ConfigType::Default => "config.default".to_string(),
        }
    }
}

pub fn look_for_config() -> Result<Config, Error> {
    let config_dirs = vec![
        home::home_dir().unwrap().join(".config").join("rdo"),
        PathBuf::from("/etc/rdo"),
        PathBuf::from("config"),
    ];

    for path in config_dirs {
        debug!("Looking for config in {:?}", path);
        let path = path.join("config.toml");
        if path.exists() {
            println!("Found config at {:?}", path);
            return Config::builder()
                .add_source(config::File::from(path))
                .build()
                .map_err(Error::Config);
        }
    }

    Err(Error::ConfigNotFound)
}

pub fn get_config(config_type: ConfigType) -> Result<Config, Error> {
    match config_type {
        ConfigType::Production => look_for_config(),
        ConfigType::Test => get_config_from_file("config.test"),
        ConfigType::Default => get_config_from_file("config.default"),
    }
}

pub fn get_config_from_file(path: &str) -> Result<Config, Error> {
    let config = Config::builder()
        .add_source(config::File::with_name(path))
        .build();

    match config {
        Ok(config) => Ok(config),
        Err(err) => Err(Error::Config(err)),
    }
}

pub fn get_config_or_default(config_path: Option<String>) -> Result<Config, Error> {
    match config_path {
        Some(path) => get_config_from_file(&path),
        None => get_config(ConfigType::Production),
    }
}
