//! A minimal command-line host with no extra JavaScript globals.

mod module_files;

use spite::{Error, JsString, Limits, ObjectHandle, PromiseJobOutcome, PromiseState, Realm, Value};
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

fn data_string(realm: &Realm, object: &ObjectHandle, key: &str) -> Option<JsString> {
    let key = JsString::from(key);
    let mut next = Some(object.clone());
    while let Some(handle) = next {
        let object = realm.inspect_object(&handle).ok()?;
        if let Some(property) = object.own_property(&key) {
            return match &property.as_data()?.value {
                Value::String(value) => Some(value.clone()),
                _ => None,
            };
        }
        next = object.prototype().cloned();
    }
    None
}

fn diagnostic_text(value: &JsString) -> String {
    value.to_utf8().map_or_else(
        |_| format!("{value:?}"),
        |text| text.escape_debug().to_string(),
    )
}

fn display_error(realm: &Realm, error: Error) -> String {
    let Error::Thrown(Value::Object(handle)) = &error else {
        return error.to_string();
    };
    if !realm
        .inspect_object(handle)
        .is_ok_and(|object| object.is_error())
    {
        return error.to_string();
    }
    // Inspect only stored data. Diagnostics must not execute getters, Proxy
    // traps, user toString methods or coercion while reporting an exception.
    let name = data_string(realm, handle, "name").unwrap_or_else(|| JsString::from("Error"));
    let message = data_string(realm, handle, "message").unwrap_or_default();
    let name = diagnostic_text(&name);
    let message = diagnostic_text(&message);
    let text = if name.is_empty() {
        message
    } else if message.is_empty() {
        name
    } else {
        format!("{name}: {message}")
    };
    format!("uncaught {text}")
}

fn evaluate_script(
    realm: &mut Realm,
    source: &str,
    filename: Option<&Path>,
) -> Result<Value, String> {
    let mut files =
        module_files::FileModules::for_script(filename).map_err(|e| display_error(realm, e))?;
    realm.enable_module_imports();
    let value = realm.eval(source).map_err(|e| display_error(realm, e))?;
    let _completion_root = if matches!(&value, Value::Object(_)) {
        Some(
            realm
                .root_value(value.clone(), usize::MAX)
                .map_err(|e| e.to_string())?,
        )
    } else {
        None
    };
    loop {
        files
            .service_requests(realm)
            .map_err(|e| display_error(realm, e))?;
        match realm
            .run_next_promise_job()
            .map_err(|e| display_error(realm, e))?
        {
            PromiseJobOutcome::Completed => {}
            PromiseJobOutcome::Reported(error) => return Err(display_error(realm, error)),
            PromiseJobOutcome::NoJob => return Ok(value),
        }
    }
}

fn evaluate_module(
    realm: &mut Realm,
    source: &str,
    filename: Option<&Path>,
) -> Result<Value, String> {
    let (mut files, module) = module_files::FileModules::load(realm, source, filename)
        .map_err(|e| display_error(realm, e))?;
    realm.enable_module_imports();
    let promise = realm
        .evaluate_module_record(&module)
        .map_err(|e| display_error(realm, e))?;
    let promise = realm
        .root_value(promise, usize::MAX)
        .map_err(|e| e.to_string())?;
    let Value::Object(handle) = promise.value() else {
        unreachable!("Module evaluation returns a Promise")
    };
    loop {
        files
            .service_requests(realm)
            .map_err(|e| display_error(realm, e))?;
        match realm
            .run_next_promise_job()
            .map_err(|e| display_error(realm, e))?
        {
            PromiseJobOutcome::Completed => continue,
            PromiseJobOutcome::Reported(error) => return Err(display_error(realm, error)),
            PromiseJobOutcome::NoJob => {}
        }
        match realm
            .inspect_object(handle)
            .map_err(|e| e.to_string())?
            .promise_state()
        {
            Some(PromiseState::Fulfilled(value)) => return Ok(value.clone()),
            Some(PromiseState::Rejected(reason)) => {
                return Err(display_error(realm, Error::Thrown(reason.clone())));
            }
            Some(PromiseState::Pending) => {}
            None => unreachable!("Module evaluation Promise"),
        }
        if realm
            .run_next_atomics_wait_job()
            .map_err(|e| display_error(realm, e))?
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
            "{USAGE}\n\nEvaluates a Script with dynamic file imports, drains queued Promise jobs, and prints its completion value.\nUse --module for a Module with static file imports and top-level Await.\nExecution work per evaluation/job is unlimited unless --max-steps is supplied.\nThe ECMAScript implementation is incomplete. Input must be UTF-8."
        );
        return ExitCode::SUCCESS;
    }
    let filename = match args {
        [path] if path != "-" => Some(PathBuf::from(path)),
        _ => None,
    };
    let source = match args {
        [flag] if flag == "--eval" => {
            eprintln!("--eval requires a source argument");
            return ExitCode::from(2);
        }
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
            evaluate_script(&mut realm, &source, filename.as_deref())
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
