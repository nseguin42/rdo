#[macro_use]
extern crate log;

use crate::config::{get_config, ConfigType};
use crate::logger::setup_logger;

pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod logger;
pub(crate) mod runnable;

pub mod runner;
pub mod script;
pub mod task;
pub mod task_queue;

fn main() {
    let config = get_config(ConfigType::Production).unwrap();
    setup_logger(&config);
}
