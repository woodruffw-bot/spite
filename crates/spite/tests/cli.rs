//! Tests of the command-line host boundary.

use std::{
    io::Write,
    process::{Command, Stdio},
};

#[test]
fn evaluates_an_argument() {
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--eval", "let x = 6; x * 7"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "42");
    assert!(output.stderr.is_empty());
}

#[test]
fn syntax_checks_do_not_evaluate_source_or_load_modules() {
    for source in ["throw 7", "missingCall()", "function f(a,a){}"] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--check", "--max-steps", "0", "--eval", source])
            .output()
            .unwrap();
        assert!(output.status.success(), "{source}: {output:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
    for flags in [["--check", "--module"], ["--module", "--check"]] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(flags)
            .args(["--eval", "import './absent.mjs' with {type:'json'};export const answer=await Promise.resolve(42);throw 7"])
            .output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
}

#[test]
fn syntax_checks_preserve_goal_early_errors_and_unsupported_diagnostics() {
    for (module, source, expected) in [
        (false, "const x;", "Syntax"),
        (false, "export {};", "Syntax"),
        (false, "import.meta", "Syntax"),
        (true, "export {missing};", "Syntax"),
        (true, "function f(a,a){}", "Syntax"),
        (true, "with({}){}", "Syntax"),
        (
            false,
            "async function f(){await using resource=null}",
            "Unsupported",
        ),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_spite"));
        command.arg("--check");
        if module {
            command.arg("--module");
        }
        let output = command.args(["--eval", source]).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with(expected)
        );
    }
}

