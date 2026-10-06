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
fn errors_have_a_failing_exit_status() {
    for (source, expected) in [
        ("missing", "ReferenceError"),
        ("Proxy", "Unsupported"),
        ("1n + 1", "TypeError"),
        ("1n / 0n", "RangeError"),
        ("throw 7", "uncaught 7"),
        ("const x;", "Syntax"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_spite"))
            .args(["--eval", source])
            .output()
            .unwrap();
        assert_eq!(output.status.code(), Some(1));
        assert!(String::from_utf8(output.stderr).unwrap().contains(expected));
        assert!(output.stdout.is_empty());
    }
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
    let mut child = Command::new(env!("CARGO_BIN_EXE_spite"))
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
