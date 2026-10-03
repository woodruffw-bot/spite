//! Command-line gate for the reviewed Test262 corpus, not a whole-suite pass rate.

use spite_test262::run_corpus;
use std::{env, path::Path, process::ExitCode};

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    if args.len() != 1 {
        eprintln!("Usage: spite-test262 CORPUS_DIRECTORY");
        return ExitCode::from(2);
    }
    let report = match run_corpus(Path::new(&args[0])) {
        Ok(report) => report,
        Err(error) => {
            eprintln!("Corpus error: {error}");
            return ExitCode::from(2);
        }
    };
    for test in &report.tests {
        match &test.result {
            Ok(cases) => {
                for case in cases {
                    println!("{} {:?}: {:?}", test.path, case.mode, case.outcome);
                }
            }
            Err(error) => println!("{}: {error}", test.path),
        }
    }
    println!(
        "Reviewed corpus at {}: {} source files",
        report.revision,
        report.tests.len()
    );
    println!(
        "{}",
        report
            .counts()
            .iter()
            .map(|(name, count)| format!("{name}={count}"))
            .collect::<Vec<_>>()
            .join(" ")
    );
    if report.is_success() {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
