use mysql::Opts;

use crate::status::validate;

pub(crate) fn check_url(
    url: &str,
    checker: fn(Opts) -> Result<(), String>,
) -> Result<String, String> {
    let opts = Opts::from_url(url).map_err(|error| format!("invalid GALERA_URL: {error}"))?;
    let host = opts.get_ip_or_hostname().to_string();
    checker(opts)?;
    Ok(host)
}

pub(crate) fn validate_status(rows: &[(String, String)]) -> Result<(), String> {
    validate(&crate::status::from_rows(rows)?)
}

#[cfg(test)]
mod tests {
    use super::{check_url, validate_status};
    use mysql::Opts;

    fn healthy(_: Opts) -> Result<(), String> {
        Ok(())
    }

    fn failed(_: Opts) -> Result<(), String> {
        Err("transport failed".into())
    }

    #[test]
    fn parses_url_before_transport() {
        assert_eq!(
            check_url("mysql://user:password@example.test:3306", healthy),
            Ok("example.test".into())
        );
    }

    #[test]
    fn returns_transport_errors() {
        assert_eq!(
            check_url("mysql://user:password@example.test:3306", failed),
            Err("transport failed".into())
        );
    }

    #[test]
    fn validates_status_rows() {
        let healthy = vec![
            ("wsrep_local_state_comment".into(), "Synced".into()),
            ("wsrep_ready".into(), "ON".into()),
        ];
        assert!(validate_status(&healthy).is_ok());

        let unhealthy = vec![
            ("wsrep_local_state_comment".into(), "Joining".into()),
            ("wsrep_ready".into(), "ON".into()),
        ];
        assert_eq!(
            validate_status(&unhealthy),
            Err("unhealthy Galera state: state=Joining ready=ON".into())
        );
    }

    #[test]
    fn rejects_duplicate_status_rows() {
        let rows = vec![
            ("wsrep_local_state_comment".into(), "Synced".into()),
            ("wsrep_local_state_comment".into(), "Joining".into()),
            ("wsrep_ready".into(), "ON".into()),
        ];
        assert_eq!(
            validate_status(&rows),
            Err("duplicate Galera status row: wsrep_local_state_comment".into())
        );
    }
}
