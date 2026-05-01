use std::fs;

use serde::Serialize;

const INDEX_HTML: &str = include_str!("../index.html");
const EMPTY_REPORT: &str = r#"{"results":[]}"#;

#[derive(Serialize)]
#[serde(tag = "type")]
enum ApiEvent {
    #[serde(rename = "started")]
    Started {
        command: String,
    },
    RunReport {
        report: runner::RunReport,
    },
    RunError {
        error: runner::RunError,
    },
}

pub(crate) fn index_html() -> &'static str {
    INDEX_HTML
}

pub(crate) fn report_json() -> String {
    fs::read_to_string("/app/report.json").unwrap_or_else(|_| EMPTY_REPORT.to_string())
}

pub(crate) fn run_events(command: &str) -> Vec<String> {
    let mut events = vec![event_json(&ApiEvent::Started {
        command: command.to_string(),
    })];

    events.push(match runner::run() {
        Ok(report) => event_json(&ApiEvent::RunReport { report }),
        Err(error) => event_json(&ApiEvent::RunError { error }),
    });

    events
}

fn event_json(event: &ApiEvent) -> String {
    serde_json::to_string(event).expect("api event serialization should not fail")
}
