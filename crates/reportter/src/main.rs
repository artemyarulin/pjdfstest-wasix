use std::{env, fs, path::Path};

use serde::Serialize;

#[derive(Debug, Serialize)]
struct Report {
    summary: Summary,
    groups: Vec<GroupReport>,
}

#[derive(Debug, Default, Serialize)]
struct Summary {
    tests: usize,
    cases: usize,
    passed: usize,
    failed: usize,
    planned: usize,
}

#[derive(Debug, Default, Serialize)]
struct GroupReport {
    name: String,
    summary: Summary,
    tests: Vec<TestReport>,
}

#[derive(Debug, Default, Serialize)]
struct TestReport {
    name: String,
    path: String,
    planned: Option<usize>,
    summary: Summary,
    cases: Vec<CaseReport>,
    output: Vec<String>,
}

#[derive(Debug, Serialize)]
struct CaseReport {
    status: CaseStatus,
    number: Option<usize>,
    message: String,
    raw: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum CaseStatus {
    Passed,
    Failed,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let Some(path) = env::args().nth(1) else {
        eprintln!("usage: reportter <report.txt>");
        std::process::exit(2);
    };

    let input = fs::read_to_string(&path)?;
    let report = parse_report(&input);
    let json = serde_json::to_string_pretty(&report)?;
    fs::write("report.json", json)?;

    println!(
        "wrote report.json: {} tests, {} cases, {} passed, {} failed",
        report.summary.tests, report.summary.cases, report.summary.passed, report.summary.failed
    );

    Ok(())
}

fn parse_report(input: &str) -> Report {
    let mut groups: Vec<GroupReport> = Vec::new();
    let mut current: Option<TestReport> = None;

    for line in input.lines() {
        if let Some(path) = line.strip_prefix("==> ") {
            push_test(&mut groups, current.take());
            current = Some(TestReport {
                name: Path::new(path)
                    .file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or(path)
                    .to_string(),
                path: path.to_string(),
                ..TestReport::default()
            });
            continue;
        }

        let Some(test) = current.as_mut() else {
            continue;
        };

        if let Some(planned) = line.strip_prefix("1..").and_then(|n| n.parse::<usize>().ok()) {
            test.planned = Some(planned);
            test.summary.planned = planned;
            continue;
        }

        if line.starts_with("ok ") || line == "ok" {
            test.cases.push(parse_case(line, CaseStatus::Passed));
            continue;
        }

        if line.starts_with("not ok") {
            test.cases.push(parse_case(line, CaseStatus::Failed));
            continue;
        }

        if !line.trim().is_empty() {
            test.output.push(line.to_string());
        }
    }

    push_test(&mut groups, current);

    let mut summary = Summary::default();
    for group in &mut groups {
        group.summary = Summary::default();
        for test in &mut group.tests {
            summarize_test(test);
            add_summary(&mut group.summary, &test.summary);
        }
        add_summary(&mut summary, &group.summary);
    }

    Report { summary, groups }
}

fn parse_case(line: &str, status: CaseStatus) -> CaseReport {
    let rest = line
        .strip_prefix("not ok")
        .or_else(|| line.strip_prefix("ok"))
        .unwrap_or("")
        .trim();

    let (number, message) = match rest.split_once(' ') {
        Some((first, remaining)) => match first.parse::<usize>() {
            Ok(number) => (Some(number), remaining.trim().to_string()),
            Err(_) => (None, rest.to_string()),
        },
        None => match rest.parse::<usize>() {
            Ok(number) => (Some(number), String::new()),
            Err(_) => (None, rest.to_string()),
        },
    };

    CaseReport {
        status,
        number,
        message,
        raw: line.to_string(),
    }
}

fn push_test(groups: &mut Vec<GroupReport>, test: Option<TestReport>) {
    let Some(test) = test else {
        return;
    };
    let group_name = test.path.split('/').next().unwrap_or("unknown").to_string();

    if let Some(group) = groups.iter_mut().find(|group| group.name == group_name) {
        group.tests.push(test);
    } else {
        groups.push(GroupReport {
            name: group_name,
            tests: vec![test],
            ..GroupReport::default()
        });
    }
}

fn summarize_test(test: &mut TestReport) {
    test.summary.tests = 1;
    test.summary.cases = test.cases.len();
    test.summary.passed = test
        .cases
        .iter()
        .filter(|case| matches!(case.status, CaseStatus::Passed))
        .count();
    test.summary.failed = test
        .cases
        .iter()
        .filter(|case| matches!(case.status, CaseStatus::Failed))
        .count();
    test.summary.planned = test.planned.unwrap_or(0);
}

fn add_summary(total: &mut Summary, item: &Summary) {
    total.tests += item.tests;
    total.cases += item.cases;
    total.passed += item.passed;
    total.failed += item.failed;
    total.planned += item.planned;
}
