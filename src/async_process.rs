use std::ffi::OsStr;
use std::process::Stdio;
use std::{collections::HashMap, os::unix::process::ExitStatusExt};
use tokio::process::Command;

pub struct AsyncProcessOut {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

pub async fn async_process<S: AsRef<OsStr>, T: AsRef<OsStr>>(
    cmd: S,
    args: Option<&[T]>,
    env: Option<HashMap<String, String>>,
) -> anyhow::Result<AsyncProcessOut> {
    let cmd = Command::new(cmd)
        .args(args.unwrap_or_default())
        .envs(env.unwrap_or_default())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;
    let out = cmd.wait_with_output().await?;
    Ok(AsyncProcessOut {
        exit_code: out
            .status
            .code()
            .unwrap_or_else(|| out.status.signal().map(|sig| sig + 128).unwrap_or(-1337)),
        stdout: String::from_utf8(out.stdout)?,
        stderr: String::from_utf8(out.stderr)?,
    })
}
