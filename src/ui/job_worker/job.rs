use std::collections::HashMap;

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
    pub fn new_cmd(
        env: Option<HashMap<String, String>>,
        cmd: String,
        args: Option<Vec<String>>,
    ) -> Self {
        Self::Cmd(CmdWorkerData {
            environment: env.unwrap_or_default(),
            command: cmd,
            args: args.unwrap_or_default(),
        })
    }

    pub fn new_func(func: Box<dyn FnOnce() -> FuncWorkerOut + Send + Sync + 'static>) -> Self {
        Self::Func(FuncWorkerData { func })
    }
}
