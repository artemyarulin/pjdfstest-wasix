mod fs;
mod process;
mod report;
mod test_case;
mod test_result;

use std::{
    path::{Path, PathBuf},
    sync::mpsc::{self, SendError},
    thread,
    time::Instant,
};

use serde::Serialize;

pub use process::{ProcessError, run_bash_script};
pub use report::TestReport;
pub use test_case::{TestCase, discover};
pub use test_result::{TestResult, TestResultPassStatus};

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum RunEvent {
    RunStarted,
    TestsDiscovered { tests: Vec<TestCase> },
    TestResult { test: TestCase, result: TestResult },
    TestFinished { report: TestReport },
    RunError { error: RunError },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum RunError {
    Read { message: String },
    MissingDesc { path: PathBuf },
}

impl From<std::io::Error> for RunError {
    fn from(value: std::io::Error) -> Self {
        RunError::Read {
            message: value.to_string(),
        }
    }
}

pub type RunEventReceiver = mpsc::Receiver<RunEvent>;

pub fn run() -> RunEventReceiver {
    run_in(PathBuf::from("/app/tests"))
}

pub fn run_in(test_dir: PathBuf) -> RunEventReceiver {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        if let Err(error) = run_to_channel(&test_dir, tx) {
            eprintln!("failed to send runner event: {error}");
        }
    });
    rx
}

fn run_to_channel(test_dir: &Path, tx: mpsc::Sender<RunEvent>) -> Result<(), SendError<RunEvent>> {
    let started = Instant::now();
    tx.send(RunEvent::RunStarted)?;

    let cases = match discover(test_dir) {
        Ok(cases) => cases,
        Err(error) => {
            tx.send(RunEvent::RunError { error })?;
            return Ok(());
        }
    };
    tx.send(RunEvent::TestsDiscovered {
        tests: cases.clone(),
    })?;

    let mut results = Vec::new();
    for case in cases {
        let (test, result) = run_test(case);
        tx.send(RunEvent::TestResult {
            test: test.clone(),
            result: result.clone(),
        })?;
        results.push((test, result));
    }

    tx.send(RunEvent::TestFinished {
        report: TestReport::new(&results, started.elapsed().as_millis()),
    })?;

    Ok(())
}

fn run_test(case: TestCase) -> (TestCase, TestResult) {
    let result = match run_bash_script(&case.path) {
        Ok(stdout) => TestResult {
            passed: TestResultPassStatus::Passed,
            details: stdout,
        },
        Err(error) => TestResult {
            passed: TestResultPassStatus::Failed,
            details: process_error_details(&error),
        },
    };
    (case, result)
}

fn process_error_details(error: &ProcessError) -> String {
    match error {
        ProcessError::NonZeroExit { stdout, stderr, .. }
        | ProcessError::Timeout { stdout, stderr, .. } => {
            let mut details = error.to_string();
            if !stdout.is_empty() {
                details.push_str("\n\nstdout:\n");
                details.push_str(stdout);
            }
            if !stderr.is_empty() {
                details.push_str("\n\nstderr:\n");
                details.push_str(stderr);
            }
            details
        }
        _ => error.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn run_tests_converts_process_errors_to_failed_results() {
        let missing = TestCase {
            group: "missing".to_string(),
            path: PathBuf::from("/definitely/missing/test.t"),
            name: "test".to_string(),
            desc: "missing test".to_string(),
        };

        let (_, result) = run_test(missing);
        assert_eq!(result.passed, TestResultPassStatus::Failed);
        assert!(!result.details.is_empty());
    }
}
