use std::io::Write;

use config::Config;
use pretty_env_logger::env_logger;




pub fn setup_logger(config: Config) {
    let log_level = config.get_string("log.level");

    env_logger::Builder::from_default_env()
        .filter_level(log_level.unwrap().parse().unwrap())
        .format(|buf, record| {
            writeln!(
                buf,
                "[{}][{}] {}",
                record.level(),
                record.target(),
                record.args()
            )
        })
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
        setup_logger(config);
    }
}
