mod fs;
mod test_case;

use std::path::{Path, PathBuf};

use serde::Serialize;

pub use test_case::{TestCase, discover};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RunReport {
    pub results: Vec<(TestCase, TestResult)>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestResult {
    pub passed: TestResultPassStatus,
    pub error: Option<String>,
    pub stdout: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TestResultPassStatus {
    Passed,
    Failed,
    Ignored,
    Skip,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum RunError {
    NotImplemented,
    ReadDir { path: PathBuf, message: String },
    ReadFile { path: PathBuf, message: String },
    MissingDesc { path: PathBuf },
}

pub fn run() -> Result<RunReport, RunError> {
    let cases = discover(Path::new("/app/tests"))?;
    run_tests(cases)
}

pub fn run_tests(cases: Vec<TestCase>) -> Result<RunReport, RunError> {
    let results = cases
        .into_iter()
        .map(|case| {
            (
                case,
                TestResult {
                    passed: TestResultPassStatus::Skip,
                    error: None,
                    stdout: String::new(),
                },
            )
        })
        .collect();

    Ok(RunReport { results })
}
