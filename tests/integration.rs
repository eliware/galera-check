//! Opt-in integration checks. Set GALERA_CHECK_INTEGRATION=1 and provide the
//! endpoint variables documented below; tests are intentionally skipped otherwise.
use std::{
    env,
    io::{Read, Write},
    net::TcpStream,
    process::Command,
    thread,
    time::Duration,
};

fn endpoint(name: &str) -> Option<String> {
    (env::var("GALERA_CHECK_INTEGRATION").ok().as_deref() == Some("1"))
        .then(|| env::var(name).ok())
        .flatten()
}

fn check(name: &str, expected: i32) {
    let Some(url) = endpoint(name) else { return };
    let status = Command::new(env!("CARGO_BIN_EXE_galera-check"))
        .env("GALERA_URL", url)
        .arg("--check")
        .status()
        .unwrap();
    assert_eq!(status.code(), Some(expected), "integration endpoint {name}");
}

#[test]
fn authentication_rejection() {
    check("GALERA_INTEGRATION_AUTH_URL", 1);
}
#[test]
fn dns_failure() {
    check("GALERA_INTEGRATION_DNS_URL", 1);
}
#[test]
fn tcp_refusal() {
    check("GALERA_INTEGRATION_REFUSED_URL", 1);
}
#[test]
fn tcp_timeout() {
    check("GALERA_INTEGRATION_TIMEOUT_URL", 1);
}

#[test]
fn tls_failure() {
    check("GALERA_INTEGRATION_TLS_URL", 1);
}
#[test]
fn delayed_or_hung_response() {
    check("GALERA_INTEGRATION_HUNG_URL", 1);
}
#[test]
fn non_primary() {
    check("GALERA_INTEGRATION_NON_PRIMARY_URL", 1);
}
#[test]
fn desynchronized() {
    check("GALERA_INTEGRATION_DESYNC_URL", 1);
}
#[test]
fn mariadb_restart() {
    check("GALERA_INTEGRATION_RESTART_URL", 1);
}
#[test]
fn galera_state_transfer() {
    check("GALERA_INTEGRATION_STATE_TRANSFER_URL", 1);
}

#[test]
fn concurrent_clients() {
    let Some(url) = endpoint("GALERA_INTEGRATION_AGENT_URL") else {
        return;
    };
    let listener = "127.0.0.1:33061";
    let mut child = Command::new(env!("CARGO_BIN_EXE_galera-check"))
        .env("GALERA_URL", url)
        .env("GALERA_AGENT_LISTEN", listener)
        .arg("--agent")
        .spawn()
        .unwrap();
    thread::sleep(Duration::from_millis(200));
    let clients: Vec<_> = (0..8)
        .map(|_| {
            thread::spawn(move || {
                let mut stream = TcpStream::connect(listener).unwrap();
                stream
                    .set_read_timeout(Some(Duration::from_secs(10)))
                    .unwrap();
                stream.write_all(b"\n").unwrap();
                let mut response = String::new();
                stream.read_to_string(&mut response).unwrap();
                assert!(response == "up\n" || response == "down\n" || response.starts_with("up "));
            })
        })
        .collect();
    for client in clients {
        client.join().unwrap();
    }
    let _ = child.kill();
    let _ = child.wait();
}
