mod database;
mod utils;

use crate::database::migration::migrate;
use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(error) = migrate() {
        eprintln!("Failed to initialize the database: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
