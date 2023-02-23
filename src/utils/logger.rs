use log::LevelFilter;
use pretty_env_logger::formatted_builder;

pub fn setup_logger(level: Option<LevelFilter>) {
    formatted_builder()
        .filter_level(level.unwrap_or(LevelFilter::Info))
        .init();

    info!("Logger initialized");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_setup_logger() {
        setup_logger(None);
    }
}
