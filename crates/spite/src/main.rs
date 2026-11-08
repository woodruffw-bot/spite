//! A minimal command-line host with no extra JavaScript globals.

mod module_files;

use spite::{Error, Limits, PromiseJobOutcome, PromiseState, Realm, Value};
use std::{
    env,
    fs::File,
    io::{self, Read},
    path::{Path, PathBuf},
    process::ExitCode,
    thread,
    time::Duration,
};

const USAGE: &str = "Usage: spite [--max-steps N] [--module] (--eval SOURCE | FILE | -)";

fn read_source(mut reader: impl Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes).map_err(|e| e.to_string())?;
    String::from_utf8(bytes).map_err(|_| "source must be valid UTF-8".into())
}

fn evaluate_module(
    realm: &mut Realm,
    source: &str,
    filename: Option<&Path>,
) -> Result<Value, String> {
    let module = module_files::load(realm, source, filename).map_err(|e| e.to_string())?;
    let promise = realm
        .evaluate_module_record(&module)
        .map_err(|e| e.to_string())?;
    let promise = realm
        .root_value(promise, usize::MAX)
        .map_err(|e| e.to_string())?;
    let Value::Object(handle) = promise.value() else {
        unreachable!("Module evaluation returns a Promise")
    };
    loop {
        match realm.run_next_promise_job().map_err(|e| e.to_string())? {
            PromiseJobOutcome::Completed => continue,
            PromiseJobOutcome::Reported(error) => return Err(error.to_string()),
            PromiseJobOutcome::NoJob => {}
        }
        match realm
            .inspect_object(handle)
            .map_err(|e| e.to_string())?
            .promise_state()
        {
            Some(PromiseState::Fulfilled(value)) => return Ok(value.clone()),
            Some(PromiseState::Rejected(reason)) => {
                return Err(Error::Thrown(reason.clone()).to_string());
            }
            Some(PromiseState::Pending) => {}
            None => unreachable!("Module evaluation Promise"),
        }
        if realm
            .run_next_atomics_wait_job()
            .map_err(|e| e.to_string())?
        {
            continue;
        }
        let Some(delay) = realm.next_atomics_wait_delay() else {
            return Err("module evaluation Promise did not settle".into());
        };
        thread::sleep(delay.min(Duration::from_millis(10)));
    }
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let mut args = args.as_slice();
    let mut max_steps = None;
    let mut module = false;
    loop {
        match args {
            [flag, rest @ ..] if flag == "--module" => {
                module = true;
                args = rest;
            }
            [flag, count, rest @ ..] if flag == "--max-steps" => {
                let Some(count) = count.to_str().and_then(|count| count.parse::<usize>().ok())
                else {
                    eprintln!("--max-steps requires a non-negative integer");
                    return ExitCode::from(2);
                };
                max_steps = Some(count);
                args = rest;
            }
            [flag] if flag == "--max-steps" => {
                eprintln!("--max-steps requires a non-negative integer");
                return ExitCode::from(2);
            }
            _ => break,
        }
    }
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "{USAGE}\n\nEvaluates a Script and prints its completion value.\nUse --module for a Module with static file imports, awaiting its evaluation.\nExecution work per evaluation/job is unlimited unless --max-steps is supplied.\nThe ECMAScript implementation is incomplete. Input must be UTF-8."
        );
        return ExitCode::SUCCESS;
    }
    let filename = match args {
        [path] if path != "-" => Some(PathBuf::from(path)),
        _ => None,
    };
    let source = match args {
        [flag, text] if flag == "--eval" => text
            .clone()
            .into_string()
            .map_err(|_| "source must be valid UTF-8".into()),
        [path] if path == "-" => read_source(io::stdin().lock()),
        [path] => File::open(path)
            .map_err(|e| e.to_string())
            .and_then(read_source),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = source.and_then(|source| {
        let mut realm = Realm::new(Limits {
            max_steps,
            ..Limits::default()
        });
        if module {
            evaluate_module(&mut realm, &source, filename.as_deref())
        } else {
            realm.eval(&source).map_err(|e| e.to_string())
        }
    });
    match result {
        Ok(value) => {
            println!("{value}");
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}
