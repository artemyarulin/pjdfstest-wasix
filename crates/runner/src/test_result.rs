use serde::Serialize;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct TestResult {
    pub passed: TestResultPassStatus,
    pub details: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum TestResultPassStatus {
    Passed,
    Failed,
    Ignored,
    Skip,
}
