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
        tools_invoke: Option<(u16, Value)>,
        tools_invoke_unknown: Option<(u16, Value)>,
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

        fn post_json(&self, path: &str, body: &Value) -> Result<(u16, Value), TransportError> {
            match path {
                "/channels/nonexistent/webhook" => self.unknown_webhook.clone().ok_or_else(|| {
                    TransportError::Protocol("missing unknown webhook fixture".to_owned())
                }),
                "/tools/invoke" => {
                    let tool = body.get("tool").and_then(Value::as_str).ok_or_else(|| {
                        TransportError::Protocol(
                            "missing tools invoke tool in fixture request".to_owned(),
                        )
                    })?;
                    if tool != "gateway.request" {
                        return self.tools_invoke_unknown.clone().ok_or_else(|| {
                            TransportError::Protocol(
                                "missing tools invoke unknown fixture".to_owned(),
                            )
                        });
                    }
                    let method = body
                        .get("args")
                        .and_then(|args| args.get("method"))
                        .and_then(Value::as_str)
                        .or_else(|| body.get("action").and_then(Value::as_str))
                        .ok_or_else(|| {
                            TransportError::Protocol(
                                "missing tools invoke method/action in fixture request".to_owned(),
                            )
                        })?;
                    if method != "health" {
                        return Err(TransportError::Protocol(format!(
                            "unexpected tools invoke payload: tool={tool}, method={method}"
                        )));
                    }
                    self.tools_invoke.clone().ok_or_else(|| {
                        TransportError::Protocol("missing tools invoke fixture".to_owned())
                    })
                }
                _ => Err(TransportError::Protocol("unknown path".to_owned())),
            }
        }

        fn websocket_first_response(&self, _frame: &Value) -> Result<Value, TransportError> {
            self.websocket_response.clone().ok_or_else(|| {
                TransportError::Protocol("missing websocket response fixture".to_owned())
            })
        }

        fn websocket_exchange(&self, frames: &[Value]) -> Result<Vec<Value>, TransportError> {
            let methods = frames
                .iter()
                .map(|frame| {
                    frame.get("method").and_then(Value::as_str).ok_or_else(|| {
                        TransportError::Protocol("missing method in websocket fixture".to_owned())
                    })
                })
                .collect::<Result<Vec<_>, _>>()?;

            if methods.as_slice() == ["connect", "agent.wait"] {
                let wait_run_id = frames[1]
                    .get("params")
                    .and_then(|params| params.get("runId"))
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing wait runId in websocket fixture".to_owned(),
                        )
                    })?;

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
                            "runId": wait_run_id,
                            "status": "timeout"
                        }
                    }),
                ]);
            }

            if methods.as_slice() == ["connect", "channels.status"] {
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
                            "ts": 1,
                            "channels": [{
                                "id": "webchat",
                                "connected": true,
                                "kind": "internal"
                            }],
                            "channelOrder": ["webchat"],
                            "channelLabels": { "webchat": "webchat" },
                            "channelMeta": {
                                "webchat": {
                                    "kind": "internal",
                                    "label": "webchat"
                                }
                            },
                            "channelsById": {
                                "webchat": {
                                    "connected": true,
                                    "kind": "internal"
                                }
                            },
                            "channelAccounts": {
                                "webchat": [{
                                    "accountId": "default",
                                    "connected": true,
                                    "kind": "internal",
                                    "loggedOutAtMs": Value::Null
                                }]
                            },
                            "channelDefaultAccountId": {
                                "webchat": "default"
                            }
                        }
                    }),
                ]);
            }

            if methods.as_slice() == ["connect", "channels.logout", "channels.status"] {
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
                            "ok": true,
                            "channel": "webchat",
                            "accountId": "ops",
                            "loggedOut": true
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "ts": 2,
                            "channels": [
                                {
                                    "id": "webchat",
                                    "connected": true,
                                    "kind": "internal"
                                },
                                {
                                    "id": "webchat",
                                    "accountId": "ops",
                                    "connected": false,
                                    "kind": "internal",
                                    "loggedOutAtMs": 42
                                }
                            ],
                            "channelOrder": ["webchat"],
                            "channelLabels": { "webchat": "webchat" },
                            "channelMeta": {
                                "webchat": {
                                    "kind": "internal",
                                    "label": "webchat"
                                }
                            },
                            "channelsById": {
                                "webchat": {
                                    "connected": true,
                                    "kind": "internal"
                                }
                            },
                            "channelAccounts": {
                                "webchat": [
                                    {
                                        "accountId": "default",
                                        "connected": true,
                                        "kind": "internal",
                                        "loggedOutAtMs": Value::Null
                                    },
                                    {
                                        "accountId": "ops",
                                        "connected": false,
                                        "kind": "internal",
                                        "loggedOutAtMs": 42
                                    }
                                ]
                            },
                            "channelDefaultAccountId": {
                                "webchat": "default"
                            }
                        }
                    }),
                ]);
            }

            if methods.as_slice() == ["connect", "chat.send", "agent.wait"] {
                let chat_params = frames[1].get("params").ok_or_else(|| {
                    TransportError::Protocol(
                        "missing chat.send params in websocket fixture".to_owned(),
                    )
                })?;
                let run_id = chat_params
                    .get("idempotencyKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing chat.send idempotencyKey in websocket fixture".to_owned(),
                        )
                    })?;
                let session_key = chat_params
                    .get("sessionKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing chat.send sessionKey in websocket fixture".to_owned(),
                        )
                    })?;

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
                            "runId": run_id,
                            "status": "queued",
                            "sessionKey": session_key,
                            "message": Value::Null
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "status": "completed",
                            "result": {
                                "output": "Echo: conformance deferred chat",
                                "sessionKey": session_key
                            }
                        }
                    }),
                ]);
            }

            if methods.as_slice() == ["connect", "chat.send", "chat.abort", "agent.wait"] {
                let chat_params = frames[1].get("params").ok_or_else(|| {
                    TransportError::Protocol(
                        "missing chat.send params in websocket fixture".to_owned(),
                    )
                })?;
                let run_id = chat_params
                    .get("idempotencyKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing chat.send idempotencyKey in websocket fixture".to_owned(),
                        )
                    })?;
                let session_key = chat_params
                    .get("sessionKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing chat.send sessionKey in websocket fixture".to_owned(),
                        )
                    })?;

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
                            "runId": run_id,
                            "status": "queued",
                            "sessionKey": session_key,
                            "message": Value::Null
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

            if methods.as_slice()
                == [
                    "connect",
                    "chat.send",
                    "chat.send",
                    "chat.abort",
                    "agent.wait",
                    "agent.wait",
                ]
            {
                let first_params = frames[1].get("params").ok_or_else(|| {
                    TransportError::Protocol(
                        "missing first chat.send params in websocket fixture".to_owned(),
                    )
                })?;
                let second_params = frames[2].get("params").ok_or_else(|| {
                    TransportError::Protocol(
                        "missing second chat.send params in websocket fixture".to_owned(),
                    )
                })?;
                let run_id_one = first_params
                    .get("idempotencyKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing first chat.send idempotencyKey in websocket fixture"
                                .to_owned(),
                        )
                    })?;
                let run_id_two = second_params
                    .get("idempotencyKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing second chat.send idempotencyKey in websocket fixture"
                                .to_owned(),
                        )
                    })?;
                let session_key = first_params
                    .get("sessionKey")
                    .and_then(Value::as_str)
                    .ok_or_else(|| {
                        TransportError::Protocol(
                            "missing chat.send sessionKey in websocket fixture".to_owned(),
                        )
                    })?;

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
                            "runId": run_id_one,
                            "status": "queued",
                            "sessionKey": session_key,
                            "message": Value::Null
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "runId": run_id_two,
                            "status": "queued",
                            "sessionKey": session_key,
                            "message": Value::Null
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "aborted": true,
                            "runIds": [run_id_one, run_id_two]
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

            if methods.as_slice() == ["connect", "agent", "chat.abort"] {
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
                        "ok": false,
                        "error": {
                            "code": "INVALID_REQUEST"
                        }
                    }),
                ]);
            }

            if methods.as_slice() == ["connect", "agent", "agent.wait", "chat.abort"] {
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
                            "status": "completed",
                            "result": {
                                "output": "Echo: complete then abort",
                                "sessionKey": session_key
                            }
                        }
                    }),
                    json!({
                        "ok": true,
                        "payload": {
                            "aborted": false,
                            "runIds": [run_id]
                        }
                    }),
                ]);
            }

            if methods.as_slice()
                == [
                    "connect",
                    "agent",
                    "agent",
                    "chat.abort",
                    "agent.wait",
                    "agent.wait",
                ]
            {
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

            if methods.as_slice() == ["connect", "agent", "chat.abort", "agent.wait"] {
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

            if methods.as_slice() == ["connect", "agent", "agent.wait"] {
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
                            "status": "completed",
                            "result": {
                                "output": "Echo: conformance deferred",
                                "sessionKey": session_key
                            }
                        }
                    }),
                ]);
            }

            Err(TransportError::Protocol(format!(
                "unsupported websocket fixture methods: {methods:?}"
            )))
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
            tools_invoke: Some((
                200,
                json!({
                    "ok": true,
                    "result": {
                        "ok": true
                    }
                }),
            )),
            tools_invoke_unknown: Some((
                404,
                json!({
                    "ok": false,
                    "error": {
                        "type": "not_found"
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

        assert_eq!(report.total, 20);
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
            tools_invoke: Some((
                200,
                json!({
                    "ok": true,
                    "result": {
                        "ok": true
                    }
                }),
            )),
            tools_invoke_unknown: Some((
                404,
                json!({
                    "ok": false,
                    "error": {
                        "type": "not_found"
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

        assert_eq!(report.total, 20);
        assert_eq!(report.failed, 1);
        let protocol_case = report
            .outcomes
            .iter()
            .find(|entry| entry.name == "info.protocol_version")
            .expect("protocol scenario should exist");
        assert!(!protocol_case.passed);
    }
}
