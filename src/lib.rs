use mysql::{prelude::Queryable, Opts, Pool};

#[derive(Debug, PartialEq, Eq)]
pub struct CommandModeError;

/// Returns whether the CLI should perform a check for its first argument.
pub fn command_mode(argument: Option<&str>) -> Result<bool, CommandModeError> {
    match argument {
        None => Ok(false),
        Some("--check") => Ok(true),
        Some(_) => Err(CommandModeError),
    }
}

/// Connects to a MariaDB Galera node and verifies it is ready for traffic.
pub fn check_url(url: &str) -> Result<String, String> {
    let opts = Opts::from_url(url).map_err(|error| format!("invalid GALERA_URL: {error}"))?;
    let host = opts.get_ip_or_hostname().to_string();
    let pool = Pool::new(opts).map_err(|error| format!("connection setup failed: {error}"))?;
    let mut connection = pool
        .get_conn()
        .map_err(|error| format!("connection failed: {error}"))?;
    let rows: Vec<(String, String)> = connection
        .query(
            "SHOW GLOBAL STATUS WHERE Variable_name IN ('wsrep_local_state_comment','wsrep_ready')",
        )
        .map_err(|error| format!("status query failed: {error}"))?;
    let status = status_from_rows(&rows);
    validate_status(&status)?;
    Ok(host)
}

fn status_from_rows(rows: &[(String, String)]) -> (String, String) {
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
    use super::{check_url, command_mode, status_from_rows, validate_status, CommandModeError};

    #[test]
    fn parses_cli_modes() {
        assert_eq!(command_mode(None), Ok(false));
        assert_eq!(command_mode(Some("--check")), Ok(true));
        assert_eq!(command_mode(Some("--invalid")), Err(CommandModeError));
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
}
