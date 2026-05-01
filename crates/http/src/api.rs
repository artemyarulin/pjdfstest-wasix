use std::fs;

use serde::Serialize;

const INDEX_HTML: &str = include_str!("../index.html");
const EMPTY_REPORT: &str = r#"{"results":[]}"#;

#[derive(Serialize)]
#[serde(tag = "type")]
enum ApiEvent {
    RunError { error: runner::RunError },
}

pub(crate) fn index_html() -> &'static str {
    INDEX_HTML
}

pub(crate) fn report_json() -> String {
    fs::read_to_string("/app/report.json").unwrap_or_else(|_| EMPTY_REPORT.to_string())
}

pub(crate) fn run_events(_command: &str) -> Vec<String> {
    let mut events = Vec::new();
    if let Err(error) = runner::run_with_events(|event| events.push(runner_event_json(&event))) {
        events.push(event_json(&ApiEvent::RunError { error }));
    }
    events
}

fn event_json(event: &ApiEvent) -> String {
    serde_json::to_string(event).expect("api event serialization should not fail")
}

fn runner_event_json(event: &runner::RunEvent) -> String {
    serde_json::to_string(event).expect("runner event serialization should not fail")
}
