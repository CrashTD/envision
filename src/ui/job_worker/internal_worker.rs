use super::{
    job::{CmdWorkerData, FuncWorkerOut, WorkerJob},
    state::JobWorkerState,
};
use crate::{
    config::Config,
    profile::{LighthouseDriver, Profile},
    ui::SENDER_IO_ERR_MSG,
};
use nix::unistd::Pid;
use relm4::{Worker, prelude::*};
use std::{
    collections::VecDeque,
    io::{BufRead, BufReader},
    mem,
    os::unix::process::ExitStatusExt,
    process::{Command, Stdio},
    sync::{Arc, Mutex},
    thread,
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

#[derive(Debug)]
pub enum JobWorkerOut {
    Log(Vec<String>),
    Exit(i32),
}

#[derive(Debug)]
pub enum JobWorkerMsg {
    Start,
}

pub struct InternalJobWorker {
    jobs: VecDeque<WorkerJob>,
    state: Arc<Mutex<JobWorkerState>>,
}

pub struct JobWorkerInit {
    pub jobs: VecDeque<WorkerJob>,
    pub state: Arc<Mutex<JobWorkerState>>,
}

impl Worker for InternalJobWorker {
    type Init = JobWorkerInit;
    type Input = JobWorkerMsg;
    type Output = JobWorkerOut;

    fn init(init: Self::Init, _sender: ComponentSender<Self>) -> Self {
        Self {
            jobs: init.jobs,
            state: init.state,
        }
    }

    fn update(&mut self, msg: JobWorkerMsg, sender: ComponentSender<Self>) {
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
                            let data = data.clone();
                            if let Ok(mut cmd) = Command::new(data.command)
                                .args(data.args)
                                .envs(data.environment)
                                .stdin(Stdio::piped())
                                .stderr(Stdio::piped())
                                .stdout(Stdio::piped())
                                .spawn()
                            {
                                self.state.lock().unwrap().current_pid = Some(Pid::from_raw(
                                    cmd.id().try_into().expect("Could not convert pid to u32"),
                                ));
                                let stdout = cmd.stdout.take().unwrap();
                                let stderr = cmd.stderr.take().unwrap();
                                let _stdin = cmd.stdin.take().unwrap();
                                let stdout_sender = sender.clone();
                                let stderr_sender = sender.clone();
                                let stdout_logger = logger_thread!(stdout, stdout_sender);
                                let stderr_logger = logger_thread!(stderr, stderr_sender);
                                let status = cmd.wait().expect("Failed to wait for process");
                                if !status.success() {
                                    self.state.lock().unwrap().exit_status =
                                        Some(status.code().unwrap_or_else(|| {
                                            status.signal().map(|sig| sig + 128).unwrap_or(-1337)
                                        }));
                                    break;
                                }
                                stdout_logger
                                    .join()
                                    .expect("Failed to join stdout reader thread");
                                stderr_logger
                                    .join()
                                    .expect("Failed to join stderr reader thread");
                            } else {
                                let msg = "Failed to run command".to_string();
                                sender
                                    .output(Self::Output::Log(vec![msg]))
                                    .expect(SENDER_IO_ERR_MSG);
                                self.state.lock().unwrap().exit_status = Some(1);
                                break;
                            }
                        }
                        WorkerJob::Func(data) => {
                            let func =
                                mem::replace(&mut data.func, Box::new(FuncWorkerOut::default));
                            let out = func();
                            sender
                                .output(Self::Output::Log(out.out.clone()))
                                .expect(SENDER_IO_ERR_MSG);
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

pub const LAUNCH_OPTS_CMD_PLACEHOLDER: &str = "%command%";

impl InternalJobWorker {
    pub fn xrservice_worker_from_profile(
        prof: &Profile,
        config: &Config,
        state: Arc<Mutex<JobWorkerState>>,
        debug: bool,
    ) -> Option<relm4::WorkerHandle<InternalJobWorker>> {
        let mut env = prof.environment.clone();
        if !env.contains_key("LH_DRIVER") {
            match prof.lighthouse_driver {
                // don't set LH_DRIVER for steamvr driver, set this instead
                LighthouseDriver::SteamVR => {
                    env.insert("STEAMVR_LH_ENABLE".into(), "true".into());
                }
                d => {
                    env.insert("LH_DRIVER".into(), d.to_string().to_lowercase());
                }
            }
        }
        let mut launch_opts = prof.xrservice_launch_options.trim().to_string();
        let debug_launch_opts = if debug {
            if launch_opts.contains(LAUNCH_OPTS_CMD_PLACEHOLDER) {
                format!("{} {}", "gdbserver localhost:9000", launch_opts)
            } else {
                format!(
                    "{} {} {}",
                    "gdbserver localhost:9000", LAUNCH_OPTS_CMD_PLACEHOLDER, launch_opts
                )
            }
        } else {
            String::default()
        };
        launch_opts = if debug {
            debug_launch_opts
        } else {
            launch_opts
        };
        let (command, args) = match launch_opts.is_empty() {
            false => (
                "sh".into(),
                vec![
                    "-c".into(),
                    match launch_opts.contains(LAUNCH_OPTS_CMD_PLACEHOLDER) {
                        true => launch_opts.replacen(
                            LAUNCH_OPTS_CMD_PLACEHOLDER,
                            prof.xrservice_binary(Some(config))?
                                .to_string_lossy()
                                .as_ref(),
                            1,
                        ),
                        false => format!(
                            "{} {}",
                            prof.xrservice_binary(Some(config))?.to_string_lossy(),
                            launch_opts
                        ),
                    },
                ],
            ),
            true => (
                prof.xrservice_binary(Some(config))?
                    .to_string_lossy()
                    .to_string(),
                vec![],
            ),
        };
        let data = CmdWorkerData {
            environment: env,
            command,
            args,
        };
        let mut jobs = VecDeque::new();
        jobs.push_back(WorkerJob::Cmd(data));
        Some(Self::builder().detach_worker(JobWorkerInit { jobs, state }))
    }
}
