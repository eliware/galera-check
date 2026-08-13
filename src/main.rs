use mysql::{prelude::Queryable, Opts, Pool};
use std::{env, process};

fn main() {
    if env::args().nth(1).as_deref() != Some("--check") {
        if env::args().nth(1).is_some() {
            eprintln!("usage: galera-check [--check]");
            process::exit(2);
        }
        return;
    }

    let url = env::var("GALERA_URL").unwrap_or_else(|_| usage("GALERA_URL is required"));
    let opts = Opts::from_url(&url).unwrap_or_else(|e| fail(format!("invalid GALERA_URL: {e}")));
    let host = opts.get_ip_or_hostname().to_string();
    let pool = Pool::new(opts).unwrap_or_else(|e| fail(format!("connection setup failed: {e}")));
    let mut conn = pool.get_conn().unwrap_or_else(|e| fail(format!("connection failed: {e}")));

    let rows: Vec<(String, String)> = conn
        .query("SHOW GLOBAL STATUS WHERE Variable_name IN ('wsrep_local_state_comment','wsrep_ready')")
        .unwrap_or_else(|e| fail(format!("status query failed: {e}")));
    let state = rows.iter().find(|(name, _)| name == "wsrep_local_state_comment").map(|(_, value)| value.as_str()).unwrap_or("");
    let ready = rows.iter().find(|(name, _)| name == "wsrep_ready").map(|(_, value)| value.as_str()).unwrap_or("");
    if state != "Synced" || ready != "ON" {
        fail(format!("unhealthy Galera state: state={state} ready={ready}"));
    }
    println!("{host}: Synced, wsrep_ready=ON");
}

fn usage(message: &str) -> ! {
    eprintln!("{message}; usage: GALERA_URL=mysql://user:password@host:3306 galera-check --check");
    process::exit(2);
}

fn fail(message: String) -> ! {
    eprintln!("{message}");
    process::exit(1);
}
