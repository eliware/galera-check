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
    match galera_check::run(&arguments, galera_url.as_deref()) {
        Ok(None) => {}
        Ok(Some(message)) => println!("{message}"),
        Err((code, message)) => {
            eprintln!("{message}");
            process::exit(code.into());
        }
    }
}
