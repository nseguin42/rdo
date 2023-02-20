use rdo::config::{get_config, ConfigType};
use rdo::logger::setup_logger;
use rdo::runner::TaskRunner;
use rdo::script::{load_all_from_config, Script};
use rdo::task::Task;

#[test]
fn test_run_all_scripts() {
    let config = get_config(ConfigType::Production).unwrap();
    setup_logger(&config);

    let scripts = load_all_from_config(&config)
        .unwrap()
        .into_iter()
        .map(|script| script.into())
        .collect::<Vec<Task<Script>>>();

    let runner = TaskRunner::new(scripts.iter().collect::<Vec<_>>());
    runner.run_all().unwrap();
}
