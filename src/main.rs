mod reader;
mod model;

#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate itertools;

use reader::*;
use model::*;
use std::io::BufReader;
use std::fs::File;
use itertools::Itertools;

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
    println!("{:?}", builder.packages);

    let result = builder.packages.iter()
        .flat_map(|p| {
            get_leaf_tasks(&p.tasks)
                .iter()
                .map(|t| (
                    (&p.package_name, &p.container_name, &t.name),
                    t.end_time.signed_duration_since(t.start_time).num_milliseconds()
                ))
                .collect::<Vec<_>>()
        })
        //.collect_vec().iter()
        //.group_by(|x| x.0)
        .collect_vec();

    println!("{:?}", result);
}

fn get_leaf_tasks(tasks : &Vec<Task>) -> Vec<&Task> {
        tasks.iter().flat_map(|t|
            if t.tasks.is_empty() {
                vec![t]
            } else {
                get_leaf_tasks(&t.tasks)
            }).collect::<Vec<_>>()
}
