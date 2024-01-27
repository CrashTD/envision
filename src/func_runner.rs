use crate::runner::{Runner, RunnerStatus};
use std::{
    mem,
    thread::{self, JoinHandle},
};

#[derive(Debug, Clone, Default)]
pub struct FuncRunnerOut {
    pub success: bool,
    pub out: Vec<String>,
}

pub struct FuncRunner {
    output: Vec<String>,
    res: Option<bool>,
    thread: Option<JoinHandle<FuncRunnerOut>>,
    func: Box<dyn FnOnce() -> FuncRunnerOut + Send + Sync + 'static>,
}

impl FuncRunner {
    pub fn new(func: Box<dyn FnOnce() -> FuncRunnerOut + Send + Sync + 'static>) -> Self {
        FuncRunner {
            output: Vec::new(),
            thread: None,
            res: None,
            func,
        }
    }
}

impl Runner for FuncRunner {
    fn start(&mut self) {
        if self.res.is_some() {
            panic!("Cannot start a FuncRunner twice!");
        }
        let f = mem::replace(&mut self.func, Box::new(move || FuncRunnerOut::default()));
        self.thread = Some(thread::spawn(move || f()));
    }

    fn status(&mut self) -> RunnerStatus {
        if let Some(res) = self.res {
            if res {
                return RunnerStatus::Stopped(Some(0));
            }
            return RunnerStatus::Stopped(Some(1));
        }
        if let Some(t) = self.thread.take() {
            if t.is_finished() {
                let out = t.join().expect("Failed to join thread");
                self.res = Some(out.success);
                self.output = out.out;
                return self.status();
            }
            self.thread = Some(t);
            return RunnerStatus::Running;
        }
        RunnerStatus::Stopped(None)
    }

    fn consume_output(&mut self) -> String {
        self.consume_rows().concat()
    }

    fn consume_rows(&mut self) -> Vec<String> {
        let res = self.output.clone();
        self.output.clear();
        res
    }
}
