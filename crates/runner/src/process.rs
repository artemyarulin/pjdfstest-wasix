use std::{
    io::{self, Read},
    path::{Path, PathBuf},
    process::{Command, ExitStatus, Stdio},
    time::{Duration, Instant},
};

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProcessError {
    #[error("failed to spawn bash for {path}: {source}")]
    Spawn {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("failed to wait for bash script {path}: {source}")]
    Wait {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("bash script {path} exited with {status}")]
    NonZeroExit {
        path: PathBuf,
        status: ExitStatus,
        stdout: String,
        stderr: String,
    },
    #[error("bash script {path} timed out after {timeout_ms}ms")]
    Timeout {
        path: PathBuf,
        timeout_ms: u128,
        stdout: String,
        stderr: String,
    },
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

pub fn run_bash_script(path: &Path) -> Result<String, ProcessError> {
    run_bash_script_with_timeout(path, DEFAULT_TIMEOUT)
}

pub fn run_bash_script_with_timeout(
    path: &Path,
    timeout: Duration,
) -> Result<String, ProcessError> {
    let child = Command::new("/bin/bash")
        .arg(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|source| ProcessError::Spawn {
            path: path.to_path_buf(),
            source,
        })?;

    wait_for_child(path, child, timeout)
}

fn wait_for_child(
    path: &Path,
    mut child: std::process::Child,
    timeout: Duration,
) -> Result<String, ProcessError> {
    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let (stdout, stderr) = read_child_output(&mut child);
                if status.success() {
                    return Ok(stdout);
                }

                return Err(ProcessError::NonZeroExit {
                    path: path.to_path_buf(),
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) if started.elapsed() >= timeout => {
                let _ = child.kill();
                return Err(ProcessError::Timeout {
                    path: path.to_path_buf(),
                    timeout_ms: timeout.as_millis(),
                    stdout: String::new(),
                    stderr: String::new(),
                });
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(10)),
            Err(source) => {
                return Err(ProcessError::Wait {
                    path: path.to_path_buf(),
                    source,
                });
            }
        }
    }
}

fn read_child_output(child: &mut std::process::Child) -> (String, String) {
    let mut stdout = String::new();
    let mut stderr = String::new();

    if let Some(mut stream) = child.stdout.take() {
        let _ = stream.read_to_string(&mut stdout);
    }
    if let Some(mut stream) = child.stderr.take() {
        let _ = stream.read_to_string(&mut stderr);
    }

    (stdout, stderr)
}
