mod database;

use std::process::ExitCode;

fn main() -> ExitCode {
    if let Err(error) = database::migrate() {
        eprintln!("Failed to initialize the database: {error}");
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}
