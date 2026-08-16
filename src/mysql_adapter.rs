use std::{fmt::Display, time::Duration};

use mysql::{prelude::Queryable, Opts, OptsBuilder, Pool};

use crate::checker::validate_status;

pub fn check(opts: Opts) -> Result<(), String> {
    let pool = Pool::new(with_default_timeouts(opts))
        .map_err(|error| format!("connection setup failed: {error}"))?;
    let mut connection = pool.get_conn().map_err(connection_error)?;
    let rows: Vec<(String, String)> = connection
        .query(
            "SHOW GLOBAL STATUS WHERE Variable_name IN ('wsrep_local_state_comment','wsrep_ready')",
        )
        .map_err(status_query_error)?;
    validate_status(&rows)
}

pub fn weight(opts: Opts) -> Result<String, String> {
    let started = std::time::Instant::now();
    let pool = Pool::new(with_default_timeouts(opts))
        .map_err(|error| format!("connection setup failed: {error}"))?;
    let mut connection = pool.get_conn().map_err(connection_error)?;
    let rows: Vec<(String, String)> = connection.query(
        "SHOW GLOBAL STATUS WHERE Variable_name IN ('wsrep_local_state_comment','wsrep_ready','wsrep_local_recv_queue','wsrep_local_send_queue','wsrep_flow_control_paused')",
    ).map_err(status_query_error)?;
    let latency_ms = started.elapsed().as_millis() as u64;
    format_weight(crate::weight::calculate(&rows, latency_ms))
}

fn format_weight(weight: u8) -> Result<String, String> {
    if weight == 0 {
        return Err("Galera node is not safe for read traffic".into());
    }
    Ok(format!("up {weight}%"))
}

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(5);

fn with_default_timeouts(opts: Opts) -> Opts {
    OptsBuilder::from_opts(opts.clone())
        .tcp_connect_timeout(opts.get_tcp_connect_timeout().or(Some(DEFAULT_TIMEOUT)))
        .read_timeout(opts.get_read_timeout().copied().or(Some(DEFAULT_TIMEOUT)))
        .write_timeout(opts.get_write_timeout().copied().or(Some(DEFAULT_TIMEOUT)))
        .into()
}

fn connection_error<E: Display>(error: E) -> String {
    format!("connection failed: {error}")
}

fn status_query_error<E: Display>(error: E) -> String {
    format!("status query failed: {error}")
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use mysql::{Opts, OptsBuilder};

    use super::{format_weight, with_default_timeouts, DEFAULT_TIMEOUT};

    #[test]
    fn performance_agent_response_has_explicit_up_state() {
        assert_eq!(format_weight(98), Ok("up 98%".into()));
        assert_eq!(
            format_weight(0),
            Err("Galera node is not safe for read traffic".into())
        );
    }

    #[test]
    fn adds_defaults_and_preserves_explicit_timeouts() {
        let defaults = with_default_timeouts(Opts::from_url("mysql://u:p@host").unwrap());
        assert_eq!(defaults.get_tcp_connect_timeout(), Some(DEFAULT_TIMEOUT));
        assert_eq!(defaults.get_read_timeout(), Some(&DEFAULT_TIMEOUT));
        assert_eq!(defaults.get_write_timeout(), Some(&DEFAULT_TIMEOUT));

        let configured: Opts = OptsBuilder::from_opts(Opts::from_url("mysql://u:p@host").unwrap())
            .tcp_connect_timeout(Some(Duration::from_secs(1)))
            .read_timeout(Some(Duration::from_secs(2)))
            .write_timeout(Some(Duration::from_secs(3)))
            .into();
        let configured = with_default_timeouts(configured);
        assert_eq!(
            configured.get_tcp_connect_timeout(),
            Some(Duration::from_secs(1))
        );
        assert_eq!(configured.get_read_timeout(), Some(&Duration::from_secs(2)));
        assert_eq!(
            configured.get_write_timeout(),
            Some(&Duration::from_secs(3))
        );
    }
}
