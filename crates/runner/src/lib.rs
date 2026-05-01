#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunReport;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RunError {
    NotImplemented,
}

pub fn run() -> Result<RunReport, RunError> {
    Err(RunError::NotImplemented)
}
