use std::{
    io::Write,
    net::{TcpListener, TcpStream},
};

pub fn serve(
    url: &str,
    listen: &str,
    checker: fn(&str) -> Result<String, String>,
) -> Result<(), String> {
    let listener = TcpListener::bind(listen)
        .map_err(|error| format!("agent bind failed on {listen}: {error}"))?;
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => respond(&mut stream, checker(url)),
            Err(error) => eprintln!("agent accept failed: {error}"),
        }
    }
    Ok(())
}

fn respond(stream: &mut TcpStream, result: Result<String, String>) {
    let response = if result.is_ok() { "up\n" } else { "down\n" };
    let _ = stream.write_all(response.as_bytes());
}
