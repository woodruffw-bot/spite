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
