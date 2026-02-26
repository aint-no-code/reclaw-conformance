use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::{ConformanceOutcome, ConformanceTransport, EXPECTED_PROTOCOL_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    HealthzOkTrue,
    ReadyzOkTrue,
    InfoProtocolVersion,
    InfoMethodsIncludeHealthAndStatus,
    UnknownChannelWebhookNotFound,
    WsHandshakeRequiresConnectFirstFrame,
    WsAgentDeferredWaitCompletes,
    WsChatAbortCancelsDeferredRun,
}

impl Scenario {
    pub fn all() -> [Self; 8] {
        [
            Self::HealthzOkTrue,
            Self::ReadyzOkTrue,
            Self::InfoProtocolVersion,
            Self::InfoMethodsIncludeHealthAndStatus,
            Self::UnknownChannelWebhookNotFound,
            Self::WsHandshakeRequiresConnectFirstFrame,
            Self::WsAgentDeferredWaitCompletes,
            Self::WsChatAbortCancelsDeferredRun,
        ]
    }

    pub fn run<T: ConformanceTransport>(&self, transport: &T) -> ConformanceOutcome {
        match self {
            Self::HealthzOkTrue => run_healthz(transport),
            Self::ReadyzOkTrue => run_readyz(transport),
            Self::InfoProtocolVersion => run_info_protocol_version(transport),
            Self::InfoMethodsIncludeHealthAndStatus => {
                run_info_methods_include_health_and_status(transport)
            }
            Self::UnknownChannelWebhookNotFound => run_unknown_channel_webhook_not_found(transport),
            Self::WsHandshakeRequiresConnectFirstFrame => {
                run_ws_handshake_requires_connect_first_frame(transport)
            }
            Self::WsAgentDeferredWaitCompletes => run_ws_agent_deferred_wait_completes(transport),
            Self::WsChatAbortCancelsDeferredRun => {
                run_ws_chat_abort_cancels_deferred_run(transport)
            }
        }
    }
}

fn run_healthz<T: ConformanceTransport>(transport: &T) -> ConformanceOutcome {
    let name = "healthz.ok_true";

    match transport.get_json("/healthz") {
        Ok(payload) => {
            let ok = payload.get("ok").and_then(Value::as_bool).unwrap_or(false);
            if ok {
                ConformanceOutcome {
                    name,
                    passed: true,
                    detail: "health endpoint returned ok=true".to_owned(),
                }
            } else {
                ConformanceOutcome {
                    name,
                    passed: false,
                    detail: "health endpoint did not return {\"ok\":true}".to_owned(),
                }
            }
        }
        Err(error) => ConformanceOutcome {
            name,
            passed: false,
            detail: format!("health endpoint request failed: {error}"),
        },
    }
}

fn run_readyz<T: ConformanceTransport>(transport: &T) -> ConformanceOutcome {
    let name = "readyz.ok_true";

    match transport.get_json("/readyz") {
        Ok(payload) => {
            let ok = payload.get("ok").and_then(Value::as_bool).unwrap_or(false);
            if ok {
                ConformanceOutcome {
                    name,
                    passed: true,
                    detail: "ready endpoint returned ok=true".to_owned(),
                }
            } else {
                ConformanceOutcome {
                    name,
                    passed: false,
                    detail: "ready endpoint did not return {\"ok\":true}".to_owned(),
                }
            }
        }
        Err(error) => ConformanceOutcome {
            name,
            passed: false,
            detail: format!("ready endpoint request failed: {error}"),
        },
    }
}

fn run_info_protocol_version<T: ConformanceTransport>(transport: &T) -> ConformanceOutcome {
    let name = "info.protocol_version";

    match transport.get_json("/info") {
        Ok(payload) => {
            let actual = payload.get("protocolVersion").and_then(Value::as_u64);
            match actual {
                Some(version) if version == EXPECTED_PROTOCOL_VERSION => ConformanceOutcome {
                    name,
                    passed: true,
                    detail: format!("protocolVersion={version}"),
                },
                Some(version) => ConformanceOutcome {
                    name,
                    passed: false,
                    detail: format!(
                        "expected protocolVersion={}, found {version}",
                        EXPECTED_PROTOCOL_VERSION
                    ),
                },
                None => ConformanceOutcome {
                    name,
                    passed: false,
                    detail: "info endpoint missing numeric protocolVersion".to_owned(),
                },
            }
        }
        Err(error) => ConformanceOutcome {
            name,
            passed: false,
            detail: format!("info endpoint request failed: {error}"),
        },
    }
}

