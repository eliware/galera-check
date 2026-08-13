use std::{env, process};

fn main() {
    let arguments: Vec<String> = env::args().skip(1).collect();
    match galera_check::run(&arguments, env::var("GALERA_URL").ok().as_deref()) {
        Ok(None) => {}
        Ok(Some(message)) => println!("{message}"),
        Err((code, message)) => {
            eprintln!("{message}");
            process::exit(code.into());
        }
    }
}