#[test]
fn syntax_checks_accept_files_and_lossless_utf8_stdin() {
    for (module, path) in [
        (
            false,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/module-graph/dynamic-script.js"
            ),
        ),
        (
            true,
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/module-graph/root.mjs"
            ),
        ),
    ] {
        let mut command = Command::new(env!("CARGO_BIN_EXE_spite"));
        command.current_dir(std::env::temp_dir()).arg("--check");
        if module {
            command.arg("--module");
        }
        let output = command.arg(path).output().unwrap();
        assert!(output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
    for (flags, source) in [
        (vec!["--check"], "const π=42;throw π"),
        (vec!["--module", "--check"], "const π=await 42;export{π}"),
    ] {
        let output = stdin_with_flags(source.as_bytes(), &flags);
        assert!(output.status.success(), "{output:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
        let output = stdin_with_flags(&[0xff], &flags);
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .contains("valid UTF-8")
        );
    }
}

#[test]
fn execution_work_limit_is_optional() {
    let source = "'a'.repeat(2000).indexOf('a'.repeat(999)+'b')";
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--eval", source])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "-1");
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--max-steps", "100000", "--eval", source])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("Limit")
    );
    insta::allow_duplicates! {
        for count in ["-1", "invalid", "184467440737095516160"] {
            let output = Command::new(env!("CARGO_BIN_EXE_spite"))
                .args(["--max-steps", count, "--eval", "1"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(2));
            insta::assert_snapshot!(
                String::from_utf8(output.stderr).unwrap(),
                @"--max-steps requires a non-negative integer"
            );
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--max-steps", "0", "--eval", "1"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("Limit")
    );
}

#[test]
fn source_byte_limits_cover_goals_checks_utf8_and_argument_validation() {
    for module in [false, true] {
        let source = if module {
            "export const π=7"
        } else {
            "let π=7;π"
        };
        for check in [false, true] {
            for limit in [source.len(), source.len() - 1] {
                let count = limit.to_string();
                let mut flags = vec!["--max-source-bytes", count.as_str()];
                if module {
                    flags.push("--module");
                }
                if check {
                    flags.push("--check");
                }
                let argument = Command::new(env!("CARGO_BIN_EXE_spite"))
                    .args(&flags)
                    .args(["--eval", source])
                    .output()
                    .unwrap();
                let stdin = stdin_with_flags(source.as_bytes(), &flags);
                for output in [argument, stdin] {
                    if limit == source.len() {
                        assert!(output.status.success(), "{flags:?}: {output:?}");
                        assert!(output.stderr.is_empty());
                        if check {
                            assert!(output.stdout.is_empty());
                        } else {
                            assert_eq!(
                                String::from_utf8(output.stdout).unwrap().trim(),
                                if module { "undefined" } else { "7" }
                            );
                        }
                    } else {
                        assert_eq!(output.status.code(), Some(1), "{flags:?}: {output:?}");
                        assert!(output.stdout.is_empty());
                        let diagnostic = String::from_utf8(output.stderr).unwrap();
                        assert!(
                            diagnostic.starts_with("Limit")
                                && diagnostic.contains("source size limit exceeded"),
                            "{diagnostic}"
                        );
                    }
                }
            }
            let mut command = Command::new(env!("CARGO_BIN_EXE_spite"));
            if module {
                command.arg("--module");
            }
            if check {
                command.args(["--check", "--max-steps", "0"]);
            }
            let output = command
                .args(["--max-source-bytes", "0", "--eval", ""])
                .output()
                .unwrap();
            assert!(output.status.success(), "{output:?}");
        }
    }
    for args in [
        vec!["--max-source-bytes"],
        vec!["--max-source-bytes", "-1", "--eval", "1"],
        vec!["--max-source-bytes", "invalid", "--eval", "1"],
        vec!["--max-source-bytes", "184467440737095516160", "--eval", "1"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(args)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(2));
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().trim(),
            "--max-source-bytes requires a non-negative integer"
        );
    }
    let output = stdin_with_flags(&[0xff], &["--check", "--max-source-bytes", "0"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("valid UTF-8")
    );
    for (limit, expected) in [("0", "Limit"), ("1", "Syntax")] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--check", "--max-source-bytes", limit, "--eval", "("])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with(expected)
        );
    }
}

#[test]
fn source_byte_limits_reach_imported_and_dynamically_compiled_sources() {
    let directory = std::env::temp_dir().join(format!(
        "spite-source-limit-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir(&directory).unwrap();
    std::fs::write(
        directory.join("large.mjs"),
        format!("export const value=7;/*{}*/", "x".repeat(256)),
    )
    .unwrap();
    for (file, source, module) in [
        ("script.js", "import('./large.mjs');42", false),
        ("static.mjs", "import './large.mjs';", true),
        ("dynamic.mjs", "await import('./large.mjs');", true),
    ] {
        std::fs::write(directory.join(file), source).unwrap();
        let mut command = Command::new(env!("CARGO_BIN_EXE_spite"));
        command
            .current_dir(&directory)
            .args(["--max-source-bytes", "64"]);
        if module {
            command.arg("--module");
        }
        let output = command.arg(file).output().unwrap();
        assert_eq!(output.status.code(), Some(1), "{file}: {output:?}");
        assert!(output.stdout.is_empty());
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(
            diagnostic.starts_with("Limit") && diagnostic.contains("source size limit exceeded"),
            "{diagnostic}"
        );
        let mut command = Command::new(env!("CARGO_BIN_EXE_spite"));
        command
            .current_dir(&directory)
            .args(["--check", "--max-source-bytes", "64"]);
        if module {
            command.arg("--module");
        }
        let output = command.arg(file).output().unwrap();
        assert!(output.status.success(), "{file}: {output:?}");
        assert!(output.stdout.is_empty() && output.stderr.is_empty());
    }
    for source in [
        "try{eval('1;'.repeat(64))}catch{42}",
        "try{Function('return 1;'.repeat(16))}catch{42}",
        "Promise.resolve().then(()=>eval('1;'.repeat(64)));42",
    ] {
        assert!(source.len() <= 64);
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--max-source-bytes", "64", "--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty());
        let diagnostic = String::from_utf8(output.stderr).unwrap();
        assert!(
            diagnostic.starts_with("Limit") && diagnostic.contains("source size limit exceeded"),
            "{diagnostic}"
        );
    }
    std::fs::remove_dir_all(directory).unwrap();
}

#[test]
fn errors_have_a_failing_exit_status() {
    for (source, expected) in [
        ("missing", "ReferenceError"),
        (
            "async function f(){await using resource=null}",
            "Unsupported",
        ),
        ("1n + 1", "TypeError"),
        ("1n / 0n", "RangeError"),
        ("throw 7", "uncaught 7"),
        ("const x;", "Syntax"),
        ("import.meta", "Syntax"),
        ("export {}", "Syntax"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
        assert!(output.stdout.is_empty());
    }
    for source in [
        "async function* gap(){};gap();undefined",
        "async function* gap(){yield*[]};gap().next();undefined",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--eval", source])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            "undefined"
        );
        assert!(output.stderr.is_empty());
    }
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--eval", "function* gap(){}; gap().next().done"])
        .output()
        .unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "true");
    assert!(output.stderr.is_empty());
}

#[test]
fn bigint_host_display_is_exact_and_language_string_conversion_is_decimal() {
    for (source, expected) in [
        ("2n ** 64n + 1n", "0x10000000000000001n"),
        ("`${2n ** 64n + 1n}`", "\"18446744073709551617\""),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--eval", source])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), expected);
    }
}

fn stdin(source: &[u8]) -> std::process::Output {
    stdin_with_flags(source, &[])
}

fn stdin_with_flags(source: &[u8], flags: &[&str]) -> std::process::Output {
    let mut child = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(flags)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child.stdin.take().unwrap().write_all(source).unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn stdin_is_utf8_without_lossy_decoding() {
    let output = stdin(b"1 + 2");
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "3");
    let output = stdin(&[0xff]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("valid UTF-8")
    );
}

#[test]
fn stdin_accepts_sources_above_the_former_default_size_cutoff() {
    let source = " ".repeat(1024 * 1024 + 1) + "42";
    let output = stdin(source.as_bytes());
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "42");
}

