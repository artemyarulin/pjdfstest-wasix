mod fs;
mod test_case;

use std::path::{Path, PathBuf};
use std::time::Instant;

use serde::Serialize;

pub use test_case::{TestCase, discover};

#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct TestReport {
    pub duration_ms: u128,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub ignored: usize,
    pub skipped: usize,
    pub success_rate: f64,
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

#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(tag = "type")]
pub enum RunEvent {
    RunStarted,
    TestsDiscovered { tests: Vec<TestCase> },
    TestResult { test: TestCase, result: TestResult },
    TestFinished { report: TestReport },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind")]
pub enum RunError {
    NotImplemented,
    ReadDir { path: PathBuf, message: String },
    ReadFile { path: PathBuf, message: String },
    MissingDesc { path: PathBuf },
}

pub fn run_with_events(mut emit: impl FnMut(RunEvent)) -> Result<(), RunError> {
    run_with_events_in(Path::new("/app/tests"), &mut emit)
}

pub fn run_with_events_in(test_dir: &Path, mut emit: impl FnMut(RunEvent)) -> Result<(), RunError> {
    let started = Instant::now();
    emit(RunEvent::RunStarted);

    let cases = discover(test_dir)?;
    emit(RunEvent::TestsDiscovered {
        tests: cases.clone(),
    });

    let results = run_tests(cases)?;
    for (test, result) in &results {
        emit(RunEvent::TestResult {
            test: test.clone(),
            result: result.clone(),
        });
    }

    emit(RunEvent::TestFinished {
        report: summarize_results(&results, started.elapsed().as_millis()),
    });
    Ok(())
}

pub fn run_tests(cases: Vec<TestCase>) -> Result<Vec<(TestCase, TestResult)>, RunError> {
    Ok(cases
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
        .collect())
}

fn summarize_results(results: &[(TestCase, TestResult)], duration_ms: u128) -> TestReport {
    let mut report = TestReport {
        duration_ms,
        total: results.len(),
        passed: 0,
        failed: 0,
        ignored: 0,
        skipped: 0,
        success_rate: 0.0,
    };

    for (_, result) in results {
        match result.passed {
            TestResultPassStatus::Passed => report.passed += 1,
            TestResultPassStatus::Failed => report.failed += 1,
            TestResultPassStatus::Ignored => report.ignored += 1,
            TestResultPassStatus::Skip => report.skipped += 1,
        }
    }

    let completed = report.passed + report.failed;
    if completed > 0 {
        report.success_rate = (report.passed as f64 / completed as f64) * 100.0;
    }

    report
}
