//! The `kakeibo-app auth ...` account commands.

use super::{config, database};
use crate::{cli::AuthCommand, repository::auth::AuthRepository, service::auth::AuthService};
use std::{
    io::{self, BufRead},
    process::ExitCode,
};

pub async fn run(command: AuthCommand) -> ExitCode {
    match execute(command).await {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("error: {message}");
            ExitCode::FAILURE
        }
    }
}

async fn execute(command: AuthCommand) -> Result<(), String> {
    let lookup = |key: &str| std::env::var(key).ok();
    let path = config::database_path(lookup);
    let min_password_length =
        config::min_password_length(lookup).map_err(|error| error.to_string())?;
    let pool = database::connect(&path)
        .await
        .map_err(|error| error.to_string())?;
    // The session lifetime does not matter for account management.
    let service = AuthService::new(AuthRepository::new(pool), 0)
        .with_min_password_length(min_password_length);

    match command {
        AuthCommand::AddUser {
            username,
            password_stdin,
        } => {
            let password = read_password(password_stdin)?;
            service
                .add_user(&username, &password)
                .await
                .map_err(|error| error.to_string())?;
            println!("Added user '{}' to {}", username.trim(), path.display());
        }
        AuthCommand::SetPassword {
            username,
            password_stdin,
        } => {
            let password = read_password(password_stdin)?;
            service
                .set_password(&username, &password)
                .await
                .map_err(|error| error.to_string())?;
            println!(
                "Changed the password of '{}' and signed them out",
                username.trim()
            );
        }
        AuthCommand::ListUsers => {
            for username in service
                .list_usernames()
                .await
                .map_err(|error| error.to_string())?
            {
                println!("{username}");
            }
        }
        AuthCommand::RemoveUser { username } => {
            service
                .remove_user(&username)
                .await
                .map_err(|error| error.to_string())?;
            println!("Removed user '{}'", username.trim());
        }
    }
    Ok(())
}

fn read_password(from_stdin: bool) -> Result<String, String> {
    if from_stdin {
        let mut line = String::new();
        io::stdin()
            .lock()
            .read_line(&mut line)
            .map_err(|error| format!("failed to read the password: {error}"))?;
        return Ok(line.trim_end_matches(['\r', '\n']).to_owned());
    }

    let password = rpassword::prompt_password("Password: ")
        .map_err(|error| format!("failed to read the password: {error}"))?;
    let confirmation = rpassword::prompt_password("Confirm password: ")
        .map_err(|error| format!("failed to read the password: {error}"))?;
    if password != confirmation {
        return Err("passwords do not match".to_owned());
    }
    Ok(password)
}