#[test]
fn modules_use_the_module_goal_and_await_evaluation() {
    for source in [
        "export const value=42;if(this!==undefined||Object.hasOwn(globalThis,'value'))throw 7",
        "const value=await Promise.resolve(42);if(value!==42)throw value;export{value}",
        "const value=await{then(resolve){resolve(42)}};if(value!==42)throw value",
        "const view=new Int32Array(new SharedArrayBuffer(4));if(await Atomics.waitAsync(view,0,0,1).value!=='timed-out')throw 7",
    ] {
        for flags in [
            vec!["--module"],
            vec!["--module", "--max-steps", "100000"],
            vec!["--max-steps", "100000", "--module"],
        ] {
            let output = Command::new(env!("CARGO_BIN_EXE_spite"))
                .args(flags)
                .args(["--eval", source])
                .output()
                .unwrap();
            assert!(output.status.success(), "{source}: {output:?}");
            assert_eq!(
                String::from_utf8(output.stdout).unwrap().trim(),
                "undefined"
            );
            assert!(output.stderr.is_empty());
        }
    }
}

#[test]
fn module_failures_and_host_limits_have_failing_exit_statuses() {
    for (source, expected) in [
        ("await Promise.reject(7)", "uncaught 7"),
        ("throw 8", "uncaught 8"),
        ("await new Promise(()=>{})", "did not settle"),
        ("export const = 1", "Syntax"),
        ("import 'missing'", "Host"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--module", "--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args([
            "--module",
            "--max-steps",
            "100000",
            "--eval",
            "try{await 0;while(true){}}catch{}finally{throw 7}",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("Limit")
    );
}

#[test]
fn modules_accept_files_and_utf8_stdin() {
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args([
            "--module",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/standalone-module.mjs"
            ),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "undefined"
    );
    let output = stdin_with_flags(
        "const π=await Promise.resolve(42);export{π}".as_bytes(),
        &["--module"],
    );
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "undefined"
    );
    let output = stdin_with_flags(&[0xff], &["--module"]);
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .contains("valid UTF-8")
    );
}

#[test]
fn modules_load_relative_graphs_with_shared_records_cycles_and_awaits() {
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .current_dir(std::env::temp_dir())
        .args([
            "--module",
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/module-graph/root.mjs"
            ),
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "undefined"
    );
    assert!(output.stderr.is_empty());
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .current_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/module-graph"
        ))
        .args([
            "--module",
            "--eval",
            "import{readRoot}from'./root.mjs';if(readRoot()!==42)throw 7",
        ])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(
        String::from_utf8(output.stdout).unwrap().trim(),
        "undefined"
    );
}

