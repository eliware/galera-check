use std::fmt::Display;

use mysql::{prelude::Queryable, Opts, Pool};

use crate::checker::validate_status;

pub fn check(opts: Opts) -> Result<(), String> {
    let pool = Pool::new(opts).map_err(|error| format!("connection setup failed: {error}"))?;
    let mut connection = pool.get_conn().map_err(connection_error)?;
    let rows: Vec<(String, String)> = connection
        .query(
            "SHOW GLOBAL STATUS WHERE Variable_name IN ('wsrep_local_state_comment','wsrep_ready')",
        )
        .map_err(status_query_error)?;
    validate_status(&rows)
}

fn connection_error<E: Display>(error: E) -> String {
    format!("connection failed: {error}")
}

fn status_query_error<E: Display>(error: E) -> String {
    format!("status query failed: {error}")
}
