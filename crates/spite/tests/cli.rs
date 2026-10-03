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
fn errors_have_a_failing_exit_status() {
    for (source, expected) in [
        ("missing", "ReferenceError"),
        ("1n", "Unsupported"),
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
