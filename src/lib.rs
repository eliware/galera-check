use mysql::{prelude::Queryable, Opts, Pool};

/// Connects to a MariaDB Galera node and verifies it is ready for traffic.
pub fn check_url(url: &str) -> Result<String, String> {
    let opts = Opts::from_url(url).map_err(|error| format!("invalid GALERA_URL: {error}"))?;
    let host = opts.get_ip_or_hostname().to_string();
    let pool = Pool::new(opts).map_err(|error| format!("connection setup failed: {error}"))?;
    let mut connection = pool
        .get_conn()
        .map_err(|error| format!("connection failed: {error}"))?;
    let rows: Vec<(String, String)> = connection
        .query("SHOW GLOBAL STATUS WHERE Variable_name IN ('wsrep_local_state_comment','wsrep_ready')")
        .map_err(|error| format!("status query failed: {error}"))?;
    let state = rows
        .iter()
        .find(|(name, _)| name == "wsrep_local_state_comment")
        .map(|(_, value)| value.as_str())
        .unwrap_or("");
    let ready = rows
        .iter()
        .find(|(name, _)| name == "wsrep_ready")
        .map(|(_, value)| value.as_str())
        .unwrap_or("");
    if state != "Synced" || ready != "ON" {
        return Err(format!("unhealthy Galera state: state={state} ready={ready}"));
    }
    Ok(host)
}

#[cfg(test)]
mod tests {
    use super::check_url;

    #[test]
    fn rejects_invalid_urls_without_connecting() {
        let error = check_url("not-a-mysql-url").expect_err("invalid URL should fail");
        assert!(error.starts_with("invalid GALERA_URL:"));
    }
}
