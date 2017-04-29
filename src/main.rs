mod reader;
mod model;

#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;

use reader::*;
use model::*;
use std::io::BufReader;
use std::fs::File;

fn main() {

    let f = File::open(r"C:\Users\Aleksander\Documents\projects\logparse\log.txt").unwrap();
    let r = BufReader::new(f);

    let mut reader = EventReader::new(r);
    let mut builder = Builder::new();

    while let Some(e) = reader.next() {
        match e {
            Event::PackageStarted(e) => builder.start_package(&e),
            Event::ContainerFinished(e) => builder.container_name(&e),
            Event::PreExecuteTask(e) => builder.pre_task(&e),
            Event::PostExecuteTask(e) => builder.post_task(&e)
        }
    }
    println!("{:?}", builder.packages)
}
