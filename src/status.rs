#[derive(Debug, PartialEq, Eq)]
pub(crate) struct GaleraStatus {
    pub(crate) state: String,
    pub(crate) ready: String,
    pub(crate) cluster_status: String,
}

pub(crate) fn from_rows(rows: &[(String, String)]) -> Result<GaleraStatus, String> {
    let value = |name: &str| -> Result<String, String> {
        let mut values = rows.iter().filter(|(row_name, _)| row_name == name);
        let value = values
            .next()
            .map(|(_, value)| value.clone())
            .unwrap_or_default();
        if values.next().is_some() {
            return Err(format!("duplicate Galera status row: {name}"));
        }
        Ok(value)
    };

    Ok(GaleraStatus {
        state: value("wsrep_local_state_comment")?,
        ready: value("wsrep_ready")?,
        cluster_status: value("wsrep_cluster_status")?,
    })
}

pub(crate) fn validate(status: &GaleraStatus) -> Result<(), String> {
    if status.state != "Synced" || status.ready != "ON" || status.cluster_status != "Primary" {
        return Err(format!(
            "unhealthy Galera state: state={} ready={} cluster_status={}",
            status.state, status.ready, status.cluster_status
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
            ("wsrep_cluster_status".into(), "Primary".into()),
        ];
        assert_eq!(
            from_rows(&rows),
            Ok(GaleraStatus {
                state: "Synced".into(),
                ready: "ON".into(),
                cluster_status: "Primary".into()
            })
        );
    }

    #[test]
    fn missing_status_values_are_unhealthy() {
        assert_eq!(
            from_rows(&[]),
            Ok(GaleraStatus {
                state: "".into(),
                ready: "".into(),
                cluster_status: "".into()
            })
        );
        assert_eq!(
            from_rows(&[("wsrep_local_state_comment".into(), "Synced".into())]),
            Ok(GaleraStatus {
                state: "Synced".into(),
                ready: "".into(),
                cluster_status: "".into()
            })
        );
    }

    #[test]
    fn rejects_duplicate_ready_rows() {
        let rows = vec![
            ("wsrep_local_state_comment".into(), "Synced".into()),
            ("wsrep_ready".into(), "ON".into()),
            ("wsrep_ready".into(), "OFF".into()),
        ];
        assert_eq!(
            from_rows(&rows),
            Err("duplicate Galera status row: wsrep_ready".into())
        );
    }

    #[test]
    fn accepts_only_synced_ready_status() {
        assert!(validate(&GaleraStatus {
            state: "Synced".into(),
            ready: "ON".into(),
            cluster_status: "Primary".into()
        })
        .is_ok());
        assert!(validate(&GaleraStatus {
            state: "Joining".into(),
            ready: "ON".into(),
            cluster_status: "Primary".into()
        })
        .is_err());
        assert!(validate(&GaleraStatus {
            state: "Synced".into(),
            ready: "OFF".into(),
            cluster_status: "Primary".into()
        })
        .is_err());
    }

    #[test]
    fn rejects_non_primary_and_malformed_cluster_status() {
        for value in ["Non-Primary", "", "primary", "Primary extra"] {
            assert!(validate(&GaleraStatus {
                state: "Synced".into(),
                ready: "ON".into(),
                cluster_status: value.into()
            })
            .is_err());
        }
    }

    #[test]
    fn rejects_duplicate_cluster_status() {
        let rows = vec![
            ("wsrep_cluster_status".into(), "Primary".into()),
            ("wsrep_cluster_status".into(), "Non-Primary".into()),
        ];
        assert!(from_rows(&rows).is_err());
    }
}