#[test]
fn module_file_loading_preserves_syntax_host_and_attribute_failures() {
    for (source, expected) in [
        ("import './invalid.mjs'", "Syntax"),
        ("import './absent.mjs'", "Host"),
        ("import './absent.mjs' with {type:'json'}", "SyntaxError"),
        ("import '\\uD800'", "valid Unicode"),
        ("import '\\u0000'", "Host"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .current_dir(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/module-graph"
            ))
            .args(["--module", "--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
    }
}

#[test]
fn dynamic_file_imports_share_static_records_and_follow_nested_referrers() {
    for file in ["dynamic-root.mjs", "dynamic-errors.mjs"] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .current_dir(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/module-graph"
            ))
            .args(["--module", file])
            .output()
            .unwrap();
        assert!(output.status.success(), "{file}: {output:?}");
        assert_eq!(
            String::from_utf8(output.stdout).unwrap().trim(),
            "undefined"
        );
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn dynamic_file_imports_keep_host_failures_outside_language_rejections() {
    for (file, expected) in [
        ("./absent.mjs", "Host"),
        ("./unsupported.mjs", "Unsupported"),
        ("./forever.mjs", "Limit"),
    ] {
        let source = format!("try{{await import('{file}')}}catch{{}}finally{{throw 7}}");
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .current_dir(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/fixtures/module-graph"
            ))
            .args(["--module", "--max-steps", "100000", "--eval", &source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with(expected)
        );
    }
}

#[test]
fn script_imports_share_graphs_and_resolve_from_files_arguments_and_stdin() {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/module-graph");
    let source = include_str!("fixtures/module-graph/dynamic-script.js");
    let file = format!("{directory}/dynamic-script.js");
    for (cwd, arguments) in [
        (std::env::temp_dir(), vec![file.as_str()]),
        (std::path::PathBuf::from(directory), vec!["--eval", source]),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .current_dir(cwd)
            .args(arguments)
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{output:?}");
        assert!(output.stdout.is_empty());
        assert_eq!(
            String::from_utf8(output.stderr).unwrap().trim(),
            "uncaught Error: fulfilled 42"
        );
    }
    let mut child = Command::new(env!("CARGO_BIN_EXE_spite"))
        .current_dir(directory)
        .arg("-")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(source.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert_eq!(output.status.code(), Some(1), "{output:?}");
    assert!(output.stdout.is_empty());
    assert_eq!(
        String::from_utf8(output.stderr).unwrap().trim(),
        "uncaught Error: fulfilled 42"
    );
}

#[test]
fn script_imports_preserve_language_rejections_and_host_failures() {
    let directory = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures/module-graph");
    for (source, expected) in [
        (
            "const p=import('./dynamic-errors.mjs');p.constructor={[Symbol.species]:class{constructor(executor){executor(()=>{throw new Error('imports checked')},reason=>{throw reason})}}};p.then(()=>{});42",
            "uncaught Error: imports checked",
        ),
        ("import('./absent.mjs').catch(()=>42);42", "Host"),
        (
            "import('./unsupported.mjs').catch(()=>42);42",
            "Unsupported",
        ),
        ("import('./forever.mjs').catch(()=>42);42", "Limit"),
        ("import('./absent.mjs');throw 7", "uncaught 7"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .current_dir(directory)
            .args(["--max-steps", "100000", "--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1), "{source}: {output:?}");
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with(expected)
        );
    }
}

#[test]
fn pending_imports_keep_the_original_script_completion() {
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .current_dir(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/tests/fixtures/module-graph"
        ))
        .args(["--eval", "import('./pending.mjs');42"])
        .output()
        .unwrap();
    assert!(output.status.success(), "{output:?}");
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "42");
    assert!(output.stderr.is_empty());
}

#[test]
fn error_diagnostics_show_data_without_executing_user_code() {
    insta::allow_duplicates! {
        for flags in [vec![], vec!["--module"]] {
            let output = Command::new(env!("CARGO_BIN_EXE_spite"))
                .args(flags)
                .args(["--eval", "throw new TypeError('failure')"])
                .output()
                .unwrap();
            assert_eq!(output.status.code(), Some(1));
            assert!(output.stdout.is_empty());
            insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap(), @"uncaught TypeError: failure");
        }
    }
    for (source, expected) in [
        (
            "let e=new TypeError('failure');e.toString=()=>{e.message='changed';return 'changed'};throw e",
            "uncaught TypeError: failure",
        ),
        (
            "let e=new TypeError('failure');Object.defineProperty(e,'name',{get(){e.message='changed';return 'Changed'}});throw e",
            "uncaught Error: failure",
        ),
        (
            "let e=new TypeError('failure');Object.defineProperty(e,'message',{get(){e.name='Changed';return 'changed'}});throw e",
            "uncaught TypeError",
        ),
        (
            "throw new Error('line\\nnext')",
            "uncaught Error: line\\nnext",
        ),
        (
            "throw new Error('bad-\\ud800')",
            "uncaught Error: \"bad-\\ud800\"",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--module", "--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert_eq!(String::from_utf8(output.stderr).unwrap().trim(), expected);
    }
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args([
            "--module",
            "--eval",
            "throw new Proxy(new Error('x'),{get(){throw 7},getPrototypeOf(){throw 8}})",
        ])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(
        String::from_utf8(output.stderr)
            .unwrap()
            .starts_with("uncaught Object(")
    );
}

#[test]
fn help_and_usage() {
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .arg("--help")
        .output()
        .unwrap();
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)
            .unwrap()
            .contains("incomplete")
    );
    let output = Command::new(env!("CARGO_BIN_EXE_spite")).output().unwrap();
    assert_eq!(output.status.code(), Some(2));
}

#[test]
fn bigint_invalid_integer_string_diagnostic() {
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--eval", "BigInt('1.5')"])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    insta::assert_snapshot!(String::from_utf8(output.stderr).unwrap());
}

#[test]
fn malformed_host_zone_is_uncatchable_and_utc_invalid_branches_do_not_load_it() {
    let path = std::env::temp_dir().join(format!(
        "spite-invalid-zone-{}-{}.tzif",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut file = std::fs::File::create_new(&path).unwrap();
        file.write_all(b"invalid zone data").unwrap();
    }
    let failed = Command::new(env!("CARGO_BIN_EXE_spite"))
        .env("TZ", &path)
        .args([
            "--eval",
            "try { new Date(0).getHours(); } catch { 41; } finally { throw 42; }",
        ])
        .output()
        .unwrap();
    let independent = Command::new(env!("CARGO_BIN_EXE_spite"))
        .env("TZ", &path)
        .args([
            "--eval",
            "Number.isNaN(new Date(NaN).getHours()) && new Date(0).getUTCHours()===0",
        ])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert_eq!(failed.status.code(), Some(1));
    assert!(failed.stdout.is_empty());
    insta::assert_snapshot!(String::from_utf8(failed.stderr).unwrap());
    assert!(independent.status.success());
    assert_eq!(
        String::from_utf8(independent.stdout).unwrap().trim(),
        "true"
    );
    assert!(independent.stderr.is_empty());
}

#[test]
fn local_getters_honor_explicit_posix_and_empty_utc_host_settings() {
    for (tz, source) in [
        (
            "ABC-2:30",
            "let d=new Date(0);d.getHours()===2 && d.getMinutes()===30 && d.getTimezoneOffset()===-150",
        ),
        ("", "1/new Date(0).getTimezoneOffset()===Infinity"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .env("TZ", tz)
            .args(["--eval", source])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "true");
        assert!(output.stderr.is_empty());
    }
}

#[test]
fn calendar_inputs_preserve_nonfinite_utc_and_date_only_branches_before_zone_loading() {
    let path = std::env::temp_dir().join(format!(
        "spite-invalid-input-zone-{}-{}.tzif",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut file = std::fs::File::create_new(&path).unwrap();
        file.write_all(b"invalid zone data").unwrap();
    }
    let output=Command::new(env!("CARGO_BIN_EXE_spite")).env("TZ",&path)
        .args(["--eval","Number.isNaN(new Date(NaN,0).getTime()) && Number.isNaN(new Date(1970,Infinity).getTime()) && Number.isNaN(Date.parse('invalid')) && Date.parse('1970-01-01')===0 && new Date('1970-01-01T00:00Z').getTime()===0"]).output().unwrap();
    let finite = Command::new(env!("CARGO_BIN_EXE_spite"))
        .env("TZ", &path)
        .args(["--eval", "new Date(1970,0)"])
        .output()
        .unwrap();
    std::fs::remove_file(path).unwrap();
    assert!(output.status.success());
    assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "true");
    assert!(output.stderr.is_empty());
    assert_eq!(finite.status.code(), Some(1));
    assert!(finite.stdout.is_empty());
    assert!(String::from_utf8(finite.stderr).unwrap().contains("Host"));
}

#[test]
fn local_setter_conversions_and_host_lookup_follow_their_specified_order() {
    let path = std::env::temp_dir().join(format!(
        "spite-invalid-setter-zone-{}-{}.tzif",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut file = std::fs::File::create_new(&path).unwrap();
        file.write_all(b"invalid zone data").unwrap();
    }
    let run = |source: &str| {
        Command::new(env!("CARGO_BIN_EXE_spite"))
            .env("TZ", &path)
            .args(["--eval", source])
            .output()
            .unwrap()
    };
    let mut independent = Vec::new();
    for (method, arity) in [
        ("setDate", 1),
        ("setMonth", 2),
        ("setHours", 4),
        ("setMinutes", 3),
        ("setSeconds", 2),
        ("setMilliseconds", 1),
    ] {
        let mut args = vec!["NaN"; arity];
        args[arity - 1] = "{valueOf(){throw 7;}}";
        independent.push(run(&format!(
            "let caught=false;try{{new Date(0).{method}({});}}catch(e){{caught=e===7;}}caught",
            args.join(",")
        )));
        independent.push(run(&format!("let d=new Date(NaN);Number.isNaN(d.{method}({{valueOf(){{d.setTime(9);return 0;}}}})) && d.getTime()===9")));
    }
    for source in [
        "let caught=false;try{new Date(0).setFullYear({valueOf(){throw 7;}},{valueOf(){throw 8;}});}catch(e){caught=e===7;}caught",
        "let caught=false;try{new Date(NaN).setFullYear(NaN,{valueOf(){throw 8;}});}catch(e){caught=e===8;}caught",
        "let d=new Date(NaN);Number.isNaN(d.setFullYear(NaN)) && Number.isNaN(d.getTime())",
    ] {
        independent.push(run(source));
    }
    let mut host_failures = Vec::new();
    for source in [
        "try{new Date(0).setFullYear(NaN,{valueOf(){throw 8;}});}catch{true;}",
        "new Date(NaN).setFullYear(2000)",
        "new Date(0).setHours(NaN)",
        "new Date(0).setMonth(NaN)",
        "new Date(0).setDate(NaN)",
    ] {
        host_failures.push(run(source));
    }
    std::fs::remove_file(path).unwrap();
    for output in independent {
        assert!(output.status.success(), "{:?}", output);
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), "true");
        assert!(output.stderr.is_empty());
    }
    for output in host_failures {
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with("Host")
        );
    }
}

