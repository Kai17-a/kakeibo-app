use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(version, about)]
pub struct Cli {
    #[arg(long, default_value_t = 8000)]
    pub port: u16,

    /// Runs a maintenance command instead of the server.
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum Command {
    /// Manages accounts for KAKEIBO_AUTH_MODE=local.
    #[command(subcommand)]
    Auth(AuthCommand),
}

#[derive(Debug, PartialEq, Eq, Subcommand)]
pub enum AuthCommand {
    /// Registers a user. The password is prompted for twice.
    AddUser {
        username: String,
        /// Reads the password from the first line of standard input instead of prompting.
        #[arg(long)]
        password_stdin: bool,
    },
    /// Replaces a user's password and signs them out of every session.
    SetPassword {
        username: String,
        /// Reads the password from the first line of standard input instead of prompting.
        #[arg(long)]
        password_stdin: bool,
    },
    /// Prints every registered username.
    ListUsers,
    /// Deletes a user and their sessions.
    RemoveUser { username: String },
}
