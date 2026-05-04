use serde::Serialize;

use crate::{
    test_case::TestCase,
    test_result::{TestResult, TestResultPassStatus},
};

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

impl TestReport {
    pub fn new(results: &[(TestCase, TestResult)], duration_ms: u128) -> Self {
        let mut report = Self {
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
}
