use crate::{profile::Profile, withclones};
use nix::{
    sys::signal::{
        kill,
        Signal::{SIGKILL, SIGTERM},
    },
    unistd::Pid,
};
use relm4::{prelude::*, Sender, Worker, WorkerController};
use std::{
    collections::{HashMap, VecDeque},
    io::{BufRead, BufReader},
    mem,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread::{self, sleep},
    time::Duration,
};

macro_rules! logger_thread {
    ($buf_fd: expr, $sender: expr) => {
        thread::spawn(move || {
            let mut reader = BufReader::new($buf_fd);
            loop {
                let mut buf = String::new();
                match reader.read_line(&mut buf) {
                    Err(_) => return,
                    Ok(bytes_read) => {
                        if bytes_read == 0 {
                            return;
                        }
                        if buf.is_empty() {
                            continue;
                        }
                        print!("{}", buf);
                        match $sender.output(Self::Output::Log(vec![buf])) {
                            Ok(_) => {}
                            Err(_) => return,
                        };
                    }
                };
            }
        })
    };
}

#[derive(Debug, Clone)]
pub struct CmdWorkerData {
    pub environment: HashMap<String, String>,
    pub command: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct FuncWorkerOut {
    pub success: bool,
    pub out: Vec<String>,
}

pub struct FuncWorkerData {
    pub func: Box<dyn FnOnce() -> FuncWorkerOut + Send + Sync + 'static>,
}

pub enum WorkerJob {
    Cmd(CmdWorkerData),
    Func(FuncWorkerData),
}

impl WorkerJob {
    pub fn new_cmd(env: Option<HashMap<String, String>>, cmd: String, args: Option<Vec<String>>) -> Self {
        Self::Cmd(CmdWorkerData { environment: env.unwrap_or_default(), command: cmd, args: args.unwrap_or_default() })
    }

    pub fn new_func(func: Box<dyn FnOnce() -> FuncWorkerOut + Send + Sync + 'static>) -> Self {
        Self::Func(FuncWorkerData { func })
    }
}

#[derive(Debug)]
pub enum RunnerWorkerOut {
    Log(Vec<String>),
    Exit(i32),
}

#[derive(Debug)]
pub enum RunnerWorkerMsg {
    Start,
}

#[derive(Debug, Clone, Default)]
pub struct RunnerWorkerState {
    pub current_pid: Option<Pid>,
    pub exit_status: Option<i32>,
    pub stop_requested: bool,
    pub started: bool,
    pub exited: bool,
}

pub struct RunnerWorker {
    jobs: VecDeque<WorkerJob>,
    state: Arc<Mutex<RunnerWorkerState>>,
}

pub struct RunnerWorkerInit {
    pub jobs: VecDeque<WorkerJob>,
    pub state: Arc<Mutex<RunnerWorkerState>>,
}

impl Worker for RunnerWorker {
    type Init = RunnerWorkerInit;
    type Input = RunnerWorkerMsg;
    type Output = RunnerWorkerOut;

    fn init(init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            jobs: init.jobs,
            state: init.state,
        }
    }

    fn update(&mut self, msg: RunnerWorkerMsg, sender: ComponentSender<Self>) {
        // Send the result of the calculation back
        match msg {
            Self::Input::Start => {
                if self.state.lock().unwrap().started {
                    panic!("Cannot start a RunnerWorker twice!")
                }
                self.state.lock().unwrap().started = true;
                while !(self.jobs.is_empty() || self.state.lock().unwrap().stop_requested) {
                    self.state.lock().unwrap().current_pid = None;
                    let mut job = self.jobs.pop_front().unwrap();
                    match &mut job {
                        WorkerJob::Cmd(data) => {
                            withclones![data];
                            if let Ok(mut cmd) = Command::new(data.command)
                                .args(data.args)
                                .envs(data.environment)
                                .stderr(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                self.state.lock().unwrap().current_pid = Some(Pid::from_raw(
                                    cmd.id().try_into().expect("Could not convert pid to u32"),
                                ));
                                let stdout = cmd.stdout.take().unwrap();
                                let stderr = cmd.stderr.take().unwrap();
                                let stdout_sender = sender.clone();
                                let stderr_sender = sender.clone();
                                let stdout_logger = logger_thread!(stdout, stdout_sender);
                                let stderr_logger = logger_thread!(stderr, stderr_sender);
                                cmd.wait().expect("Failed to wait for process");
                                stdout_logger.join().expect("Failed to join reader thread");
                                stderr_logger.join().expect("Failed to join reader thread");
                            } else {
                                let msg = "Failed to run command".to_string();
                                sender.output(Self::Output::Log(vec![msg]));
                                self.state.lock().unwrap().exit_status = Some(1);
                                break;
                            }
                        }
                        WorkerJob::Func(data) => {
                            let func = mem::replace(
                                &mut data.func,
                                Box::new(move || FuncWorkerOut::default()),
                            );
                            let out = func();
                            sender.output(Self::Output::Log(out.out.clone()));
                            if !out.success {
                                self.state.lock().unwrap().exit_status = Some(1);
                                break;
                            }
                        }
                    }
                }
                sender
                    .output(Self::Output::Exit(
                        self.state.lock().unwrap().exit_status.unwrap_or(0),
                    ))
                    .expect("Failed to send output exit");
                self.state.lock().unwrap().exited = true;
            }
        }
    }
}

impl RunnerWorker {
    pub fn xrservice_worker_from_profile(
        prof: &Profile,
        state: Arc<Mutex<RunnerWorkerState>>,
    ) -> relm4::WorkerHandle<RunnerWorker> {
        let mut env = prof.environment.clone();
        if !env.contains_key("LH_DRIVER") {
            env.insert(
                "LH_DRIVER".into(),
                prof.lighthouse_driver.to_string().to_lowercase(),
            );
        }
        let data = CmdWorkerData {
            environment: env,
            command: prof.xrservice_binary(),
            args: vec![],
        };
        let mut jobs = VecDeque::new();
        jobs.push_back(WorkerJob::Cmd(data));
        Self::builder().detach_worker(RunnerWorkerInit { jobs, state })
    }
}

pub struct RunnerWorkerWrap {
    pub worker: WorkerController<RunnerWorker>,
    pub state: Arc<Mutex<RunnerWorkerState>>,
}

impl RunnerWorkerWrap {
    pub fn new<X: 'static, F: (Fn(RunnerWorkerOut) -> X) + 'static>(
        jobs: VecDeque<WorkerJob>,
        sender: &Sender<X>,
        transform: F,
    ) -> Self {
        let state = Arc::new(Mutex::new(RunnerWorkerState::default()));
        let init = RunnerWorkerInit {
            jobs,
            state: state.clone(),
        };
        Self {
            worker: RunnerWorker::builder()
                .detach_worker(init)
                .forward(sender, transform),
            state,
        }
    }

    pub fn xrservice_worker_wrap_from_profile<
        X: 'static,
        F: (Fn(RunnerWorkerOut) -> X) + 'static,
    >(
        prof: &Profile,
        sender: &Sender<X>,
        transform: F,
    ) -> Self {
        let state = Arc::new(Mutex::new(RunnerWorkerState::default()));
        Self {
            worker: RunnerWorker::xrservice_worker_from_profile(prof, state.clone())
                .forward(sender, transform),
            state,
        }
    }

    pub fn start(&self) {
        self.worker.emit(RunnerWorkerMsg::Start);
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
                        if s.exited {
                            // process is still alive
                            kill(pid, SIGKILL).expect("Failed to kill process");
                        }
                    }
                });
            }
        }
    }
}
