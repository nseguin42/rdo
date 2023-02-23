use config::Config;

use crate::utils::error::Error;

#[allow(dead_code)]
pub enum ConfigType {
    Production,
    Test,
    Default,
}

const CONFIG_DIR: &str = "config";

impl ToString for ConfigType {
    fn to_string(&self) -> String {
        match self {
            ConfigType::Production => "config".to_string(),
            ConfigType::Test => "config.test".to_string(),
            ConfigType::Default => "config.default".to_string(),
        }
    }
}

pub fn get_config(config_type: ConfigType) -> Result<Config, Error> {
    let path = format!("{}/{}", CONFIG_DIR, config_type.to_string());
    let config = Config::builder()
        .add_source(config::File::with_name(&path))
        .build();

    match config {
        Ok(config) => Ok(config),
        Err(err) => Err(Error::Config(err)),
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
