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
}

impl Scenario {
    pub fn all() -> [Self; 6] {
        [
            Self::HealthzOkTrue,
            Self::ReadyzOkTrue,
            Self::InfoProtocolVersion,
            Self::InfoMethodsIncludeHealthAndStatus,
            Self::UnknownChannelWebhookNotFound,
            Self::WsHandshakeRequiresConnectFirstFrame,
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
