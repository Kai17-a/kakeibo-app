mod database;
use crate::database::database::migrate;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(error) = migrate() {
        eprintln!("Failed to initialize the database: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
