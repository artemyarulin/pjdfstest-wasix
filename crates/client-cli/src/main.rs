use std::{
    fs,
    io::Write,
    path::{Path, PathBuf},
};

use clap::Parser;
use env_logger::{Builder, Env, Target};
use runner::RunEvent;

const REPORT_PATH: &str = "/data/report.json";

#[derive(Debug, Parser)]
#[command(about = "Run pjdfstest and print runner events")]
struct Args {}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let _args = Args::parse();
    init_logger();

    for event in runner::run() {
        handle_event(event)?;
    }

    Ok(())
}

fn handle_event(event: RunEvent) -> Result<(), Box<dyn std::error::Error>> {
    log::info!("{}", serde_json::to_string(&event)?);
    if let RunEvent::TestFinished { report } = event {
        write_report(Path::new(REPORT_PATH), &report)?;
    }
    Ok(())
}

fn init_logger() {
    Builder::from_env(Env::default().default_filter_or("info"))
        .target(Target::Stdout)
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .init();
}

fn write_report(
    path: &Path,
    report: &runner::TestReport,
) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = writable_parent(path) {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, serde_json::to_string_pretty(report)?)?;
    Ok(())
}

fn writable_parent(path: &Path) -> Option<PathBuf> {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .map(Path::to_path_buf)
}
