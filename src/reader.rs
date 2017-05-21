use std::io::prelude::*;
use chrono::prelude::*;
use regex::Regex;

#[derive(Debug)]
pub struct LogEvent {
    pub value: String
}
#[derive(Debug)]
pub struct SsisEvent {
    pub value: String,
    pub time: DateTime<FixedOffset>
}
#[derive(Debug)]
pub enum Event {
    PackageStarted(LogEvent),
    ContainerFinished(LogEvent),
    PreExecuteTask(SsisEvent),
    PostExecuteTask(SsisEvent)
}

pub struct EventReader<R: BufRead> {
    source: R
}

impl<R: BufRead> EventReader<R> {

    /// Creates a new reader, consuming the given stream.
    pub fn new(source: R) -> EventReader<R> {
        EventReader { source: source }
    }

    fn handle_log_event(& self, line: &str, rgx: &Regex) -> Option<LogEvent> {
        rgx.captures(line).and_then(|c|
            Some(LogEvent { 
                value: c.get(1).unwrap().as_str().to_string()
            }))
    }

    fn check_ssis_event(& self, line: &str) -> Option<DateTime<FixedOffset>> {
        if line.len() < 34 {
            return None;
        }

        let clean_line = line[34..].trim();
        //println!("clean_line: '{}'", clean_line);
        if clean_line == "PRE EXECUTE Container Start"
            || clean_line == "POST EXECUTE Container Start"
            || clean_line == "PRE EXECUTE Container End"
            || clean_line == "POST EXECUTE Container End"
            || clean_line == "PRE EXECUTE Package Start"
            || clean_line == "POST EXECUTE Package Start"
            || clean_line == "PRE EXECUTE Package End"
            || clean_line == "POST EXECUTE Package End" {
            return None;
        }

        DateTime::parse_from_rfc3339(&line[..33]).ok()
    }

    fn handle_ssis_event(& self, line: &str, dt: &DateTime<FixedOffset>, rgx: &Regex) -> Option<SsisEvent> {
        rgx.captures(&line[34..]).and_then(|c| 
            Some(SsisEvent {
                value: c.get(1).unwrap().as_str().to_string(),
                time: dt.clone()
            }))
    }

    fn handle_line(& self, line: &str) -> Option<Event> {

        lazy_static! {
            static ref PRE_EXECUTE_PACKAGE_RGX: Regex = Regex::new(r"Pre-execute package (.*)").unwrap();
            static ref CONTAINER_NAME_RGX: Regex = Regex::new(r"Container Name       : (.*)").unwrap();
            static ref PRE_EXECUTE_TASK_RGX: Regex = Regex::new(r"PRE EXECUTE (.*)").unwrap();
            static ref POST_EXECUTE_TASK_RGX: Regex = Regex::new(r"POST EXECUTE (.*)").unwrap();
        }

        match self.check_ssis_event(line) {
            //log events
            None => None
                    .or(self.handle_log_event(line, &*PRE_EXECUTE_PACKAGE_RGX).map(Event::PackageStarted))
                    .or(self.handle_log_event(line, &*CONTAINER_NAME_RGX).map(Event::ContainerFinished)),
            //SSIS events
            Some(dt) => None
                    .or(self.handle_ssis_event(line, &dt, &*PRE_EXECUTE_TASK_RGX).map(Event::PreExecuteTask))
                    .or(self.handle_ssis_event(line, &dt, &*POST_EXECUTE_TASK_RGX).map(Event::PostExecuteTask))
        }
    }

    pub fn next(&mut self) -> Option<Event> {
        let mut line = String::new();
        loop {
            match self.source.read_line(&mut line).unwrap() {
                0 => return None,
                _ => if let Some(x) = self.handle_line(&line.trim()) {
                    return Some(x)
                }
            }
            line.clear();
        }
    }
}

#[cfg(test)]
#[allow(unused_variables)]
mod tests {
    use super::*;
    use std::io::BufReader;

    macro_rules! test_data (
        ($d:expr) => ({
            static DATA: &'static str = $d;
            {
                let mut r1 = BufReader::new(DATA.as_bytes());
                let mut line = String::new();
                loop {
                    match r1.read_line(&mut line).unwrap() {
                        0 => break,
                        _ => println!(">> {}", line.trim())
                    }
                    line.clear();
                }
            }
            let r = BufReader::new(DATA.as_bytes());
            let p = EventReader::new(r);
            p
        })
    );

    macro_rules! expect_event (
        ( $p:expr, $t:pat) => (
            let result = $p.next();
            println!("Got '{:?}'", result);
            match result {
                $t => { }
                e => panic!("Unexpected event: {:?}", e)
            }
        );
        ($p:expr, $t:pat => $c:expr ) => (
            let result = $p.next();
            println!("Got '{:?}'", result);
            match result {
                $t => {}
                e => panic!("Unexpected event: {:?}", e)
            }
        )
    );

    #[test]
    fn should_parse_pre_execute_package() {
        let mut r = test_data!(r#"Pre-execute package Package"#);

        expect_event!(r, Some(Event::PackageStarted( LogEvent { value })) => value == "Package");
    }

    #[test]
    fn should_parse_container_name() {
        let mut r = test_data!(r#"Container Name       : Container"#);

        expect_event!(r, Some(Event::ContainerFinished( LogEvent{ value })) => value == "Container");
    }

    #[test]
    fn should_parse_pre_execute_task() {
        let mut r = test_data!(r#"2017-04-20T10:53:24.6607935+01:00 PRE EXECUTE Task"#);

        expect_event!(r, Some(Event::PreExecuteTask(SsisEvent { value, .. })) => value == "Task" );
    }

    #[test]
    fn should_parse_post_execute_task() {
        let mut r = test_data!(r#"2017-04-20T10:53:24.9420381+01:00 POST EXECUTE Task"#);

        expect_event!(r, Some(Event::PostExecuteTask(SsisEvent { value, .. })) => value == "Task");
    }

    #[test]
    fn should_ignore_not_needed_lines() {
        let mut r = test_data!("
Start...
2017-04-25T16:43:46.8297379+01:00 PRE EXECUTE Container Start
2017-04-25T16:43:52.3297111+01:00 PRE EXECUTE Package End
2017-04-25T16:43:52.3297111+01:00 POST EXECUTE Package End
2017-04-25T16:43:52.3297111+01:00 POST EXECUTE Container Start
End..
");

        expect_event!(r, None);
    }

    #[test]
    fn should_parse_multiple_events_and_end_with_none() {
        let mut r = test_data!("Start...
Pre-execute package Package
2017-04-20T10:53:24.6607935+01:00 PRE EXECUTE Task
2017-04-20T10:53:24.9420381+01:00 POST EXECUTE Task
Container Name       : Container
End..");
        expect_event!(r, Some(Event::PackageStarted(..)));
        expect_event!(r, Some(Event::PreExecuteTask(..)));
        expect_event!(r, Some(Event::PostExecuteTask(..)));
        expect_event!(r, Some(Event::ContainerFinished(..)));
        expect_event!(r, None);
    }
}
