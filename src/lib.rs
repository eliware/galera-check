mod agent;
mod checker;
mod cli;
mod mysql_adapter;
mod status;

pub use cli::{CommandMode, CommandModeError};

/// Runs the HAProxy agent listener until the process is stopped.
pub fn run_agent(galera_url: &str, listen: &str) -> Result<(), String> {
    agent::serve(galera_url, listen, check_url)
}

/// Runs the checker logic without terminating the process.
pub fn run(arguments: &[String], galera_url: Option<&str>) -> Result<Option<String>, (u8, String)> {
    cli::run(arguments, galera_url, check_url)
}

/// Returns whether the CLI should perform a check for its arguments.
pub fn command_mode(arguments: &[String]) -> Result<CommandMode, CommandModeError> {
    cli::command_mode(arguments)
}

/// Connects to a MariaDB Galera node and verifies it is ready for traffic.
pub fn check_url(url: &str) -> Result<String, String> {
    checker::check_url(url, mysql_adapter::check)
}

#[cfg(test)]
mod tests {
    use super::{check_url, command_mode, run, run_agent, CommandMode, CommandModeError};

    #[test]
    fn parses_cli_modes() {
        assert_eq!(command_mode(&[]), Ok(CommandMode::Noop));
        assert_eq!(
            command_mode(&["--check".to_string()]),
            Ok(CommandMode::Check)
        );
        assert_eq!(
            command_mode(&["--agent".to_string()]),
            Ok(CommandMode::Agent)
        );
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

    #[test]
    fn reports_agent_bind_failures() {
        let error = run_agent("mysql://user:password@host", "bad address")
            .expect_err("invalid listen address should fail");
        assert!(error.starts_with("agent bind failed on bad address:"));
    }
}
