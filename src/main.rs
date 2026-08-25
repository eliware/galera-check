use std::{env, process};

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    if arguments == ["--help"] || arguments == ["-h"] {
        println!("usage: galera-check [--check | --agent [--performance]]");
        return;
    }
    if arguments == ["--version"] || arguments == ["-V"] {
        println!("galera-check {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let galera_url = connection_url();
    if (arguments.len() == 1 && arguments[0] == "--agent")
        || (arguments.len() == 2 && arguments[0] == "--agent" && arguments[1] == "--performance")
    {
        let url = galera_url.unwrap_or_else(|| {
            usage("GALERA_URL or GALERA_USER/GALERA_PASSWORD/GALERA_HOST is required for --agent")
        });
        let listen = env::var("GALERA_AGENT_LISTEN")
            .unwrap_or_else(|_| default_agent_listen(&url, arguments.len() == 2));
        let result = if arguments.len() == 2 {
            galera_check::run_performance_agent(&url, &listen)
        } else {
            galera_check::run_agent(&url, &listen)
        };
        if let Err(message) = result {
            eprintln!("{message}");
            process::exit(1);
        }
        return;
    }
    match galera_check::run(&arguments, galera_url.as_deref()) {
        Ok(None) => {}
        Ok(Some(message)) => println!("{message}"),
        Err((code, message)) => {
            emit_diagnostic(&message);
            eprintln!("{message}");
            process::exit(code.into());
        }
    }
}

fn emit_diagnostic(message: &str) {
    let mode = env::var("GALERA_DIAGNOSTICS").ok();
    if !matches!(mode.as_deref(), Some("1") | Some("json")) {
        return;
    }
    let reason = if message.contains("GALERA_URL") || message.contains("configuration") {
        "config"
    } else if message.contains("query") {
        "query"
    } else if message.contains("unhealthy") {
        "state"
    } else if message.contains("access denied") || message.contains("1045") {
        "auth"
    } else {
        "connect"
    };
    if mode.as_deref() == Some("json") {
        eprintln!(r#"{{"reason":"{reason}"}}"#);
    } else {
        eprintln!("galera-check [{reason}]");
    }
}

fn usage(message: &str) -> ! {
    eprintln!("{message}; usage: GALERA_URL=mysql://user:password@host:3306 galera-check --agent [--performance]");
    process::exit(2);
}

fn connection_url() -> Option<String> {
    match env::var("GALERA_URL") {
        Ok(url) => Some(url),
        Err(env::VarError::NotUnicode(_)) => usage("GALERA_URL is not valid UTF-8"),
        Err(env::VarError::NotPresent) => {
            let user = env::var("GALERA_USER").ok()?;
            let password = env::var("GALERA_PASSWORD").ok()?;
            let host = env::var("GALERA_HOST").ok()?;
            let port = env::var("GALERA_PORT").unwrap_or_else(|_| "3306".into());
            if port.parse::<u16>().is_err() {
                usage("GALERA_PORT must be a valid port");
            }
            let host = if host.contains(':') && !host.starts_with('[') {
                format!("[{host}]")
            } else {
                host
            };
            Some(format!(
                "mysql://{}:{}@{}:{port}",
                encode_component(&user),
                encode_component(&password),
                host
            ))
        }
    }
}

fn encode_component(value: &str) -> String {
    value
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char]
            }
            byte => format!("%{byte:02X}").chars().collect(),
        })
        .collect()
}

fn default_agent_listen(url: &str, performance: bool) -> String {
    let port = url
        .rsplit('@')
        .next()
        .and_then(|host| host.split(':').next())
        .and_then(|host| host.rsplit('.').next())
        .and_then(|octet| octet.parse::<u16>().ok())
        .filter(|octet| (81..=83).contains(octet))
        .map(|octet| {
            if performance {
                33160 + octet - 80
            } else {
                33060 + octet - 80
            }
        })
        .unwrap_or(33060);
    format!("127.0.0.1:{port}")
}
