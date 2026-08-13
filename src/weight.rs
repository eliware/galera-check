use std::collections::HashMap;

const MAX_QUEUE: u64 = 16;

pub(crate) fn calculate(rows: &[(String, String)], latency_ms: u64) -> u8 {
    let values: HashMap<&str, &str> = rows
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    if values.get("wsrep_local_state_comment") != Some(&"Synced")
        || values.get("wsrep_ready") != Some(&"ON")
    {
        return 0;
    }
    let recv = number(&values, "wsrep_local_recv_queue");
    let send = number(&values, "wsrep_local_send_queue");
    let paused = fraction(&values, "wsrep_flow_control_paused");
    if recv > MAX_QUEUE || send > MAX_QUEUE || paused >= 0.05 {
        return 0;
    }
    let pressure = ((recv.max(send) * 40) / MAX_QUEUE).min(40)
        + (paused * 400.0).min(20.0) as u64
        + (latency_ms.saturating_sub(1) / 10).min(30);
    (100u64.saturating_sub(pressure).max(1)) as u8
}

fn number(values: &HashMap<&str, &str>, name: &str) -> u64 {
    values
        .get(name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(u64::MAX)
}

fn fraction(values: &HashMap<&str, &str>, name: &str) -> f64 {
    values
        .get(name)
        .and_then(|value| value.parse().ok())
        .unwrap_or(f64::INFINITY)
}

#[cfg(test)]
mod tests {
    use super::calculate;

    fn rows(queue: &str, paused: &str) -> Vec<(String, String)> {
        vec![
            ("wsrep_local_state_comment".into(), "Synced".into()),
            ("wsrep_ready".into(), "ON".into()),
            ("wsrep_local_recv_queue".into(), queue.into()),
            ("wsrep_local_send_queue".into(), "0".into()),
            ("wsrep_flow_control_paused".into(), paused.into()),
        ]
    }

    #[test]
    fn healthy_node_gets_high_weight() {
        assert_eq!(calculate(&rows("0", "0"), 1), 100);
    }

    #[test]
    fn pressure_reduces_weight() {
        assert!(calculate(&rows("8", "0.01"), 101) < 70);
    }

    #[test]
    fn unsafe_state_gets_zero() {
        let mut unhealthy = rows("0", "0");
        unhealthy[0].1 = "Joining".into();
        assert_eq!(calculate(&unhealthy, 1), 0);
        let mut paused = rows("0", "0.05");
        paused[4].1 = "0.05".into();
        assert_eq!(calculate(&paused, 1), 0);
        assert_eq!(calculate(&rows("17", "0"), 1), 0);
    }
}
