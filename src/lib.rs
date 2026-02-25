mod report;
mod runner;
mod scenario;
mod transport;

pub use report::{ConformanceOutcome, ConformanceReport};
pub use runner::ConformanceRunner;
pub use transport::{ConformanceTransport, HttpTransport, TransportError};

pub const EXPECTED_PROTOCOL_VERSION: u64 = 3;

#[cfg(test)]
mod tests {
    use serde_json::{json, Value};

    use crate::{
        ConformanceRunner, ConformanceTransport, TransportError, EXPECTED_PROTOCOL_VERSION,
    };

    #[derive(Default)]
    struct MockTransport {
        healthz: Option<Value>,
        readyz: Option<Value>,
        info: Option<Value>,
        unknown_webhook: Option<(u16, Value)>,
    }

    impl ConformanceTransport for MockTransport {
        fn get_json(&self, path: &str) -> Result<Value, TransportError> {
            match path {
                "/healthz" => self
                    .healthz
                    .clone()
                    .ok_or_else(|| TransportError::Protocol("missing healthz fixture".to_owned())),
                "/readyz" => self
                    .readyz
                    .clone()
                    .ok_or_else(|| TransportError::Protocol("missing readyz fixture".to_owned())),
                "/info" => self
                    .info
                    .clone()
                    .ok_or_else(|| TransportError::Protocol("missing info fixture".to_owned())),
                _ => Err(TransportError::Protocol("unknown path".to_owned())),
            }
        }

        fn post_json(&self, path: &str, _body: &Value) -> Result<(u16, Value), TransportError> {
            match path {
                "/channels/nonexistent/webhook" => self.unknown_webhook.clone().ok_or_else(|| {
                    TransportError::Protocol("missing unknown webhook fixture".to_owned())
                }),
                _ => Err(TransportError::Protocol("unknown path".to_owned())),
            }
        }
    }

    #[test]
    fn runner_reports_all_pass_when_invariants_hold() {
        let transport = MockTransport {
            healthz: Some(json!({ "ok": true })),
            readyz: Some(json!({ "ok": true })),
            info: Some(json!({
                "protocolVersion": EXPECTED_PROTOCOL_VERSION,
                "methods": ["health", "status"]
            })),
            unknown_webhook: Some((
                404,
                json!({
                    "ok": false,
                    "error": {
                        "code": "NOT_FOUND"
                    }
                }),
            )),
        };

        let report = ConformanceRunner::new(transport).run();

        assert_eq!(report.total, 5);
        assert_eq!(report.failed, 0);
        assert!(report.outcomes.iter().all(|outcome| outcome.passed));
    }

    #[test]
    fn runner_reports_failure_for_invalid_protocol_version() {
        let transport = MockTransport {
            healthz: Some(json!({ "ok": true })),
            readyz: Some(json!({ "ok": true })),
            info: Some(json!({
                "protocolVersion": 9,
                "methods": ["health", "status"]
            })),
            unknown_webhook: Some((
                404,
                json!({
                    "ok": false,
                    "error": {
                        "code": "NOT_FOUND"
                    }
                }),
            )),
        };

        let report = ConformanceRunner::new(transport).run();

        assert_eq!(report.total, 5);
        assert_eq!(report.failed, 1);
        let protocol_case = report
            .outcomes
            .iter()
            .find(|entry| entry.name == "info.protocol_version")
            .expect("protocol scenario should exist");
        assert!(!protocol_case.passed);
    }
}
