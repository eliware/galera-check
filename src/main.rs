use std::{env, process};

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    let galera_url = match env::var("GALERA_URL") {
        Ok(url) => Some(url),
        Err(env::VarError::NotPresent) => None,
        Err(env::VarError::NotUnicode(_)) => {
            eprintln!("GALERA_URL is not valid UTF-8");
            process::exit(2);
        }
    };
    if arguments.len() == 1 && arguments[0] == "--agent" {
        let url = galera_url.unwrap_or_else(|| usage("GALERA_URL is required for --agent"));
        let listen =
            env::var("GALERA_AGENT_LISTEN").unwrap_or_else(|_| "127.0.0.1:33060".to_string());
        if let Err(message) = galera_check::run_agent(&url, &listen) {
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
    eprintln!("{message}; usage: GALERA_URL=mysql://user:password@host:3306 galera-check --agent");
    process::exit(2);
}
