use std::process::Command;

fn binary() -> Command {
    Command::new(env!("CARGO_BIN_EXE_galera-check"))
}

#[test]
fn no_arguments_is_a_successful_noop() {
    let output = binary().output().expect("run checker");
    assert!(output.status.success());
    assert!(output.stdout.is_empty());
    assert!(output.stderr.is_empty());
}

#[test]
fn invalid_arguments_exit_two() {
    let output = binary()
        .arg("--check")
        .arg("unexpected")
        .output()
        .expect("run checker");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("usage:"));
}

#[test]
fn help_is_stable_and_succeeds() {
    let output = binary().arg("--help").output().expect("run checker");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "usage: galera-check [--check | --agent [--performance]]\n"
    );
}

#[test]
fn version_is_stable_and_succeeds() {
    let output = binary().arg("--version").output().expect("run checker");
    assert!(output.status.success());
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "galera-check 0.2.5\n"
    );
}

#[test]
fn missing_url_exits_two() {
    let output = binary()
        .arg("--check")
        .env_remove("GALERA_URL")
        .output()
        .expect("run checker");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("GALERA_URL is required"));
}

#[test]
fn invalid_url_exits_two() {
    let output = binary()
        .arg("--check")
        .env("GALERA_URL", "not-a-mysql-url")
        .output()
        .expect("run checker");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid GALERA_URL:"));
}

#[test]
fn explicit_url_takes_precedence_over_separate_variables() {
    let output = binary()
        .arg("--check")
        .env("GALERA_URL", "not-a-mysql-url")
        .env("GALERA_USER", "fallback-user")
        .env("GALERA_PASSWORD", "fallback-password")
        .env("GALERA_HOST", "127.0.0.1")
        .output()
        .expect("run checker");
    assert_eq!(output.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&output.stderr).contains("invalid GALERA_URL:"));
}

#[test]
fn healthy_url_succeeds_when_supplied() {
    if std::env::var("GALERA_CHECK_LIVE").ok().as_deref() != Some("1") {
        return;
    }
    let Ok(url) = std::env::var("GALERA_URL") else {
        return;
    };
    let output = binary()
        .arg("--check")
        .env("GALERA_URL", url)
        .output()
        .expect("run checker");
    assert!(output.status.success());
    assert!(String::from_utf8_lossy(&output.stdout).contains("Synced, wsrep_ready=ON"));
}
