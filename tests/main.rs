use rdo::config::{get_config, ConfigType};
use rdo::logger::setup_logger;
use rdo::runner::TaskRunner;
use rdo::script::{load_all_from_config, Script};
use rdo::task::Task;

#[test]
fn test_run_all_scripts() {
    let config = get_config(ConfigType::Test).unwrap();
    setup_logger(&config);

    let scripts = load_all_from_config(&config)
        .unwrap()
        .into_iter()
        .map(|script| script.into())
        .collect::<Vec<Task<Script>>>();

    // Borrow each script
    let scripts = scripts.iter().collect::<Vec<_>>();

    let runner = TaskRunner::new_with_dependencies(scripts);
    runner.run_all().unwrap();
}
