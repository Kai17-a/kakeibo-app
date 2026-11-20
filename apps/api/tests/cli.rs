use clap::Parser;
use kakeibo_app::cli::Cli;

#[test]
fn uses_default_port() {
    let cli = Cli::try_parse_from(["kakeibo-app"]).unwrap();

    assert_eq!(cli.port, 8000);
}

#[test]
fn accepts_port_option() {
    let cli = Cli::try_parse_from(["kakeibo-app", "--port", "8080"]).unwrap();

    assert_eq!(cli.port, 8080);
}

#[test]
fn runs_the_server_without_a_subcommand() {
    let cli = Cli::try_parse_from(["kakeibo-app"]).unwrap();

    assert!(cli.command.is_none());
}

#[test]
fn parses_auth_user_commands() {
    use kakeibo_app::cli::{AuthCommand, Command};

    let cases = [
        (
            vec!["kakeibo-app", "auth", "add-user", "kaito"],
            AuthCommand::AddUser {
                username: "kaito".to_owned(),
                password_stdin: false,
            },
        ),
        (
            vec![
                "kakeibo-app",
                "auth",
                "add-user",
                "kaito",
                "--password-stdin",
            ],
            AuthCommand::AddUser {
                username: "kaito".to_owned(),
                password_stdin: true,
            },
        ),
        (
            vec!["kakeibo-app", "auth", "set-password", "kaito"],
            AuthCommand::SetPassword {
                username: "kaito".to_owned(),
                password_stdin: false,
            },
        ),
        (
            vec!["kakeibo-app", "auth", "list-users"],
            AuthCommand::ListUsers,
        ),
        (
            vec!["kakeibo-app", "auth", "remove-user", "kaito"],
            AuthCommand::RemoveUser {
                username: "kaito".to_owned(),
            },
        ),
    ];

    for (args, expected) in cases {
        let cli = Cli::try_parse_from(&args).unwrap();
        assert_eq!(cli.command, Some(Command::Auth(expected)), "{args:?}");
    }
}

#[test]
fn add_user_requires_a_username() {
    assert!(Cli::try_parse_from(["kakeibo-app", "auth", "add-user"]).is_err());
}
