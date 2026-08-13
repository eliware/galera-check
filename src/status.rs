#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GaleraStatus {
    pub(crate) state: String,
    pub(crate) ready: String,
}

pub(crate) fn from_rows(rows: &[(String, String)]) -> GaleraStatus {
    let value = |name: &str| {
        rows.iter()
            .find(|(row_name, _)| row_name == name)
            .map(|(_, value)| value.clone())
            .unwrap_or_default()
    };

    GaleraStatus {
        state: value("wsrep_local_state_comment"),
        ready: value("wsrep_ready"),
    }
}

pub(crate) fn validate(status: &GaleraStatus) -> Result<(), String> {
    if status.state != "Synced" || status.ready != "ON" {
        return Err(format!(
            "unhealthy Galera state: state={} ready={}",
            status.state, status.ready
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{from_rows, validate, GaleraStatus};

    #[test]
    fn extracts_required_status_values() {
        let rows = vec![
            ("wsrep_ready".into(), "ON".into()),
            ("wsrep_local_state_comment".into(), "Synced".into()),
        ];
        assert_eq!(
            from_rows(&rows),
            GaleraStatus {
                state: "Synced".into(),
                ready: "ON".into()
            }
        );
    }

    #[test]
    fn accepts_only_synced_ready_status() {
        assert!(validate(&GaleraStatus {
            state: "Synced".into(),
            ready: "ON".into()
        })
        .is_ok());
        assert!(validate(&GaleraStatus {
            state: "Joining".into(),
            ready: "ON".into()
        })
        .is_err());
        assert!(validate(&GaleraStatus {
            state: "Synced".into(),
            ready: "OFF".into()
        })
        .is_err());
    }
}
