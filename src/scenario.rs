use serde_json::Value;

use crate::{ConformanceOutcome, ConformanceTransport, EXPECTED_PROTOCOL_VERSION};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scenario {
    HealthzOkTrue,
    InfoProtocolVersion,
    UnknownChannelWebhookNotFound,
}

impl Scenario {
    pub fn all() -> [Self; 3] {
        [
            Self::HealthzOkTrue,
            Self::InfoProtocolVersion,
            Self::UnknownChannelWebhookNotFound,
        ]
    }

    pub fn run<T: ConformanceTransport>(&self, transport: &T) -> ConformanceOutcome {
        match self {
            Self::HealthzOkTrue => run_healthz(transport),
            Self::InfoProtocolVersion => run_info_protocol_version(transport),
            Self::UnknownChannelWebhookNotFound => run_unknown_channel_webhook_not_found(transport),
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
