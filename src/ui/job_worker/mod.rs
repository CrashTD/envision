use self::{
    internal_worker::{InternalJobWorker, JobWorkerInit, JobWorkerMsg, JobWorkerOut},
    job::{FuncWorkerOut, WorkerJob},
    state::JobWorkerState,
};
use crate::{config::Config, profile::Profile};
use nix::sys::signal::{
    Signal::{SIGKILL, SIGTERM},
    kill,
};
use relm4::{Sender, WorkerController, prelude::*};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};
use tracing::{error, warn};

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
        config: &Config,
        sender: &Sender<X>,
        transform: F,
        debug: bool,
    ) -> Option<Self> {
        let state = Arc::new(Mutex::new(JobWorkerState::default()));
        Some(Self {
            worker: InternalJobWorker::xrservice_worker_from_profile(
                prof,
                config,
                state.clone(),
                debug,
            )?
            .forward(sender, transform),
            state,
        })
    }

    pub fn start(&self) {
        self.worker.emit(JobWorkerMsg::Start);
    }

    pub fn stop(&self) {
        let should_stop_manually = {
            let state = self.state.lock().unwrap();
            state.started && !state.exited
        };
        if should_stop_manually {
            self.state.lock().unwrap().stop_requested = true;
            if let Some(pid) = self.state.lock().unwrap().current_pid {
                if let Err(e) = kill(pid, SIGTERM) {
                    error!("Failed to send SIGTERM: {e}");
                }
                let state = self.state.clone();
                thread::spawn(move || {
                    sleep(Duration::from_secs(2));
                    if let Ok(s) = state.lock()
                        && !s.exited
                    {
                        // process is still alive
                        warn!(
                            "process is still alive 2 seconds after SIGTERM, proceeding to send SIGKILL..."
                        );
                        if let Err(e) = kill(pid, SIGKILL) {
                            error!("failed to send SIGKILL: {e}");
                        };
                    }
                });
            }
        }
    }

    /// drop the reference but continue the execution
    pub fn detach(mut self) {
        self.worker.detach_runtime();
    }

    pub fn exit_code(&self) -> Option<i32> {
        if let Ok(state) = self.state.lock() {
            return state.exit_status;
        }

        None
    }

    pub fn is_alive(&self) -> bool {
        let state = self.state.lock().unwrap();
        !state.stop_requested && state.started && !state.exited && state.exit_status.is_none()
    }
}
