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
