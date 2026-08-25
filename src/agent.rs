use std::{
    io::Write,
    net::{TcpListener, TcpStream},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{sync_channel, TrySendError},
        Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

const WORKERS: usize = 4;
const QUEUE: usize = 16;
const REQUEST_DEADLINE: Duration = Duration::from_secs(5);

pub fn serve(
    url: &str,
    listen: &str,
    checker: fn(&str) -> Result<String, String>,
) -> Result<(), String> {
    serve_with_shutdown(url, listen, checker, Arc::new(AtomicBool::new(false)))
}

pub fn serve_with_shutdown(
    url: &str,
    listen: &str,
    checker: fn(&str) -> Result<String, String>,
    shutdown: Arc<AtomicBool>,
) -> Result<(), String> {
    let listener = TcpListener::bind(listen)
        .map_err(|error| format!("agent bind failed on {listen}: {error}"))?;
    listener
        .set_nonblocking(true)
        .map_err(|error| format!("agent setup failed: {error}"))?;
    let (sender, receiver) = sync_channel::<TcpStream>(QUEUE);
    let receiver = Arc::new(Mutex::new(receiver));
    let mut workers = Vec::new();
    for _ in 0..WORKERS {
        let receiver = Arc::clone(&receiver);
        let target = url.to_owned();
        workers.push(thread::spawn(move || loop {
            let Ok(mut stream) = receiver.lock().unwrap().recv() else {
                break;
            };
            let started = Instant::now();
            let result = checker(&target);
            let result = if started.elapsed() > REQUEST_DEADLINE {
                Err("agent request deadline exceeded".into())
            } else {
                result
            };
            respond(&mut stream, result);
        }));
    }
    while !shutdown.load(Ordering::Relaxed) {
        match listener.accept() {
            Ok((stream, _)) => match sender.try_send(stream) {
                Ok(()) => {}
                Err(TrySendError::Full(mut stream)) => {
                    respond(&mut stream, Err("agent queue full".into()))
                }
                Err(TrySendError::Disconnected(_)) => break,
            },
            Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                thread::sleep(Duration::from_millis(25))
            }
            Err(error) => eprintln!("agent accept failed: {error}"),
        }
    }
    drop(sender);
    for worker in workers {
        let _ = worker.join();
    }
    Ok(())
}

fn respond(stream: &mut TcpStream, result: Result<String, String>) {
    let _ = stream.set_write_timeout(Some(REQUEST_DEADLINE));
    if let Err(ref message) = result {
        emit_diagnostic(message);
    }
    let response = match result {
        Ok(response) if valid_response(&response) => format!("{response}\n"),
        _ => "down\n".into(),
    };
    let _ = stream.write_all(response.as_bytes());
}

fn valid_response(response: &str) -> bool {
    if response == "up" {
        return true;
    }
    let Some(value) = response
        .strip_prefix("up ")
        .and_then(|v| v.strip_suffix('%'))
    else {
        return false;
    };
    matches!(value.parse::<u8>(), Ok(1..=100))
}

fn emit_diagnostic(message: &str) {
    let mode = std::env::var("GALERA_DIAGNOSTICS").ok();
    if mode.as_deref() != Some("1") && mode.as_deref() != Some("json") {
        return;
    }
    let reason = if message.contains("invalid GALERA_URL") {
        "config"
    } else if message.contains("status query") {
        "query"
    } else if message.contains("unhealthy Galera") {
        "state"
    } else if message.contains("weight") || message.contains("traffic") {
        "performance"
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
