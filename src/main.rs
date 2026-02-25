use std::process::ExitCode;

use clap::Parser;
use reclaw_conformance::{ConformanceRunner, HttpTransport};

#[derive(Debug, Parser)]
#[command(name = "reclaw-conformance", version)]
struct Args {
    #[arg(long, default_value = "http://127.0.0.1:18789")]
    base_url: String,

    #[arg(long)]
    json: bool,
}

fn main() -> ExitCode {
    match run() {
        Ok(code) => code,
        Err(error) => {
            eprintln!("conformance runner failed: {error}");
            ExitCode::from(2)
        }
    }
}

fn run() -> Result<ExitCode, String> {
    let args = Args::parse();
    let transport = HttpTransport::new(args.base_url).map_err(|error| error.to_string())?;
    let report = ConformanceRunner::new(transport).run();

    if args.json {
        let text = serde_json::to_string_pretty(&report)
            .map_err(|error| format!("failed to serialize JSON report: {error}"))?;
        println!("{text}");
    } else {
        println!(
            "scenarios: {} total, {} failed",
            report.total, report.failed
        );
        for outcome in &report.outcomes {
            let status = if outcome.passed { "PASS" } else { "FAIL" };
            println!("[{status}] {} - {}", outcome.name, outcome.detail);
        }
    }

    if report.is_passing() {
        Ok(ExitCode::SUCCESS)
    } else {
        Ok(ExitCode::from(1))
    }
}
