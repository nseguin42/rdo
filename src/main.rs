#[macro_use]
extern crate log;

use crate::logger::setup_logger;

pub(crate) mod config;
pub(crate) mod error;
pub(crate) mod logger;

pub mod task;

fn main() {
    let config = config::get_config(config::ConfigType::Production).unwrap();
    setup_logger(config);
}
