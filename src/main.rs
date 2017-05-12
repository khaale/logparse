mod reader;
mod model;

#[macro_use] extern crate lazy_static;
extern crate chrono;
extern crate regex;
extern crate itertools;
#[macro_use] extern crate clap;

use reader::*;
use model::*;
use std::io::BufReader;
use std::fs;
use std::env;
use itertools::Itertools;
use std::iter::Iterator;

fn main() {

   let matches = clap_app!(logparser =>
            (version: "1.0")
            (author: "Aleksander Khanteev <khaale@yandex.ru>")
            (about: "Parses logs, extracts timing stats")
            (@arg DIR_PATH: -p --path +takes_value "Sets path to search log files")
        ).get_matches();

    let root_path = matches.value_of("path").unwrap_or(r"C:\Work\Projects\logparser");
    println!("Using root path: {}", root_path);

    let paths = fs::read_dir(root_path).unwrap();

    let packages: Vec<Package> = paths
        .map(|x| x.unwrap().path())
        .filter(|x|
            match x.extension() {
                Some(ext) if ext == "log" =>
                    { println!("Processing: {}", x.display()); true },
                _ =>
                    { println!("Skipping: {}", x.display()); false }
        })
        .flat_map(
            |x| match x.to_str() {
                Some(p) => get_packages_from_file(p),
                None => Vec::new()
            })
        .collect::<Vec<_>>();

    let sorted = packages.iter()
        .flat_map(|p| {
            get_leaf_tasks(&p.tasks)
                .iter()
                .map(|t| (
                    (&p.package_name, &p.container_name, &t.name),
                    t.end_time.signed_duration_since(t.start_time).num_seconds()
                ))
                .collect::<Vec<_>>()
        })
        .group_by(|x| x.0)
        .into_iter()
        .map(|g| (g.0, g.1.map(|x| x.1).sum::<i64>()))
        .sorted_by(|x1, x2| x2.1.cmp(&x1.1));

    sorted
        .iter()
        .take(10)
        .foreach(|x| println!(
            "{}\t{}\t{}\t{:.2}",
            (x.0).0,
            (x.0).1,
            (x.0).2,
            (x.1 as f64) / 60f64
        ));
}

fn get_packages_from_file(path: &str) -> Vec<Package>{
    let f = fs::File::open(path).unwrap();
    let r = BufReader::new(f);

    let mut reader = EventReader::new(r);
    let mut builder = Builder::new();

    while let Some(e) = reader.next() {
        let result = match e {
            Event::PackageStarted(e) => builder.start_package(&e),
            Event::ContainerFinished(e) => builder.container_name(&e),
            Event::PreExecuteTask(e) => builder.pre_task(&e),
            Event::PostExecuteTask(e) => builder.post_task(&e)
        };

        match result {
            Ok(_) => {},
            Err(err) => {
                println!("Error on parsing {}: {:?}", path, err);
                return Vec::new()
            }
        }
    }

    builder.packages
}

fn get_leaf_tasks(tasks : &Vec<Task>) -> Vec<&Task> {
    tasks.iter()
        .flat_map(|t|
            if t.tasks.is_empty() {
                vec![t]
            } else {
                get_leaf_tasks(&t.tasks)
            })
        .collect::<Vec<_>>()
}
