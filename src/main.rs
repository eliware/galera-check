use std::{env, process};

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let galera_url = connection_url();
    if (arguments.len() == 1 && arguments[0] == "--agent")
        || (arguments.len() == 2 && arguments[0] == "--agent" && arguments[1] == "--performance")
    {
        let url = galera_url.unwrap_or_else(|| {
            usage("GALERA_URL or GALERA_USER/GALERA_PASSWORD/GALERA_HOST is required for --agent")
        });
        let listen = env::var("GALERA_AGENT_LISTEN").unwrap_or_else(|_| default_agent_listen(&url));
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
            eprintln!("{message}");
            process::exit(code.into());
        }
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
            Some(format!("mysql://{user}:{password}@{host}:{port}"))
        }
    }
}

fn default_agent_listen(url: &str) -> String {
    let port = url
        .rsplit('@')
        .next()
        .and_then(|host| host.split(':').next())
        .and_then(|host| host.rsplit('.').next())
        .and_then(|octet| octet.parse::<u16>().ok())
        .filter(|octet| (81..=83).contains(octet))
        .map(|octet| 33060 + octet - 80)
        .unwrap_or(33060);
    format!("127.0.0.1:{port}")
}
