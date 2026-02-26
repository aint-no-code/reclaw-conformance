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
        websocket_response: Option<Value>,
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

        fn websocket_first_response(&self, _frame: &Value) -> Result<Value, TransportError> {
            self.websocket_response.clone().ok_or_else(|| {
                TransportError::Protocol("missing websocket response fixture".to_owned())
            })
        }

        fn websocket_exchange(&self, frames: &[Value]) -> Result<Vec<Value>, TransportError> {
            let agent_runs = frames
                .iter()
                .filter(|frame| frame.get("method").and_then(Value::as_str) == Some("agent"))
                .map(|frame| {
                    let params = frame.get("params").ok_or_else(|| {
                        TransportError::Protocol(
                            "missing agent params in websocket fixture".to_owned(),
                        )
                    })?;
                    let run_id = params
                        .get("runId")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            TransportError::Protocol(
                                "missing runId in websocket fixture".to_owned(),
                            )
                        })?
                        .to_owned();
                    let session_key = params
                        .get("sessionKey")
                        .and_then(Value::as_str)
                        .ok_or_else(|| {
                            TransportError::Protocol(
                                "missing sessionKey in websocket fixture".to_owned(),
                            )
                        })?
                        .to_owned();
                    Ok((run_id, session_key))
                })
                .collect::<Result<Vec<_>, _>>()?;
            let (run_id, session_key) = agent_runs.first().cloned().ok_or_else(|| {
                TransportError::Protocol("missing agent frame in websocket fixture".to_owned())
            })?;
            let has_chat_abort = frames
                .iter()
                .any(|frame| frame.get("method").and_then(Value::as_str) == Some("chat.abort"));
            if has_chat_abort {
                if agent_runs.len() > 1 {
                    let second_run_id = &agent_runs[1].0;
                    return Ok(vec![
                        json!({
                            "ok": true,
                            "payload": {
                                "type": "hello-ok"
                            }
                        }),
                        json!({
                            "ok": true,
                            "payload": {
                                "summary": "queued"
                            }
                        }),
                        json!({
                            "ok": true,
                            "payload": {
                                "summary": "queued"
                            }
                        }),
                        json!({
                            "ok": true,
                            "payload": {
                                "aborted": true,
                                "runIds": [run_id, second_run_id]
                            }
                        }),
                        json!({
                            "ok": true,
                            "payload": {
                                "status": "aborted",
                                "result": {
                                    "output": Value::Null,
                                    "sessionKey": session_key
                                }
                            }
                        }),
                        json!({
                            "ok": true,
                            "payload": {
                                "status": "aborted",
                                "result": {
                                    "output": Value::Null,
                                    "sessionKey": session_key
                                }
                            }
                        }),
                    ]);
                }
                return Ok(vec![
                    json!({
                        "ok": true,
                        "payload": {
                            "type": "hello-ok"
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "summary": "queued"
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "aborted": true,
                            "runIds": [run_id]
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "status": "aborted",
                            "result": {
                                "output": Value::Null,
                                "sessionKey": session_key
                            }
                        }
                    }),
                ]);
            }

            Ok(vec![
                json!({
                    "ok": true,
                    "payload": {
                        "type": "hello-ok"
                    }
                }),
                json!({
                    "ok": true,
                    "payload": {
                        "summary": "queued"
                    }
                }),
                json!({
                    "ok": true,
                    "payload": {
                        "status": "completed",
                        "result": {
                            "output": "Echo: conformance deferred",
                            "sessionKey": session_key
                        }
                    }
                }),
            ])
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
            websocket_response: Some(json!({
                "type": "res",
                "id": "conformance-handshake-invalid-1",
                "ok": false,
                "error": {
                    "code": "INVALID_REQUEST"
                }
            })),
        };

        let report = ConformanceRunner::new(transport).run();

        assert_eq!(report.total, 9);
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
            websocket_response: Some(json!({
                "type": "res",
                "id": "conformance-handshake-invalid-1",
                "ok": false,
                "error": {
                    "code": "INVALID_REQUEST"
                }
            })),
        };

        let report = ConformanceRunner::new(transport).run();

        assert_eq!(report.total, 9);
        assert_eq!(report.failed, 1);
        let protocol_case = report
            .outcomes
            .iter()
            .find(|entry| entry.name == "info.protocol_version")
            .expect("protocol scenario should exist");
        assert!(!protocol_case.passed);
    }
}
