#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

use std::cell::RefCell;
use std::result;
use chrono::prelude::*;
use reader::{LogEvent,SsisEvent};

#[derive(Debug)]
pub enum BuildError {
    BadFileStructure(&'static str)
}

pub type Result = result::Result<(),BuildError>;

#[derive(Debug)]
pub struct Package {
    pub package_name: String,
    pub container_name: String,
    pub tasks: Vec<Task>,
}

#[derive(Debug,Clone)]
pub struct Task {
    pub name: String,
    pub start_time: DateTime<FixedOffset>,
    pub end_time: DateTime<FixedOffset>,
    pub tasks: Vec<Task>
}

impl Task {

    fn set_end_time(&mut self, time: DateTime<FixedOffset>) {
        self.end_time = time;
    }
}

impl Package {

    fn set_container_name(&mut self, name: String) {
        self.container_name = name;
    }

    fn set_tasks(&mut self, tasks: Vec<Task>) {
        self.tasks = tasks;
    }
}

pub struct Builder {
    pub packages: Vec<Package>,
    tasks_stack: Vec<Task>
}

impl Builder {

    pub fn new() -> Builder {
        Builder {
            packages: Vec::new(),
            tasks_stack: Vec::new()
        }
    }

    pub fn start_package(&mut self, evt: &LogEvent) -> Result {
        let package = Package {
            package_name: evt.value.clone(),
            container_name: String::new(),
            tasks: Vec::new()
        };
        self.packages.push(package);
        Ok(())
    }

    pub fn pre_task(&mut self, evt: &SsisEvent) -> Result {

        if self.packages.len() == 0 {
            return Err(BuildError::BadFileStructure("No package for pre_task event"))
        } 

        let task = Task {
            tasks: Vec::new(),
            name: evt.value.clone(),
            start_time: evt.time.clone(),
            end_time: evt.time
        };
        self.tasks_stack.push(task);
        Ok(())
    }

    pub fn post_task(&mut self, evt: &SsisEvent) -> Result {
        let mut task = match self.tasks_stack.pop() {
            Some(x) => x,
            None => return Err(BuildError::BadFileStructure("No matching pretask for post_task event"))
        };

        task.set_end_time(evt.time);

        if let Some(parent_task) = self.tasks_stack.last_mut() {
            parent_task.tasks.push(task)
        } else {
            match self.packages.last_mut() {
                Some(x) => x.tasks.push(task),                
                None => return Err(BuildError::BadFileStructure("No package for post_task event"))
            }
        }
        Ok(())
    }

    pub fn container_name(&mut self, evt: &LogEvent) -> Result {
        match self.packages.last_mut() {
            Some(package) => {
                package.set_container_name(evt.value.clone());
                Ok(())
            },
            None => Err(BuildError::BadFileStructure("No package for container_name event"))
        }
    }
}

#[cfg(test)]
#[allow(unused_must_use)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use reader::{LogEvent,SsisEvent};

    macro_rules! log_event (
        () => ({
            static DATA: &'static str = "package";
            LogEvent { value: DATA.to_string() }
        });
        ($value:expr) => ({
            static DATA: &'static str = $value;
            LogEvent { value: DATA.to_string() }
        })
    );

    macro_rules! ssis_event (
        () => ({
            static DATA: &'static str = "task";
            SsisEvent {
                value: DATA.to_string(),
                time: FixedOffset::east(9 * 3600).ymd(2017, 4, 26).and_hms_milli(0, 0, 0, 0)
                }
        });
        ($value:expr) => ({
            static DATA: &'static str = $value;
            SsisEvent {
                value: DATA.to_string(),
                time: FixedOffset::east(9 * 3600).ymd(2017, 4, 26).and_hms_milli(0, 0, 0, 0)
                }
        });
        ($value:expr, $time:expr) => ({
            static DATA: &'static str = $value;
            SsisEvent {
                value: DATA.to_string(),
                time: $time
                }
        })
    );

