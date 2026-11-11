//! Command dispatch and subprocess execution.

use crate::Result;
use crate::fixtures;
use crate::invalid;
use crate::policy;
use crate::specification;
use crate::test_modules;
use std::env;
use std::ffi::OsString;
use std::path::Path;
use std::process::Command;

const GENERATORS: &[(&str, &str)] = &[
    ("unicode", "unicode.py"),
    ("case-mappings", "case_mappings.py"),
    ("normalization", "normalization.py"),
    ("regexp-properties", "regexp_properties.py"),
    ("number-radix", "number_radix.py"),
    ("number-sum", "number_sum.py"),
];

pub(crate) fn run(root: &Path, args: Vec<String>) -> Result<()> {
    let command = args.first().map(String::as_str).unwrap_or("help");
    match command {
        "check" if args.len() == 1 => {
            cargo(root, &["fmt", "--all", "--", "--check"])?;
            cargo(
                root,
                &[
                    "clippy",
                    "--workspace",
                    "--all-targets",
                    "--locked",
                    "--",
                    "-D",
                    "warnings",
                ],
            )?;
            policy::check(root)?;
            fixtures::check(&root.join("tests/test262"))?;
            test_modules::update(root, true)?;
            specification::check(root, None)?;
            generate(root, "all", &["--check".to_owned()])
        }
        "verify" if args.len() == 1 => {
            policy::check(root)?;
            test_modules::update(root, true)?;
            fixtures::check(&root.join("tests/test262"))
        }
        "verify-spec" if args.len() <= 2 => specification::check(root, args.get(1).map(Path::new)),
        "test262" if args.len() == 1 => cargo(
            root,
            &[
                "test",
                "--package",
                "spite-test262",
                "--profile",
                "conformance",
                "--test",
                "conformance",
                "--locked",
            ],
        ),
        "generate" if args.len() >= 2 => generate(root, &args[1], &args[2..]),
        "help" | "--help" | "-h" => {
            println!(
                "Repository maintenance:\n\
                      cargo xtask check                 Format, lint and verify repository data\n\
                      cargo xtask verify                Verify dependencies and Test262 provenance\n\
                      cargo xtask verify-spec [FILE]     Verify a local or downloaded specification\n\
                      cargo xtask test262               Run the optimized conformance target\n\
                      cargo xtask generate NAME [ARGS]   Run a documented data generator\n\
                      \nRun native tests with cargo test --workspace."
            );
            Ok(())
        }
        _ => Err(invalid("unknown command; use cargo xtask --help").into()),
    }
}

fn cargo(root: &Path, args: &[&str]) -> Result<()> {
    let cargo = env::var_os("CARGO").unwrap_or_else(|| OsString::from("cargo"));
    execute(Command::new(cargo).current_dir(root).args(args))
}

pub(crate) fn execute(command: &mut Command) -> Result<()> {
    let status = command.status()?;
    if !status.success() {
        return Err(invalid(format!("command failed ({status}): {command:?}")).into());
    }
    Ok(())
}

fn generate(root: &Path, name: &str, args: &[String]) -> Result<()> {
    if name == "test-modules" {
        if !args.is_empty() && args != ["--check"] {
            return Err(invalid("test-modules accepts only --check").into());
        }
        return test_modules::update(root, !args.is_empty());
    }
    if name == "all" {
        for (name, _) in GENERATORS {
            generate(root, name, args)?;
        }
        return Ok(());
    }
    let script = if name == "regexp-results" {
        "regexp_results.py"
    } else {
        GENERATORS
            .iter()
            .find_map(|(key, script)| (*key == name).then_some(*script))
            .ok_or_else(|| invalid(format!("unknown generator: {name}")))?
    };
    let python = python()?;
    execute(
        Command::new(python)
            .current_dir(root)
            .arg(root.join("crates/xtask/codegen").join(script))
            .args(args),
    )
}

fn python() -> Result<OsString> {
    if let Some(python) = env::var_os("PYTHON") {
        return Ok(python);
    }
    for name in ["python3", "python"] {
        if Command::new(name)
            .arg("--version")
            .output()
            .is_ok_and(|output| output.status.success())
        {
            return Ok(name.into());
        }
    }
    Err(invalid("data generation requires Python 3; set PYTHON to its executable").into())
}
