use crate::{file_utils::get_writer, profile::Profile};
use expect_dialog::ExpectDialog;
use nix::{
    sys::signal::{kill, Signal::SIGTERM},
    unistd::Pid,
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
    vec,
};

pub struct Runner {
    pub environment: HashMap<String, String>,
    pub command: String,
    pub args: Vec<String>,
    pub output: Vec<String>,
    sender: Arc<Mutex<SyncSender<String>>>,
    receiver: Receiver<String>,
    threads: Vec<JoinHandle<()>>,
    process: Option<Child>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum RunnerStatus {
    Running,
    Stopped,
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
                        match $sender
                            .clone()
                            .lock()
                            .expect_dialog("Could not lock sender")
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

impl Runner {
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

    pub fn monado_runner_from_profile(profile: Profile) -> Self {
        Self::new(
            Some(profile.environment),
            format!("{pfx}/bin/monado-service", pfx = profile.prefix),
            vec![],
        )
    }

    pub fn start(&mut self) {
        self.threads = Vec::new();
        self.process = Some(
            Command::new(self.command.clone())
                .args(self.args.clone())
                .envs(self.environment.clone())
                .stderr(Stdio::piped())
                .stdout(Stdio::piped())
                .spawn()
                .expect_dialog("Failed to execute runner"),
        );
        let stdout = self.process.as_mut().unwrap().stdout.take().unwrap();
        let stderr = self.process.as_mut().unwrap().stderr.take().unwrap();
        let stdout_sender = self.sender.clone();
        let stderr_sender = self.sender.clone();
        self.threads.push(logger_thread!(stdout, stdout_sender));
        self.threads.push(logger_thread!(stderr, stderr_sender));
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
        let child_pid = Pid::from_raw(
            proc.id()
                .try_into()
                .expect_dialog("Could not convert pid to u32"),
        );
        kill(child_pid, SIGTERM).expect_dialog("Could not send sigterm to process");
        loop {
            match self.threads.pop() {
                None => break,
                Some(thread) => thread.join().expect_dialog("Failed to join reader thread"),
            }
        }
        proc.wait().expect_dialog("Failed to wait for process");
    }

    pub fn status(&mut self) -> RunnerStatus {
        match &mut self.process {
            None => RunnerStatus::Stopped,
            Some(proc) => match proc.try_wait() {
                Err(_) => RunnerStatus::Running,
                Ok(Some(_)) => RunnerStatus::Stopped,
                Ok(None) => RunnerStatus::Running,
            },
        }
    }

    fn receive_output(&mut self) {
        loop {
            match self.receiver.try_recv() {
                Ok(data) => self.output.push(data),
                Err(_) => break,
            };
        }
    }

    pub fn consume_output(&mut self) -> String {
        self.receive_output();
        let res = self.output.concat();
        self.output.clear();
        res
    }

    pub fn consume_rows(&mut self) -> Vec<String> {
        self.receive_output();
        let res = self.output.clone();
        self.output.clear();
        res
    }

    fn save_log(path_s: String, log: &Vec<String>) -> Result<(), std::io::Error> {
        let mut writer = get_writer(&path_s);
        let log_s = log.concat();
        writer.write_all(log_s.as_ref())
    }

    pub fn save_output(&mut self, path: String) -> Result<(), std::io::Error> {
        Runner::save_log(path, &self.output)
    }
}

#[cfg(test)]
mod tests {
    use crate::profile::load_profile;

    use super::{Runner, RunnerStatus};
    use core::time;
    use std::{collections::HashMap, thread::sleep};

    #[test]
    fn can_run_command_and_read_env() {
        let mut env = HashMap::new();
        env.insert("REX2TEST".to_string(), "Lorem ipsum dolor".to_string());
        let mut runner = Runner::new(
            Some(env),
            "bash".into(),
            vec!["-c".into(), "echo \"REX2TEST: $REX2TEST\"".into()],
        );
        runner.start();
        sleep(time::Duration::from_millis(10));
        runner.terminate();
        assert_eq!(runner.status(), RunnerStatus::Stopped);
        let out = runner.consume_output();
        assert_eq!(out, "REX2TEST: Lorem ipsum dolor\n");
    }

    #[test]
    fn can_save_log() {
        let mut runner = Runner::new(
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
        Runner::monado_runner_from_profile(load_profile(&"./test/files/profile.json".to_string()));
    }
}
