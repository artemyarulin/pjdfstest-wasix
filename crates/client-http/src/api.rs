use std::fs;

const INDEX_HTML: &str = include_str!("../index.html");
const EMPTY_REPORT: &str = r#"{"results":[]}"#;

pub(crate) fn index_html() -> &'static str {
    INDEX_HTML
}

pub(crate) fn report_json() -> String {
    fs::read_to_string("/app/report.json").unwrap_or_else(|_| EMPTY_REPORT.to_string())
}

pub(crate) fn run(_command: &str) -> runner::RunEventReceiver {
    runner::run()
}

pub(crate) fn event_json(event: &runner::RunEvent) -> String {
    serde_json::to_string(event).expect("runner event serialization should not fail")
}
