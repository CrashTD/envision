use std::cell::RefCell;

use crate::runner::{Runner, RunnerStatus};

pub struct RunnerPipeline {
    runners: Vec<RefCell<Runner>>,
    current_index: usize,
    last_exit_status: Option<i32>,
    has_started: bool,
    pub log: Vec<String>,
}

impl RunnerPipeline {
    pub fn new(runners: Vec<Runner>) -> Self {
        let mut c_runners: Vec<RefCell<Runner>> = vec![];
        for runner in runners {
            c_runners.push(RefCell::new(runner));
        }
        Self {
            runners: c_runners,
            current_index: 0,
            has_started: false,
            last_exit_status: None,
            log: vec![],
        }
    }

    pub fn get_current_runner(&self) -> Option<&RefCell<Runner>> {
        self.runners.get(self.current_index)
    }

    pub fn start(&mut self) {
        if self.has_started {
            return;
        }
        self.has_started = true;
        match self.get_current_runner() {
            None => { return; },
            Some(runner) => {
                runner.borrow_mut().start();
            }
        }
    }

    pub fn update(&mut self) {
        if !self.has_started {
            return;
        }

        match self.get_current_runner() {
            None => {},
            Some(c_runner) => {
                let (status, log) = {
                    let mut runner = c_runner.borrow_mut();
                    (runner.status(), runner.consume_rows())
                };
                self.log.extend(log);
                match status {
                    RunnerStatus::Running => {},
                    RunnerStatus::Stopped(ecode) => {
                        match ecode {
                            None => {} // should never get here
                            Some(0) => {
                                self.last_exit_status = Some(0);
                                self.current_index += 1;
                                match self.get_current_runner() {
                                    None => {},
                                    Some(c_runner) => {
                                        c_runner.borrow_mut().start();
                                    }
                                }
                            }
                            Some(nonzero) => {
                                self.last_exit_status = Some(nonzero);
                                // interrupting pipeline by going past last runner
                                self.current_index = self.runners.len();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_log(&mut self) -> Vec<String> {
        let log = self.log.to_owned();
        self.log = vec![];
        log
    }

    pub fn status(&self) -> RunnerStatus {
        match self.get_current_runner() {
            None => RunnerStatus::Stopped(self.last_exit_status),
            Some(_) => RunnerStatus::Running
        }
    }
}