fn run_info_methods_include_health_and_status<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "info.methods_include_health_status";

    match transport.get_json("/info") {
        Ok(payload) => {
            let methods = payload
                .get("methods")
                .and_then(Value::as_array)
                .map(|values| {
                    values
                        .iter()
                        .filter_map(Value::as_str)
                        .map(str::to_owned)
                        .collect::<Vec<_>>()
                })
                .unwrap_or_default();

            let has_health = methods.iter().any(|method| method == "health");
            let has_status = methods.iter().any(|method| method == "status");
            if has_health && has_status {
                ConformanceOutcome {
                    name,
                    passed: true,
                    detail: "info.methods includes health and status".to_owned(),
                }
            } else {
                ConformanceOutcome {
                    name,
                    passed: false,
                    detail: format!(
                        "expected info.methods to include health and status, found {methods:?}"
                    ),
                }
            }
        }
        Err(error) => ConformanceOutcome {
            name,
            passed: false,
            detail: format!("info endpoint request failed: {error}"),
        },
    }
}

fn run_unknown_channel_webhook_not_found<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "channels.unknown_webhook_not_found";
    let payload = serde_json::json!({});

    match transport.post_json("/channels/nonexistent/webhook", &payload) {
        Ok((status, body)) => {
            let error_code = body
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_str);

            if status == 404 && error_code == Some("NOT_FOUND") {
                ConformanceOutcome {
                    name,
                    passed: true,
                    detail: "unknown channel webhook returns 404 NOT_FOUND".to_owned(),
                }
            } else {
                ConformanceOutcome {
                    name,
                    passed: false,
                    detail: format!(
                        "expected status=404 and error.code=NOT_FOUND, found status={status}, error.code={error_code:?}"
                    ),
                }
            }
        }
        Err(error) => ConformanceOutcome {
            name,
            passed: false,
            detail: format!("unknown channel webhook request failed: {error}"),
        },
    }
}

fn run_ws_handshake_requires_connect_first_frame<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.handshake_requires_connect_first_frame";
    let request = serde_json::json!({
        "type": "req",
        "id": "conformance-handshake-invalid-1",
        "method": "health",
        "params": {}
    });

    match transport.websocket_first_response(&request) {
        Ok(response) => {
            let ok = response.get("ok").and_then(Value::as_bool).unwrap_or(true);
            let code = response
                .get("error")
                .and_then(|error| error.get("code"))
                .and_then(Value::as_str);

            if !ok && code == Some("INVALID_REQUEST") {
                ConformanceOutcome {
                    name,
                    passed: true,
                    detail: "ws handshake rejects non-connect first request".to_owned(),
                }
            } else {
                ConformanceOutcome {
                    name,
                    passed: false,
                    detail: format!(
                        "expected ok=false and error.code=INVALID_REQUEST, found ok={ok}, error.code={code:?}"
                    ),
                }
            }
        }
        Err(error) => ConformanceOutcome {
            name,
            passed: false,
            detail: format!("websocket handshake request failed: {error}"),
        },
    }
}

