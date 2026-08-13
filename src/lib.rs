mod checker;
mod cli;
mod mysql_adapter;
mod status;

pub use cli::CommandModeError;

/// Runs the checker logic without terminating the process.
pub fn run(arguments: &[String], galera_url: Option<&str>) -> Result<Option<String>, (u8, String)> {
    cli::run(arguments, galera_url, check_url)
}

/// Returns whether the CLI should perform a check for its arguments.
pub fn command_mode(arguments: &[String]) -> Result<bool, CommandModeError> {
    cli::command_mode(arguments)
}

/// Connects to a MariaDB Galera node and verifies it is ready for traffic.
pub fn check_url(url: &str) -> Result<String, String> {
    checker::check_url(url, mysql_adapter::check)
}

#[cfg(test)]
mod tests {
    use super::{check_url, command_mode, run, CommandModeError};

    #[test]
    fn parses_cli_modes() {
        assert_eq!(command_mode(&[]), Ok(false));
        assert_eq!(command_mode(&["--check".to_string()]), Ok(true));
        assert_eq!(
            command_mode(&["--invalid".to_string()]),
            Err(CommandModeError)
        );
    }

    #[test]
    fn rejects_invalid_urls_without_connecting() {
        let error = check_url("not-a-mysql-url").expect_err("invalid URL should fail");
        assert!(error.starts_with("invalid GALERA_URL:"));
    }

    #[test]
    fn runs_noop_without_arguments() {
        assert_eq!(run(&[], None), Ok(None));
    }
}
