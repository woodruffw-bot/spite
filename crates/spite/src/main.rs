//! A minimal command-line host with no extra JavaScript globals.

mod module_files;

use spite::{Error, JsString, Limits, ObjectHandle, PromiseJobOutcome, PromiseState, Realm, Value};
use std::fs::File;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::Duration;
use std::{env, thread};

const USAGE: &str = "Usage: spite [--max-steps N] [--max-source-bytes N] [--module] [--check] (--eval SOURCE | FILE | -)";

fn read_source(mut reader: impl Read, max_bytes: Option<usize>) -> Result<String, Error> {
    let mut bytes = Vec::new();
    let result = match max_bytes {
        Some(limit) => reader
            .take(u64::try_from(limit.saturating_add(1)).unwrap_or(u64::MAX))
            .read_to_end(&mut bytes),
        None => reader.read_to_end(&mut bytes),
    };
    result.map_err(|error| Error::Host {
        span: spite::Span::new(0, 0),
        message: error.to_string(),
    })?;
    if max_bytes.is_some_and(|limit| bytes.len() > limit) {
        // Preserve definite encoding errors in the bounded prefix. An incomplete
        // sequence can instead be caused by stopping in a valid UTF-8 character.
        if std::str::from_utf8(&bytes).is_err_and(|error| error.error_len().is_some()) {
            return Err(Error::Host {
                span: spite::Span::new(0, 0),
                message: "source must be valid UTF-8".into(),
            });
        }
        return Err(Error::Limit {
            span: spite::Span::new(0, bytes.len()),
            message: "source size limit exceeded".into(),
        });
    }
    String::from_utf8(bytes).map_err(|_| Error::Host {
        span: spite::Span::new(0, 0),
        message: "source must be valid UTF-8".into(),
    })
}

fn source_error_message(error: Error) -> String {
    match error {
        Error::Host { message, .. } => message,
        error => error.to_string(),
    }
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
    max_source_bytes: Option<usize>,
) -> Result<Value, String> {
    let mut files = module_files::FileModules::for_script(filename, max_source_bytes)
        .map_err(|e| display_error(realm, e))?;
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
    max_source_bytes: Option<usize>,
) -> Result<Value, String> {
    let (mut files, module) =
        module_files::FileModules::load(realm, source, filename, max_source_bytes)
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
    let mut max_source_bytes = None;
    let mut module = false;
    let mut check = false;
    loop {
        match args {
            [flag, rest @ ..] if flag == "--check" => {
                check = true;
                args = rest;
            }
            [flag, rest @ ..] if flag == "--module" => {
                module = true;
                args = rest;
            }
            [flag, count, rest @ ..] if flag == "--max-steps" || flag == "--max-source-bytes" => {
                let Some(count) = count.to_str().and_then(|count| count.parse::<usize>().ok())
                else {
                    eprintln!("{} requires a non-negative integer", flag.to_string_lossy());
                    return ExitCode::from(2);
                };
                if flag == "--max-steps" {
                    max_steps = Some(count);
                } else {
                    max_source_bytes = Some(count);
                }
                args = rest;
            }
            [flag] if flag == "--max-steps" || flag == "--max-source-bytes" => {
                eprintln!("{} requires a non-negative integer", flag.to_string_lossy());
                return ExitCode::from(2);
            }
            _ => break,
        }
    }
    if args.len() == 1 && (args[0] == "--help" || args[0] == "-h") {
        println!(
            "{USAGE}\n\nEvaluates a Script with dynamic file imports, drains queued Promise jobs, and prints its completion value.\nUse --module for a Module with static file imports and top-level Await.\nUse --check to validate syntax and early errors without evaluating input.\nExecution work per evaluation/job is unlimited unless --max-steps is supplied.\nUse --max-source-bytes to limit each parsed source in UTF-8 bytes, including checks and imported Modules.\nThe ECMAScript implementation is incomplete. Input must be UTF-8."
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
        [path] if path == "-" => {
            read_source(io::stdin().lock(), max_source_bytes).map_err(source_error_message)
        }
        [path] => File::open(path)
            .map_err(|e| e.to_string())
            .and_then(|file| read_source(file, max_source_bytes).map_err(source_error_message)),
        _ => {
            eprintln!("{USAGE}");
            return ExitCode::from(2);
        }
    };
    let result = source.and_then(|source| {
        if check {
            let parsed = match (module, max_source_bytes) {
                (true, Some(limit)) => {
                    spite::parse_module_with_source_limit(&source, limit).map(drop)
                }
                (true, None) => spite::parse_module(&source).map(drop),
                (false, Some(limit)) => {
                    spite::parse_script_with_source_limit(&source, limit).map(drop)
                }
                (false, None) => spite::parse_script(&source).map(drop),
            };
            return parsed.map(|()| None).map_err(|error| error.to_string());
        }
        let mut realm = Realm::new_with_supported_import_attributes(
            Limits {
                max_steps,
                max_source_bytes,
                ..Limits::default()
            },
            [JsString::from("type")],
        );
        let value = if module {
            evaluate_module(&mut realm, &source, filename.as_deref(), max_source_bytes)
        } else {
            evaluate_script(&mut realm, &source, filename.as_deref(), max_source_bytes)
        }?;
        Ok(Some(value))
    });
    match result {
        Ok(value) => {
            if let Some(value) = value {
                println!("{value}");
            }
            ExitCode::SUCCESS
        }
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct OpenSource {
        remaining: usize,
        read: usize,
    }

    impl Read for OpenSource {
        fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
            if buffer.is_empty() {
                return Ok(0);
            }
            assert!(self.remaining > 0, "reader must stop before requesting EOF");
            let count = self.remaining.min(buffer.len());
            buffer[..count].fill(b'x');
            self.remaining -= count;
            self.read += count;
            Ok(count)
        }
    }

    #[test]
    fn source_limits_bound_reads_before_eof_and_utf8_decoding() {
        for limit in [0, 1, 64, 4096] {
            let mut reader = OpenSource {
                remaining: limit + 1,
                read: 0,
            };
            assert!(matches!(
                read_source(&mut reader, Some(limit)),
                Err(Error::Limit { .. })
            ));
            assert_eq!(reader.read, limit + 1);
        }
        for limit in [None, Some(2), Some(usize::MAX)] {
            assert_eq!(read_source("π".as_bytes(), limit).unwrap(), "π");
        }
        assert!(matches!(
            read_source("π".as_bytes(), Some(1)),
            Err(Error::Limit { .. })
        ));
        assert!(matches!(
            read_source(&[0xff][..], Some(1)),
            Err(Error::Host { .. })
        ));
        assert!(matches!(
            read_source(&[0xff][..], Some(0)),
            Err(Error::Host { .. })
        ));
        assert!(matches!(
            read_source("π".as_bytes(), Some(0)),
            Err(Error::Limit { .. })
        ));
        assert_eq!(read_source(&[][..], Some(0)).unwrap(), "");
    }
}
