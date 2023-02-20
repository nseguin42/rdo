use config::Config;
use pretty_env_logger::formatted_builder;

pub fn setup_logger(config: &Config) {
    let log_level = config.get_string("log.level");

    formatted_builder()
        .filter_level(log_level.unwrap().parse().unwrap())
        .init();

    info!("Logger initialized");
}

#[cfg(test)]
mod tests {
    use crate::config::{get_config, ConfigType};

    use super::*;

    #[test]
    fn test_setup_logger() {
        let config = get_config(ConfigType::Test).unwrap();
        setup_logger(&config);
    }
}
