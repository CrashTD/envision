use nix::{
    sys::signal::{kill, Signal::SIGTERM},
    unistd::Pid,
};

use crate::{
    file_utils::get_writer,
    profile::{Profile, XRServiceType},
    runner::{Runner, RunnerStatus},
};
use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Write},
    process::{Child, Command, Stdio},
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
};

pub struct CmdRunner {
    pub environment: HashMap<String, String>,
    pub command: String,
    pub args: Vec<String>,
    pub output: Vec<String>,
    sender: Arc<Mutex<SyncSender<String>>>,
    receiver: Receiver<String>,
    threads: Vec<JoinHandle<()>>,
    process: Option<Child>,
}

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
                        match $sender
                            .clone()
                            .lock()
                            .expect("Could not lock sender")
                            .send(buf)
                        {
                            Ok(_) => {}
                            Err(_) => return,
                        };
                    }
                };
            }
        })
    };
}

impl CmdRunner {
    pub fn new(
        environment: Option<HashMap<String, String>>,
        command: String,
        args: Vec<String>,
    ) -> Self {
        let (sender, receiver) = sync_channel(64000);
        Self {
            environment: match environment {
                None => HashMap::new(),
                Some(e) => e.clone(),
            },
            command,
            args,
            output: Vec::new(),
            process: None,
            sender: Arc::new(Mutex::new(sender)),
            threads: Vec::new(),
            receiver,
        }
    }

    pub fn xrservice_runner_from_profile(profile: &Profile) -> Self {
        let mut env = profile.environment.clone();
        if !env.contains_key("LH_DRIVER") {
            env.insert(
                "LH_DRIVER".into(),
                profile.lighthouse_driver.to_string().to_lowercase(),
            );
        }
        Self::new(
            Some(env),
            match profile.xrservice_type {
                XRServiceType::Monado => format!("{pfx}/bin/monado-service", pfx = profile.prefix),
                XRServiceType::Wivrn => format!("{pfx}/bin/wivrn-server", pfx = profile.prefix),
            },
            vec![],
        )
    }

    pub fn try_start(&mut self) -> Result<(), std::io::Error> {
        self.threads = Vec::new();
        let cmd = Command::new(self.command.clone())
            .args(self.args.clone())
            .envs(self.environment.clone())
            .stderr(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        self.process = Some(cmd);
        let stdout = self.process.as_mut().unwrap().stdout.take().unwrap();
        let stderr = self.process.as_mut().unwrap().stderr.take().unwrap();
        let stdout_sender = self.sender.clone();
        let stderr_sender = self.sender.clone();
        self.threads.push(logger_thread!(stdout, stdout_sender));
        self.threads.push(logger_thread!(stderr, stderr_sender));
        Ok(())
    }

    fn join_threads(&mut self) {
        loop {
            match self.threads.pop() {
                None => break,
                Some(thread) => thread.join().expect("Failed to join reader thread"),
            }
        }
    }

    pub fn terminate(&mut self) {
        if self.status() != RunnerStatus::Running {
            return;
        }
        let process = self.process.take();
        if process.is_none() {
            return;
        }
        let mut proc = process.unwrap();
        let child_pid = Pid::from_raw(proc.id().try_into().expect("Could not convert pid to u32"));
        kill(child_pid, SIGTERM).expect("Could not send sigterm to process");
        self.join_threads();
        proc.wait().expect("Failed to wait for process");
    }

    pub fn join(&mut self) {
        let process = self.process.take();
        if process.is_none() {
            return;
        }
        let mut proc = process.unwrap();
        proc.wait().expect("Failed to wait for process");
        self.join_threads();
    }

    fn receive_output(&mut self) {
        while let Ok(data) = self.receiver.try_recv() {
            self.output.push(data);
        }
    }

    fn save_log(path_s: String, log: &[String]) -> Result<(), std::io::Error> {
        let mut writer = get_writer(&path_s);
        let log_s = log.concat();
        writer.write_all(log_s.as_ref())
    }

    pub fn save_output(&mut self, path: String) -> Result<(), std::io::Error> {
        CmdRunner::save_log(path, &self.output)
    }
}

impl Runner for CmdRunner {
    fn consume_output(&mut self) -> String {
        self.receive_output();
        let res = self.output.concat();
        self.output.clear();
        res
    }

    fn consume_rows(&mut self) -> Vec<String> {
        self.receive_output();
        let res = self.output.clone();
        self.output.clear();
        res
    }

    fn status(&mut self) -> RunnerStatus {
        match &mut self.process {
            None => RunnerStatus::Stopped(None),
            Some(proc) => match proc.try_wait() {
                Err(_) => RunnerStatus::Running,
                Ok(Some(code)) => RunnerStatus::Stopped(code.code()),
                Ok(None) => RunnerStatus::Running,
            },
        }
    }

    fn start(&mut self) {
        self.try_start().expect("Failed to execute runner");
    }
}

#[cfg(test)]
mod tests {
    use super::{CmdRunner, RunnerStatus};
    use crate::profile::Profile;
    use crate::runner::Runner;
    use core::time;
    use std::{collections::HashMap, thread::sleep};

    #[test]
    fn can_run_command_and_read_env() {
        let mut env = HashMap::new();
        env.insert("REX2TEST".to_string(), "Lorem ipsum dolor".to_string());
        let mut runner = CmdRunner::new(
            Some(env),
            "bash".into(),
            vec!["-c".into(), "echo \"REX2TEST: $REX2TEST\"".into()],
        );
        runner.start();
        sleep(time::Duration::from_millis(1000)); // TODO: ugly, fix
        runner.terminate();
        assert_eq!(runner.status(), RunnerStatus::Stopped(Some(0)));
        let out = runner.consume_output();
        assert_eq!(out, "REX2TEST: Lorem ipsum dolor\n");
    }

    #[test]
    fn can_save_log() {
        let mut runner = CmdRunner::new(
            None,
            "bash".into(),
            vec!["-c".into(), "echo \"Lorem ipsum dolor sit amet\"".into()],
        );
        runner.start();
        while runner.status() == RunnerStatus::Running {
            sleep(time::Duration::from_millis(10));
        }

        runner
            .save_output("./target/testout/testlog".into())
            .expect("Failed to save output file");
    }

    #[test]
    fn can_create_from_profile() {
        CmdRunner::xrservice_runner_from_profile(&Profile::load_profile(
            &"./test/files/profile.json".to_string(),
        ));
    }
}
