use std::{env, process};

fn main() {
    match galera_check::command_mode(env::args().nth(1).as_deref()) {
        Ok(false) => return,
        Err(()) => {
            eprintln!("usage: galera-check [--check]");
            process::exit(2);
        }
        Ok(true) => {}
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
