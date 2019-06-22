#![allow(non_snake_case,dead_code,non_camel_case_types,non_upper_case_globals)]

#[macro_use]
extern crate log;
extern crate log4rs;
extern crate chrono;

mod locations;
mod processor;

use std::process::exit;

fn main() {
    initLogging();

    info!("Starting application");

    let locationsConfig = locations::Locations::fromFile("locations.json").unwrap();

    info!("Config: {:?}", locationsConfig);

    if !processor::processLocations(locationsConfig) {
        exit(-1);
    }
}

fn initLogging() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
}
