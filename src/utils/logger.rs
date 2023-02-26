use std::str::FromStr;

use config::Config;
use log::LevelFilter;
use pretty_env_logger::formatted_builder;

pub fn setup_logger(config: &Config) {
    let level_str = config.get("log.level").unwrap_or("info".to_string());
    formatted_builder()
        .filter_level(LevelFilter::from_str(&level_str).unwrap())
        .init();

    info!("Logger initialized");
}
