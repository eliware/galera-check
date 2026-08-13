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
    match galera_check::check_url(&url) {
        Ok(host) => println!("{host}: Synced, wsrep_ready=ON"),
        Err(message) => fail(message),
    }
}

fn usage(message: &str) -> ! {
    eprintln!("{message}; usage: GALERA_URL=mysql://user:password@host:3306 galera-check --check");
    process::exit(2);
}

fn fail(message: String) -> ! {
    eprintln!("{message}");
    process::exit(1);
}