    #[test]
    fn should_create_package_after_start_package_and_container_name() {
        let pe = log_event!("package");
        let ce = log_event!("container");
        let mut sut = Builder::new();

        sut.start_package(&pe);
        sut.container_name(&ce);

        let actual_package_option = sut.packages.pop();
        assert!(actual_package_option.is_some());
        let actual_package = actual_package_option.unwrap();
        assert!(actual_package.package_name == "package");
        assert!(actual_package.container_name == "container");
    }

    #[test]
    fn should_add_task_to_package() {
        let time_start = FixedOffset::east(9 * 3600).ymd(2017, 4, 26).and_hms_milli(0, 0, 0, 0);
        let time_end = FixedOffset::east(9 * 3600).ymd(2014, 4, 26).and_hms_milli(0, 1, 1, 1);
        let pe = log_event!();
        let pre_task_evt = ssis_event!("task_1", time_start);
        let post_task_evt = ssis_event!("task_1", time_end);
        let ce = log_event!();
        let mut sut = Builder::new();

        sut.start_package(&pe);
        sut.pre_task(&pre_task_evt);
        sut.post_task(&post_task_evt);
        sut.container_name(&ce);

        let tasks = sut.packages.pop().unwrap().tasks;
        let actual_task_option = tasks.last();
        assert!(actual_task_option.is_some());
        let actual_task = actual_task_option.unwrap();
        assert!(actual_task.name == "task_1");
        assert!(actual_task.start_time == time_start);
        assert!(actual_task.end_time == time_end);
    }

    #[test]
    fn should_add_many_tasks_to_package() {

        let mut sut = Builder::new();

        sut.start_package(&log_event!());

        //task1
        sut.pre_task(&ssis_event!("task1"));
        sut.post_task(&ssis_event!("task1"));

        //task2
        sut.pre_task(&ssis_event!("task2"));
        sut.post_task(&ssis_event!("task2"));

        sut.container_name(&log_event!());

        let tasks = sut.packages.pop().unwrap().tasks;
        assert!(tasks.len() == 2);
        let actual_task1 = tasks.get(0).unwrap();
        assert!(actual_task1.name == "task1");
        let actual_task2 = tasks.get(1).unwrap();
        assert!(actual_task2.name == "task2");

    }

    #[test]
    fn should_add_nested_task() {

        let mut sut = Builder::new();

        sut.start_package(&log_event!());

        sut.pre_task(&ssis_event!("task1"));
        sut.pre_task(&ssis_event!("task2"));
        sut.post_task(&ssis_event!("task2"));
        sut.post_task(&ssis_event!("task1"));

        sut.container_name(&log_event!());

        println!("{:?}",&sut.packages);
        let tasks = sut.packages.pop().unwrap().tasks;
        assert!(tasks.len() == 1);
        let actual_task1 = tasks.get(0).unwrap();
        assert!(actual_task1.name == "task1");
        assert!(actual_task1.tasks.len() == 1);
        let actual_task2 = actual_task1.tasks.get(0).unwrap();
        assert!(actual_task2.name == "task2");
    }

    #[test] 
    fn should_return_error_on_pre_start_task_when_package_not_started()
    {
        let mut sut = Builder::new();

        let result = sut.pre_task(&ssis_event!("task1"));

        match result {
            Err(_) => assert!(true),
            Ok(_) => assert!(false,"Expected error")
        }
    }

    #[test] 
    fn should_return_error_on_post_start_task_when_package_not_started()
    {
        let mut sut = Builder::new();

        sut.pre_task(&ssis_event!());
        let result = sut.post_task(&ssis_event!());

        match result {
            Err(_) => assert!(true),
            Ok(_) => assert!(false,"Expected error")
        }
    }

    #[test] 
    fn should_return_error_on_post_start_task_when_pre_task_not_called()
    {
        let mut sut = Builder::new();

        sut.start_package(&log_event!());
        let result = sut.post_task(&ssis_event!());

        match result {
            Err(_) => assert!(true),
            Ok(_) => assert!(false,"Expected error")
        }
    }

    #[test] 
    fn should_return_error_on_container_name_when_package_not_started()
    {
        let mut sut = Builder::new();

        let result = sut.container_name(&log_event!());

        match result {
            Err(_) => assert!(true),
            Ok(_) => assert!(false,"Expected error")
        }
    }
}