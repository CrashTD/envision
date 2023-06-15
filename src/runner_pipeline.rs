use std::{cell::RefCell, borrow::BorrowMut};

use crate::runner::{Runner, RunnerStatus};

pub struct RunnerPipeline {
    runners: Vec<RefCell<Runner>>,
    current_index: usize,
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
                    RunnerStatus::Stopped => {
                        self.current_index += 1;
                        match self.get_current_runner() {
                            None => {},
                            Some(c_runner) => {
                                c_runner.borrow_mut().start();
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn get_log(&self) -> String {
        self.log.concat()
    }

    pub fn is_running(&self) -> bool {
        self.get_current_runner().is_some()
    }
}
