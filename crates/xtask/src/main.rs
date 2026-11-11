//! Repository maintenance commands. Native tests run directly through Cargo.

mod commands;
mod fixtures;
mod integrity;
mod policy;
mod scope;
mod snapshots;
mod specification;
mod test_modules;

use std::error::Error;
use std::path::Path;
use std::process::ExitCode;

type Result<T> = std::result::Result<T, Box<dyn Error>>;

fn main() -> ExitCode {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("xtask lives under crates/");
    match commands::run(root, std::env::args().skip(1).collect()) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("{error}");
            ExitCode::FAILURE
        }
    }
}

fn invalid(message: impl Into<String>) -> std::io::Error {
    std::io::Error::other(message.into())
}