#[test]
fn local_strings_load_host_data_but_invalid_and_saved_offset_strings_remain_independent() {
    let path = std::env::temp_dir().join(format!(
        "spite-invalid-string-zone-{}-{}.tzif",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    {
        let mut file = std::fs::File::create_new(&path).unwrap();
        file.write_all(b"invalid zone data").unwrap();
    }
    let run = |source: &str| {
        Command::new(env!("CARGO_BIN_EXE_spite"))
            .env("TZ", &path)
            .args(["--eval", source])
            .output()
            .unwrap()
    };
    let independent = run(
        "let d=new Date(NaN);d.toString()==='Invalid Date' && d.toDateString()==='Invalid Date' && d.toTimeString()==='Invalid Date' && d.toLocaleString(Symbol(),1n)==='Invalid Date' && d.toLocaleDateString(Symbol(),1n)==='Invalid Date' && d.toLocaleTimeString(Symbol(),1n)==='Invalid Date' && Date.parse('Thu Jan 01 1970 00:00:00 GMT+0000')===0 && Date.parse('Thu Jan 01 1970 00:00:00 GMT+0000 (UTC+00:00:01)')===-1000 && new Date('Fri Dec 31 -0001 19:03:58 GMT-0456 (UTC-04:56:02)').getTime()===Date.parse('0000-01-01')",
    );
    let failures = [
        "new Date(0).toString()",
        "new Date(0).toDateString()",
        "new Date(0).toTimeString()",
        "new Date(0).toLocaleString({get timeZone(){throw 7;}})",
        "new Date(0).toLocaleDateString({get timeZone(){throw 7;}})",
        "new Date(0).toLocaleTimeString({get timeZone(){throw 7;}})",
        "Date({[Symbol.toPrimitive](){throw 7;}})",
    ]
    .map(|source| {
        run(&format!(
            "try{{{source};}}catch{{true;}}finally{{throw 8;}}"
        ))
    });
    std::fs::remove_file(path).unwrap();
    assert!(independent.status.success());
    assert_eq!(
        String::from_utf8(independent.stdout).unwrap().trim(),
        "true"
    );
    assert!(independent.stderr.is_empty());
    for output in failures {
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with("Host")
        );
    }
}

#[test]
fn script_jobs_drain_transitively_and_report_job_failures() {
    for (source, expected) in [
        (
            "Promise.resolve().then(()=>{throw new Error('rejected')});42",
            "42",
        ),
        (
            "Promise.resolve({then(resolve){resolve(1)}}).then(v=>v+1);'done'",
            "\"done\"",
        ),
        ("new Promise(()=>{});42", "42"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--eval", source])
            .output()
            .unwrap();
        assert!(output.status.success());
        assert_eq!(String::from_utf8(output.stdout).unwrap().trim(), expected);
        assert!(output.stderr.is_empty());
    }
    let source = "const chain=Promise.resolve().then(()=>1).then(v=>v+1);chain.constructor={ [Symbol.species]:class{constructor(executor){executor(()=>{throw new TypeError('reaction report')},()=>{})}}};chain.then(v=>v+1);42";
    let output = Command::new(env!("CARGO_BIN_EXE_spite"))
        .args(["--eval", source])
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(1));
    assert!(output.stdout.is_empty());
    insta::assert_snapshot!(
        String::from_utf8(output.stderr).unwrap(),
        @"uncaught TypeError: reaction report"
    );
    for (source, expected) in [
        ("Promise.resolve().then(()=>{while(true){}});42", "Limit"),
        (
            "Promise.resolve().then(()=>{while(true){}});throw 7",
            "uncaught 7",
        ),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--max-steps", "100000", "--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(output.stdout.is_empty());
        assert!(
            String::from_utf8(output.stderr)
                .unwrap()
                .starts_with(expected)
        );
    }
}
