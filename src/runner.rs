use std::{
    collections::HashMap,
    fs::{create_dir_all, OpenOptions},
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
    process::{Child, Command, Stdio},
};

use crate::{profile::Profile, file_utils::get_writer};

pub struct Runner {
    environment: HashMap<String, String>,
    command: String,
    args: Vec<String>,
    stdout: Vec<String>,
    stderr: Vec<String>,
    process: Option<Child>,
}

#[derive(PartialEq, Eq, Debug)]
pub enum RunnerStatus {
    Running,
    Stopped,
}

impl Runner {
    pub fn new(environment: HashMap<String, String>, command: String, args: Vec<String>) -> Self {
        Self {
            environment,
            command,
            args,
            stdout: Vec::new(),
            stderr: Vec::new(),
            process: None,
        }
    }

    pub fn monado_runner_from_profile(profile: Profile) -> Self {
        Self {
            environment: profile.environment,
            command: profile.monado_path,
            args: vec![],
            stdout: Vec::new(),
            stderr: Vec::new(),
            process: None,
        }
    }

    pub fn start(&mut self) {
        match self.process {
            Some(_) => panic!("Cannot reuse existing Runner"),
            None => {
                self.process = Some(
                    Command::new(&self.command)
                        .args(&self.args)
                        .envs(self.environment.clone())
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to execute runner"),
                );
            }
        };
    }

    pub fn terminate(&mut self) {
        match self.process.as_mut() {
            None => {
                println!("Runner already stopped")
            }
            Some(proc) => {
                proc.kill().expect("Failed to kill process");
                proc.wait().expect("Failed to wait for process");
            }
        }
    }

    pub fn status(&mut self) -> RunnerStatus {
        match self.process.as_mut() {
            None => RunnerStatus::Stopped,
            Some(proc) => match proc.try_wait() {
                Ok(_) => RunnerStatus::Stopped,
                Err(_) => RunnerStatus::Running,
            },
        }
    }

    pub fn flush_out(&mut self) {
        match self.process.as_mut() {
            None => (),
            Some(proc) => {
                let out = proc.stdout.take().expect("Unable to read stout");
                let mut reader = BufReader::new(out);
                let mut bytes_read = 1;
                let mut buf = "".to_string();
                while bytes_read != 0 {
                    bytes_read = match reader.read_line(&mut buf) {
                        Ok(bytes) => bytes,
                        Err(_) => 0,
                    };
                    self.stdout.push(buf.clone());
                    buf.clear();
                }

                let err = proc.stderr.take().expect("Unable to read stderr");
                let mut err_reader = BufReader::new(err);
                bytes_read = 1;
                while bytes_read != 0 {
                    bytes_read = match err_reader.read_line(&mut buf) {
                        Ok(bytes) => bytes,
                        Err(_) => 0,
                    };
                    self.stderr.push(buf.clone());
                    buf.clear();
                }
            }
        }
    }

    pub fn get_stdout(&mut self) -> String {
        self.flush_out();
        self.stdout.concat()
    }

    fn save_log(path_s: String, log: &Vec<String>) -> Result<(), std::io::Error> {
        let mut writer = get_writer(&path_s);
        let log_s = log.concat();
        writer.write_all(log_s.as_ref())
    }

    pub fn save_stdout(&mut self, base_path: String) -> Result<(), std::io::Error> {
        Runner::save_log(base_path + ".stdout", &self.stdout)
    }

    pub fn save_stderr(&mut self, base_path: String) -> Result<(), std::io::Error> {
        Runner::save_log(base_path + ".stderr", &self.stderr)
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
            env,
            "bash".into(),
            vec!["-c".into(), "echo \"REX2TEST: $REX2TEST\"".into()],
        );
        runner.start();
        while runner.status() == RunnerStatus::Running {
            sleep(time::Duration::from_millis(10));
        }
        runner.flush_out();
        assert_eq!(runner.status(), RunnerStatus::Stopped);
        assert_eq!(runner.stdout.len(), 2);
        assert_eq!(
            runner.stdout.get(0).unwrap(),
            "REX2TEST: Lorem ipsum dolor\n"
        );
    }

    #[test]
    fn can_save_log() {
        let mut runner = Runner::new(
            HashMap::new(),
            "bash".into(),
            vec!["-c".into(), "echo \"Lorem ipsum dolor sit amet\"".into()],
        );
        runner.start();
        while runner.status() == RunnerStatus::Running {
            sleep(time::Duration::from_millis(10));
        }
        runner.flush_out();

        runner.save_stdout("./target/testout/testlog".into());
        runner.save_stderr("./target/testout/testlog".into());
    }

    #[test]
    fn can_create_from_profile() {
        Runner::monado_runner_from_profile(
            load_profile(&"./test/files/profile.json".to_string()).unwrap(),
        );
    }
}
