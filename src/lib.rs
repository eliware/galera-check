use mysql::Opts;

mod mysql_adapter;

#[derive(Debug, PartialEq, Eq)]
pub struct CommandModeError;

/// Runs the checker logic without terminating the process.
pub fn run(arguments: &[String], galera_url: Option<&str>) -> Result<Option<String>, (u8, String)> {
    run_with(arguments, galera_url, check_url)
}

fn run_with(
    arguments: &[String],
    galera_url: Option<&str>,
    checker: fn(&str) -> Result<String, String>,
) -> Result<Option<String>, (u8, String)> {
    match command_mode(arguments) {
        Ok(false) => Ok(None),
        Err(_) => Err((2, "usage: galera-check [--check]".to_string())),
        Ok(true) => {
            let url = galera_url.ok_or_else(|| {
                (
                    2,
                    "GALERA_URL is required; usage: GALERA_URL=mysql://user:password@host:3306 galera-check --check".to_string(),
                )
            })?;
            match checker(url) {
                Ok(host) => Ok(Some(format!("{host}: Synced, wsrep_ready=ON"))),
                Err(message) => Err((1, message)),
            }
        }
    }
}

/// Returns whether the CLI should perform a check for its arguments.
pub fn command_mode(arguments: &[String]) -> Result<bool, CommandModeError> {
    match arguments {
        [] => Ok(false),
        [argument] if argument == "--check" => Ok(true),
        _ => Err(CommandModeError),
    }
}

/// Connects to a MariaDB Galera node and verifies it is ready for traffic.
pub fn check_url(url: &str) -> Result<String, String> {
    check_url_with(url, mysql_adapter::check)
}

fn check_url_with(url: &str, checker: fn(Opts) -> Result<(), String>) -> Result<String, String> {
    let opts = Opts::from_url(url).map_err(|error| format!("invalid GALERA_URL: {error}"))?;
    let host = opts.get_ip_or_hostname().to_string();
    checker(opts)?;
    Ok(host)
}

pub(crate) fn status_from_rows(rows: &[(String, String)]) -> (String, String) {
    let state = rows
        .iter()
        .find(|(name, _)| name == "wsrep_local_state_comment")
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    let ready = rows
        .iter()
        .find(|(name, _)| name == "wsrep_ready")
        .map(|(_, value)| value.clone())
        .unwrap_or_default();
    (state, ready)
}

fn validate_status((state, ready): &(String, String)) -> Result<(), String> {
    if state != "Synced" || ready != "ON" {
        return Err(format!(
            "unhealthy Galera state: state={state} ready={ready}"
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use mysql::Opts;

    use super::{
        check_url, check_url_with, command_mode, run, run_with, status_from_rows, validate_status,
        CommandModeError,
    };

    fn healthy_transport(_: Opts) -> Result<(), String> {
        Ok(())
    }

    fn healthy_url(_: &str) -> Result<String, String> {
        Ok("example.test".to_string())
    }

    fn failed_url(_: &str) -> Result<String, String> {
        Err("unhealthy".to_string())
    }

    #[test]
    fn parses_cli_modes() {
        assert_eq!(command_mode(&[]), Ok(false));
        assert_eq!(command_mode(&["--check".to_string()]), Ok(true));
        assert_eq!(
            command_mode(&["--invalid".to_string()]),
            Err(CommandModeError)
        );
        assert_eq!(
            command_mode(&["--check".to_string(), "unexpected".to_string()]),
            Err(CommandModeError)
        );
    }

    #[test]
    fn rejects_invalid_urls_without_connecting() {
        let error = check_url("not-a-mysql-url").expect_err("invalid URL should fail");
        assert!(error.starts_with("invalid GALERA_URL:"));
    }

    #[test]
    fn reports_connection_failures() {
        let error =
            check_url("mysql://user:password@127.0.0.1:1").expect_err("closed port should fail");
        assert!(error.contains("failed:"));
    }

    #[test]
    fn extracts_required_status_values() {
        let rows = vec![
            ("wsrep_ready".to_string(), "ON".to_string()),
            (
                "wsrep_local_state_comment".to_string(),
                "Synced".to_string(),
            ),
        ];
        assert_eq!(
            status_from_rows(&rows),
            ("Synced".to_string(), "ON".to_string())
        );
    }

    #[test]
    fn accepts_only_synced_ready_status() {
        assert!(validate_status(&("Synced".to_string(), "ON".to_string())).is_ok());
        assert!(validate_status(&("Joining".to_string(), "ON".to_string())).is_err());
        assert!(validate_status(&("Synced".to_string(), "OFF".to_string())).is_err());
        assert!(validate_status(&(String::new(), String::new())).is_err());
    }

    #[test]
    fn runs_noop_without_arguments() {
        assert_eq!(run(&[], None), Ok(None));
    }

    #[test]
    fn rejects_invalid_invocation() {
        assert_eq!(
            run(&["--check".to_string(), "extra".to_string()], None),
            Err((2, "usage: galera-check [--check]".to_string()))
        );
    }

    #[test]
    fn rejects_check_without_url() {
        let error = run(&["--check".to_string()], None).expect_err("URL is required");
        assert_eq!(error.0, 2);
        assert!(error.1.contains("GALERA_URL is required"));
    }

    #[test]
    fn reports_invalid_check_url() {
        let error = run(&["--check".to_string()], Some("not-a-mysql-url"))
            .expect_err("invalid URL should fail");
        assert_eq!(error.0, 1);
        assert!(error.1.starts_with("invalid GALERA_URL:"));
    }

    #[test]
    fn accepts_a_healthy_check_without_a_database() {
        assert_eq!(
            check_url_with("mysql://user:password@example.test:3306", healthy_transport,),
            Ok("example.test".to_string())
        );
    }

    #[test]
    fn formats_a_healthy_cli_result() {
        assert_eq!(
            run_with(
                &["--check".to_string()],
                Some("mysql://user:password@example.test:3306"),
                healthy_url,
            ),
            Ok(Some("example.test: Synced, wsrep_ready=ON".to_string()))
        );
    }

    #[test]
    fn maps_a_failed_cli_check_to_exit_one() {
        assert_eq!(
            run_with(&["--check".to_string()], Some("url"), failed_url),
            Err((1, "unhealthy".to_string()))
        );
    }
}
