//! A minimal command-line host with no extra JavaScript globals.

use spite::{Limits, Realm};
use spite_parser::MAX_SOURCE_BYTES;
use std::{
    env,
    fs::File,
    io::{self, Read},
    process::ExitCode,
};

fn read_source(reader: impl Read) -> Result<String, String> {
    let mut bytes = Vec::new();
    reader
        .take(MAX_SOURCE_BYTES as u64 + 1)
        .read_to_end(&mut bytes)
        .map_err(|e| e.to_string())?;
    if bytes.len() > MAX_SOURCE_BYTES {
        return Err("source size limit exceeded".into());
    }
    String::from_utf8(bytes).map_err(|_| "source must be valid UTF-8".into())
}

fn main() -> ExitCode {
    let args: Vec<_> = env::args_os().skip(1).collect();
    let (args, max_steps) = match args.as_slice() {
        [flag, count, rest @ ..] if flag == "--max-steps" => {
            let Some(count) = count.to_str().and_then(|count| count.parse::<usize>().ok()) else {
                eprintln!("--max-steps requires a non-negative integer");
                return ExitCode::from(2);
            };
            (rest, Some(count))
        }
        [flag] if flag == "--max-steps" => {
            eprintln!("--max-steps requires a non-negative integer");
            return ExitCode::from(2);
        }
        args => (args, None),
    };
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "Usage: spite [--max-steps N] (--eval SOURCE | FILE | -)\n\nEvaluates a Script and prints its completion value.\nExecution work is unlimited unless --max-steps is supplied.\nThe ECMAScript implementation is incomplete. Input must be UTF-8."
        );
        return ExitCode::SUCCESS;
    }
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
            eprintln!("Usage: spite [--max-steps N] (--eval SOURCE | FILE | -)");
            return ExitCode::from(2);
        }
    };
    let result = source.and_then(|source| {
        Realm::new(Limits {
            max_steps,
            ..Limits::default()
        })
        .eval(&source)
        .map_err(|e| e.to_string())
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
