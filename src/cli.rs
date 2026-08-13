#[derive(Debug, PartialEq, Eq)]
pub struct CommandModeError;

pub(crate) fn command_mode(arguments: &[String]) -> Result<bool, CommandModeError> {
    match arguments {
        [] => Ok(false),
        [argument] if argument == "--check" => Ok(true),
        _ => Err(CommandModeError),
    }
}

pub(crate) fn run(
    arguments: &[String],
    galera_url: Option<&str>,
    checker: fn(&str) -> Result<String, String>,
) -> Result<Option<String>, (u8, String)> {
    match command_mode(arguments) {
        Ok(false) => Ok(None),
        Err(_) => Err((2, "usage: galera-check [--check]".into())),
        Ok(true) => {
            let url = galera_url.ok_or_else(|| (2, "GALERA_URL is required; usage: GALERA_URL=mysql://user:password@host:3306 galera-check --check".into()))?;
            checker(url)
                .map(|host| Some(format!("{host}: Synced, wsrep_ready=ON")))
                .map_err(|message| {
                    let code = if message.starts_with("invalid GALERA_URL:") {
                        2
                    } else {
                        1
                    };
                    (code, message)
                })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::run;

    fn healthy(_: &str) -> Result<String, String> {
        Ok("example.test".into())
    }

    fn failed(_: &str) -> Result<String, String> {
        Err("unhealthy".into())
    }

    fn invalid_url(_: &str) -> Result<String, String> {
        Err("invalid GALERA_URL: invalid URL".into())
    }

    #[test]
    fn rejects_invalid_invocation() {
        assert_eq!(
            run(&["--check".into(), "extra".into()], None, healthy),
            Err((2, "usage: galera-check [--check]".into()))
        );
    }

    #[test]
    fn rejects_check_without_url() {
        let error = run(&["--check".into()], None, healthy).expect_err("URL is required");
        assert_eq!(error.0, 2);
        assert!(error.1.contains("GALERA_URL is required"));
    }

    #[test]
    fn formats_healthy_result() {
        assert_eq!(
            run(&["--check".into()], Some("url"), healthy),
            Ok(Some("example.test: Synced, wsrep_ready=ON".into()))
        );
    }

    #[test]
    fn maps_failed_check_to_exit_one() {
        assert_eq!(
            run(&["--check".into()], Some("url"), failed),
            Err((1, "unhealthy".into()))
        );
    }

    #[test]
    fn maps_invalid_url_to_exit_two() {
        assert_eq!(
            run(&["--check".into()], Some("url"), invalid_url),
            Err((2, "invalid GALERA_URL: invalid URL".into()))
        );
    }
}
