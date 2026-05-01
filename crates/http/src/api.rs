use std::fs;

const INDEX_HTML: &str = include_str!("../index.html");
const EMPTY_REPORT: &str =
    r#"{"summary":{"tests":0,"cases":0,"passed":0,"failed":0,"planned":0},"groups":[]}"#;

pub(crate) fn index_html() -> &'static str {
    INDEX_HTML
}

pub(crate) fn report_json() -> String {
    fs::read_to_string("/app/report.json").unwrap_or_else(|_| EMPTY_REPORT.to_string())
}

pub(crate) fn run_events(command: &str) -> Vec<String> {
    let mut events = vec![format!(
        r#"{{"type":"started","command":{}}}"#,
        json_string(command)
    )];

    events.push(match runner::run() {
        Ok(_report) => r#"{"type":"RunReport","report":{}}"#.to_string(),
        Err(runner::RunError::NotImplemented) => {
            r#"{"type":"RunError","error":"NotImplemented","message":"runner is not implemented yet"}"#
                .to_string()
        }
    });

    events
}

fn json_string(value: &str) -> String {
    format!("\"{}\"", json_escape(value))
}

fn json_escape(value: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            ch if ch.is_control() => out.push_str(&format!("\\u{:04x}", ch as u32)),
            ch => out.push(ch),
        }
    }
    out
}
