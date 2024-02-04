use self::{
    internal_worker::{InternalJobWorker, JobWorkerInit, JobWorkerMsg, JobWorkerOut},
    job::{FuncWorkerOut, WorkerJob},
    state::JobWorkerState,
};
use crate::profile::Profile;
use nix::sys::signal::{
    kill,
    Signal::{SIGKILL, SIGTERM},
};
use relm4::{prelude::*, Sender, WorkerController};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

pub mod internal_worker;
pub mod job;
pub mod state;

pub struct JobWorker {
    pub worker: WorkerController<InternalJobWorker>,
    pub state: Arc<Mutex<JobWorkerState>>,
}

impl JobWorker {
    pub fn new<X: 'static, F: (Fn(JobWorkerOut) -> X) + 'static>(
        jobs: VecDeque<WorkerJob>,
        sender: &Sender<X>,
        transform: F,
    ) -> Self {
        let state = Arc::new(Mutex::new(JobWorkerState::default()));
        let init = JobWorkerInit {
            jobs,
            state: state.clone(),
        };
        Self {
            worker: InternalJobWorker::builder()
                .detach_worker(init)
                .forward(sender, transform),
            state,
        }
    }

    pub fn new_with_timer<X: 'static, F: (Fn(JobWorkerOut) -> X) + 'static>(
        duration: Duration,
        job: WorkerJob,
        sender: &Sender<X>,
        transform: F,
    ) -> Self {
        let timer_job = WorkerJob::new_func(Box::new(move || {
            thread::sleep(duration);
            FuncWorkerOut::default()
        }));
        let mut jobs = VecDeque::new();
        let state = Arc::new(Mutex::new(JobWorkerState::default()));
        jobs.push_back(timer_job);
        jobs.push_back(job);
        let init = JobWorkerInit {
            jobs,
            state: state.clone(),
        };
        Self {
            worker: InternalJobWorker::builder()
                .detach_worker(init)
                .forward(sender, transform),
            state,
        }
    }

    pub fn xrservice_worker_wrap_from_profile<X: 'static, F: (Fn(JobWorkerOut) -> X) + 'static>(
        prof: &Profile,
        sender: &Sender<X>,
        transform: F,
        debug: bool,
    ) -> Self {
        let state = Arc::new(Mutex::new(JobWorkerState::default()));
        Self {
            worker: InternalJobWorker::xrservice_worker_from_profile(prof, state.clone(), debug)
                .forward(sender, transform),
            state,
        }
    }

    pub fn start(&self) {
        self.worker.emit(JobWorkerMsg::Start);
    }

    pub fn stop(&self) {
        if self.state.lock().unwrap().started && !self.state.lock().unwrap().exited {
            self.state.lock().unwrap().stop_requested = true;
            if let Some(pid) = self.state.lock().unwrap().current_pid {
                kill(pid, SIGTERM).expect("Could not send sigterm to process");
                let state = self.state.clone();
                thread::spawn(move || {
                    sleep(Duration::from_secs(2));
                    if let Ok(s) = state.lock() {
                        if !s.exited {
                            // process is still alive
                            kill(pid, SIGKILL).expect("Failed to kill process");
                        }
                    }
                });
            }
        }
    }

    pub fn exit_code(&self) -> Option<i32> {
        if let Ok(state) = self.state.lock() {
            return state.exit_status;
        }

        None
    }
}
