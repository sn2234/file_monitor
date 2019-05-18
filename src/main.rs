#![allow(non_snake_case,dead_code,non_camel_case_types,non_upper_case_globals)]

#[macro_use]
extern crate log;
extern crate log4rs;

mod locations;

fn main() {
    initLogging();

    info!("Starting application");

    println!("Hello, world!");
}

fn initLogging() {
    log4rs::init_file("log4rs.yml", Default::default()).unwrap();
}
