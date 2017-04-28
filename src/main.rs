mod reader;
mod model;

#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;

use reader::*;

use std::io::BufReader;
use std::fs::File;

fn main() {
    let f = File::open(r"C:\Work\Projects\logparser\log.txt").unwrap();
    let r = BufReader::new(f);

    let mut r = EventReader::new(r);

    while let Some(e) = r.next() {
        match e {
            Event::PackageStarted(e) => println!("{}", e.value),
            Event::ContainerFinished(e) => println!("{}", e.value),
            Event::PreExecuteTask(e) => println!("{}", e.value),
            Event::PostExecuteTask(e) => println!("{}", e.value)
        }
    }
}