fn run_ws_agent_deferred_wait_completes<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.agent_deferred_wait_completes";
    let run_id = unique_run_id("conformance-deferred");
    let input = "conformance deferred";
    let session_key = format!("agent:main:{run_id}");

    let connect_id = format!("{run_id}-connect");
    let agent_id = format!("{run_id}-agent");
    let wait_id = format!("{run_id}-wait");
    let connect = ws_connect_frame(&connect_id);
    let agent = serde_json::json!({
        "type": "req",
        "id": agent_id,
        "method": "agent",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
            "agentId": "main",
            "input": input,
            "deferred": true,
        }
    });
    let wait = serde_json::json!({
        "type": "req",
        "id": wait_id,
        "method": "agent.wait",
        "params": {
            "runId": run_id,
            "timeoutMs": 2000
        }
    });

    let responses = match transport.websocket_exchange(&[connect, agent, wait]) {
        Ok(responses) => responses,
        Err(error) => {
            return ConformanceOutcome {
                name,
                passed: false,
                detail: format!("websocket exchange failed: {error}"),
            };
        }
    };
    if responses.len() != 3 {
        return ConformanceOutcome {
            name,
            passed: false,
            detail: format!("expected 3 websocket responses, found {}", responses.len()),
        };
    }

    let connect_ok = responses[0]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let queued_summary = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("summary"))
        .and_then(Value::as_str);
    let final_status = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let final_output = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("output"))
        .and_then(Value::as_str);
    let final_session_key = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("sessionKey"))
        .and_then(Value::as_str);

    if connect_ok
        && queued_summary == Some("queued")
        && final_status == Some("completed")
        && final_output == Some("Echo: conformance deferred")
        && final_session_key == Some(session_key.as_str())
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "deferred agent run transitions queued->completed via agent.wait".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected queued/completed deferred lifecycle, found summary={queued_summary:?}, status={final_status:?}, output={final_output:?}, sessionKey={final_session_key:?}"
            ),
        }
    }
}

fn run_ws_chat_abort_cancels_deferred_run<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_abort_cancels_deferred_run";
    let run_id = unique_run_id("conformance-abort");
    let session_key = format!("agent:main:{run_id}");

    let connect_id = format!("{run_id}-connect");
    let agent_id = format!("{run_id}-agent");
    let abort_id = format!("{run_id}-abort");
    let wait_id = format!("{run_id}-wait");
    let connect = ws_connect_frame(&connect_id);
    let agent = serde_json::json!({
        "type": "req",
        "id": agent_id,
        "method": "agent",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
            "agentId": "main",
            "input": "conformance abort",
            "deferred": true,
        }
    });
    let abort = serde_json::json!({
        "type": "req",
        "id": abort_id,
        "method": "chat.abort",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
        }
    });
    let wait = serde_json::json!({
        "type": "req",
        "id": wait_id,
        "method": "agent.wait",
        "params": {
            "runId": run_id,
            "timeoutMs": 2000
        }
    });

    let responses = match transport.websocket_exchange(&[connect, agent, abort, wait]) {
        Ok(responses) => responses,
        Err(error) => {
            return ConformanceOutcome {
                name,
                passed: false,
                detail: format!("websocket exchange failed: {error}"),
            };
        }
    };
    if responses.len() != 4 {
        return ConformanceOutcome {
            name,
            passed: false,
            detail: format!("expected 4 websocket responses, found {}", responses.len()),
        };
    }

    let connect_ok = responses[0]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let queued_summary = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("summary"))
        .and_then(Value::as_str);
    let abort_ok = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("aborted"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let wait_status = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let wait_output_is_null = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("output"))
        .is_some_and(Value::is_null);
    let wait_session_key = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("sessionKey"))
        .and_then(Value::as_str);

    if connect_ok
        && queued_summary == Some("queued")
        && abort_ok
        && wait_status == Some("aborted")
        && wait_output_is_null
        && wait_session_key == Some(session_key.as_str())
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "chat.abort cancels deferred run and agent.wait reports aborted".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected abort lifecycle, found summary={queued_summary:?}, aborted={abort_ok}, status={wait_status:?}, sessionKey={wait_session_key:?}, outputIsNull={wait_output_is_null}"
            ),
        }
    }
}

fn ws_connect_frame(id: &str) -> Value {
    serde_json::json!({
        "type": "req",
        "id": id,
        "method": "connect",
        "params": {
            "minProtocol": 1,
            "maxProtocol": 3,
            "client": {
                "id": "reclaw-conformance",
                "displayName": "Reclaw Conformance",
                "version": "0.1.0",
                "platform": "conformance",
                "mode": "cli",
            },
            "role": "operator",
            "scopes": [],
            "auth": {
                "token": Value::Null
            }
        }
    })
}

fn unique_run_id(prefix: &str) -> String {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_millis())
        .unwrap_or(0);
    format!("{prefix}-{now_ms}")
}
