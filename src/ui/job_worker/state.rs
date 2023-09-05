use nix::unistd::Pid;

#[derive(Debug, Clone, Default)]
pub struct JobWorkerState {
    pub current_pid: Option<Pid>,
    pub exit_status: Option<i32>,
    pub stop_requested: bool,
    pub started: bool,
    pub exited: bool,
}
