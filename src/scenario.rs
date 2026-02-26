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
    WsChannelsStatusIncludesAccountViews,
    WsChannelsLogoutAccountPersists,
    WsAgentDeferredWaitCompletes,
    WsChatSendDeferredWaitCompletes,
    WsChatAbortCancelsDeferredRun,
    WsChatAbortCancelsDeferredChatSendRun,
    WsChatAbortSessionWideCancelsDeferredChatSendRuns,
    WsChatAbortSessionWideCancelsRuns,
    WsAgentWaitTimeoutForMissingRun,
    WsChatAbortRejectsRunSessionMismatch,
    WsChatAbortCompletedRunNoop,
}

impl Scenario {
    pub fn all() -> [Self; 17] {
        [
            Self::HealthzOkTrue,
            Self::ReadyzOkTrue,
            Self::InfoProtocolVersion,
            Self::InfoMethodsIncludeHealthAndStatus,
            Self::UnknownChannelWebhookNotFound,
            Self::WsHandshakeRequiresConnectFirstFrame,
            Self::WsChannelsStatusIncludesAccountViews,
            Self::WsChannelsLogoutAccountPersists,
            Self::WsAgentDeferredWaitCompletes,
            Self::WsChatSendDeferredWaitCompletes,
            Self::WsChatAbortCancelsDeferredRun,
            Self::WsChatAbortCancelsDeferredChatSendRun,
            Self::WsChatAbortSessionWideCancelsDeferredChatSendRuns,
            Self::WsChatAbortSessionWideCancelsRuns,
            Self::WsAgentWaitTimeoutForMissingRun,
            Self::WsChatAbortRejectsRunSessionMismatch,
            Self::WsChatAbortCompletedRunNoop,
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
            Self::WsChannelsStatusIncludesAccountViews => {
                run_ws_channels_status_includes_account_views(transport)
            }
            Self::WsChannelsLogoutAccountPersists => {
                run_ws_channels_logout_account_persists(transport)
            }
            Self::WsAgentDeferredWaitCompletes => run_ws_agent_deferred_wait_completes(transport),
            Self::WsChatSendDeferredWaitCompletes => {
                run_ws_chat_send_deferred_wait_completes(transport)
            }
            Self::WsChatAbortCancelsDeferredRun => {
                run_ws_chat_abort_cancels_deferred_run(transport)
            }
            Self::WsChatAbortCancelsDeferredChatSendRun => {
                run_ws_chat_abort_cancels_deferred_chat_send_run(transport)
            }
            Self::WsChatAbortSessionWideCancelsDeferredChatSendRuns => {
                run_ws_chat_abort_session_wide_cancels_deferred_chat_send_runs(transport)
            }
            Self::WsChatAbortSessionWideCancelsRuns => {
                run_ws_chat_abort_session_wide_cancels_runs(transport)
            }
            Self::WsAgentWaitTimeoutForMissingRun => {
                run_ws_agent_wait_timeout_for_missing_run(transport)
            }
            Self::WsChatAbortRejectsRunSessionMismatch => {
                run_ws_chat_abort_rejects_run_session_mismatch(transport)
            }
            Self::WsChatAbortCompletedRunNoop => run_ws_chat_abort_completed_run_noop(transport),
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

fn run_ws_channels_status_includes_account_views<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.channels_status_includes_account_views";
    let run_id = unique_run_id("conformance-channels-status");
    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let status = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-status"),
        "method": "channels.status",
        "params": {}
    });

    let responses = match transport.websocket_exchange(&[connect, status]) {
        Ok(responses) => responses,
        Err(error) => {
            return ConformanceOutcome {
                name,
                passed: false,
                detail: format!("websocket exchange failed: {error}"),
            };
        }
    };
    if responses.len() != 2 {
        return ConformanceOutcome {
            name,
            passed: false,
            detail: format!("expected 2 websocket responses, found {}", responses.len()),
        };
    }

    let connect_ok = responses[0]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let payload = responses[1].get("payload").cloned().unwrap_or(Value::Null);
    let has_channels_list = payload.get("channels").is_some_and(Value::is_array);
    let has_channel_order = payload.get("channelOrder").is_some_and(Value::is_array);
    let has_channel_labels = payload.get("channelLabels").is_some_and(Value::is_object);
    let has_channels_by_id = payload.get("channelsById").is_some_and(Value::is_object);
    let has_channel_accounts = payload.get("channelAccounts").is_some_and(Value::is_object);
    let has_channel_default_account_id = payload
        .get("channelDefaultAccountId")
        .is_some_and(Value::is_object);
    let webchat_default = payload
        .get("channelDefaultAccountId")
        .and_then(|value| value.get("webchat"))
        .and_then(Value::as_str);
    let webchat_connected = payload
        .get("channelsById")
        .and_then(|value| value.get("webchat"))
        .and_then(|value| value.get("connected"))
        .and_then(Value::as_bool);

    if connect_ok
        && has_channels_list
        && has_channel_order
        && has_channel_labels
        && has_channels_by_id
        && has_channel_accounts
        && has_channel_default_account_id
        && webchat_default == Some("default")
        && webchat_connected == Some(true)
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "channels.status includes account-aware channel summary views".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected channel account views, found channels={has_channels_list}, order={has_channel_order}, labels={has_channel_labels}, byId={has_channels_by_id}, accounts={has_channel_accounts}, defaults={has_channel_default_account_id}, webchatDefault={webchat_default:?}, webchatConnected={webchat_connected:?}"
            ),
        }
    }
}

fn run_ws_channels_logout_account_persists<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.channels_logout_account_persists";
    let run_id = unique_run_id("conformance-channels-logout");
    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let logout = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-logout"),
        "method": "channels.logout",
        "params": {
            "channel": "webchat",
            "accountId": "ops",
        }
    });
    let status = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-status"),
        "method": "channels.status",
        "params": {}
    });

    let responses = match transport.websocket_exchange(&[connect, logout, status]) {
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
    let logout_ok = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("loggedOut"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let logout_account = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("accountId"))
        .and_then(Value::as_str);
    let ops_persisted = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("channelAccounts"))
        .and_then(|payload| payload.get("webchat"))
        .and_then(Value::as_array)
        .is_some_and(|entries| {
            entries.iter().any(|entry| {
                entry.get("accountId").and_then(Value::as_str) == Some("ops")
                    && entry.get("connected").and_then(Value::as_bool) == Some(false)
            })
        });
    let webchat_connected = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("channelsById"))
        .and_then(|payload| payload.get("webchat"))
        .and_then(|entry| entry.get("connected"))
        .and_then(Value::as_bool);

    if connect_ok
        && logout_ok
        && logout_account == Some("ops")
        && ops_persisted
        && webchat_connected == Some(true)
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "channels.logout(accountId) persists account-specific disconnected state"
                .to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected account-aware logout persistence, found loggedOut={logout_ok}, accountId={logout_account:?}, opsPersisted={ops_persisted}, webchatConnected={webchat_connected:?}"
            ),
        }
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

fn run_ws_chat_send_deferred_wait_completes<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_send_deferred_wait_completes";
    let run_id = unique_run_id("conformance-chat-deferred");
    let input = "conformance deferred chat";
    let session_key = format!("agent:main:{run_id}");

    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let chat_send = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-chat-send"),
        "method": "chat.send",
        "params": {
            "sessionKey": session_key,
            "message": input,
            "idempotencyKey": run_id,
            "deferred": true,
        }
    });
    let wait = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-wait"),
        "method": "agent.wait",
        "params": {
            "runId": run_id,
            "timeoutMs": 2000
        }
    });

    let responses = match transport.websocket_exchange(&[connect, chat_send, wait]) {
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
    let queued_status = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let queued_message_is_null = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("message"))
        .is_some_and(Value::is_null);
    let wait_status = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let wait_output = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("output"))
        .and_then(Value::as_str);
    let wait_session_key = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("result"))
        .and_then(|result| result.get("sessionKey"))
        .and_then(Value::as_str);

    if connect_ok
        && queued_status == Some("queued")
        && queued_message_is_null
        && wait_status == Some("completed")
        && wait_output == Some("Echo: conformance deferred chat")
        && wait_session_key == Some(session_key.as_str())
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "deferred chat.send run transitions queued->completed via agent.wait"
                .to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected deferred chat.send lifecycle, found status={queued_status:?}, messageIsNull={queued_message_is_null}, waitStatus={wait_status:?}, waitOutput={wait_output:?}, sessionKey={wait_session_key:?}"
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

fn run_ws_chat_abort_cancels_deferred_chat_send_run<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_abort_cancels_deferred_chat_send_run";
    let run_id = unique_run_id("conformance-chat-abort");
    let session_key = format!("agent:main:{run_id}");

    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let chat_send = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-chat-send"),
        "method": "chat.send",
        "params": {
            "sessionKey": session_key,
            "message": "conformance deferred chat abort",
            "idempotencyKey": run_id,
            "deferred": true,
        }
    });
    let abort = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-abort"),
        "method": "chat.abort",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
        }
    });
    let wait = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-wait"),
        "method": "agent.wait",
        "params": {
            "runId": run_id,
            "timeoutMs": 2000
        }
    });

    let responses = match transport.websocket_exchange(&[connect, chat_send, abort, wait]) {
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
    let queued_status = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("status"))
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
        && queued_status == Some("queued")
        && abort_ok
        && wait_status == Some("aborted")
        && wait_output_is_null
        && wait_session_key == Some(session_key.as_str())
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "chat.abort cancels deferred chat.send run and agent.wait reports aborted"
                .to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected deferred chat.send abort lifecycle, found status={queued_status:?}, aborted={abort_ok}, waitStatus={wait_status:?}, waitOutputIsNull={wait_output_is_null}, sessionKey={wait_session_key:?}"
            ),
        }
    }
}

fn run_ws_chat_abort_session_wide_cancels_deferred_chat_send_runs<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_abort_session_wide_cancels_deferred_chat_send_runs";
    let session_id = unique_run_id("conformance-chat-abort-all");
    let run_id_one = format!("{session_id}-one");
    let run_id_two = format!("{session_id}-two");
    let session_key = format!("agent:main:{session_id}");

    let connect = ws_connect_frame(&format!("{session_id}-connect"));
    let first = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-chat-send-1"),
        "method": "chat.send",
        "params": {
            "sessionKey": session_key,
            "message": "abort all chat one",
            "idempotencyKey": run_id_one,
            "deferred": true,
        }
    });
    let second = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-chat-send-2"),
        "method": "chat.send",
        "params": {
            "sessionKey": session_key,
            "message": "abort all chat two",
            "idempotencyKey": run_id_two,
            "deferred": true,
        }
    });
    let abort = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-abort"),
        "method": "chat.abort",
        "params": {
            "sessionKey": session_key,
        }
    });
    let wait_one = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-wait-1"),
        "method": "agent.wait",
        "params": {
            "runId": run_id_one,
            "timeoutMs": 2000
        }
    });
    let wait_two = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-wait-2"),
        "method": "agent.wait",
        "params": {
            "runId": run_id_two,
            "timeoutMs": 2000
        }
    });

    let responses =
        match transport.websocket_exchange(&[connect, first, second, abort, wait_one, wait_two]) {
            Ok(responses) => responses,
            Err(error) => {
                return ConformanceOutcome {
                    name,
                    passed: false,
                    detail: format!("websocket exchange failed: {error}"),
                };
            }
        };
    if responses.len() != 6 {
        return ConformanceOutcome {
            name,
            passed: false,
            detail: format!("expected 6 websocket responses, found {}", responses.len()),
        };
    }

    let connect_ok = responses[0]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let queued_one = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let queued_two = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let abort_ok = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("aborted"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let abort_ids = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("runIds"))
        .and_then(Value::as_array);
    let wait_one_status = responses[4]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let wait_two_status = responses[5]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);

    let has_run_one = abort_ids.is_some_and(|values| {
        values
            .iter()
            .any(|value| value.as_str() == Some(run_id_one.as_str()))
    });
    let has_run_two = abort_ids.is_some_and(|values| {
        values
            .iter()
            .any(|value| value.as_str() == Some(run_id_two.as_str()))
    });

    if connect_ok
        && queued_one == Some("queued")
        && queued_two == Some("queued")
        && abort_ok
        && has_run_one
        && has_run_two
        && wait_one_status == Some("aborted")
        && wait_two_status == Some("aborted")
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "chat.abort without runId cancels all session deferred chat.send runs"
                .to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected session-wide deferred chat.send abort lifecycle, found queuedOne={queued_one:?}, queuedTwo={queued_two:?}, aborted={abort_ok}, hasRunOne={has_run_one}, hasRunTwo={has_run_two}, waitOne={wait_one_status:?}, waitTwo={wait_two_status:?}"
            ),
        }
    }
}

fn run_ws_chat_abort_session_wide_cancels_runs<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_abort_session_wide_cancels_runs";
    let session_id = unique_run_id("conformance-abort-all");
    let run_id_one = format!("{session_id}-one");
    let run_id_two = format!("{session_id}-two");
    let session_key = format!("agent:main:{session_id}");

    let connect = ws_connect_frame(&format!("{session_id}-connect"));
    let first = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-agent-1"),
        "method": "agent",
        "params": {
            "runId": run_id_one,
            "sessionKey": session_key,
            "agentId": "main",
            "input": "abort all one",
            "deferred": true,
        }
    });
    let second = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-agent-2"),
        "method": "agent",
        "params": {
            "runId": run_id_two,
            "sessionKey": session_key,
            "agentId": "main",
            "input": "abort all two",
            "deferred": true,
        }
    });
    let abort = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-abort"),
        "method": "chat.abort",
        "params": {
            "sessionKey": session_key,
        }
    });
    let wait_one = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-wait-1"),
        "method": "agent.wait",
        "params": {
            "runId": run_id_one,
            "timeoutMs": 2000
        }
    });
    let wait_two = serde_json::json!({
        "type": "req",
        "id": format!("{session_id}-wait-2"),
        "method": "agent.wait",
        "params": {
            "runId": run_id_two,
            "timeoutMs": 2000
        }
    });

    let responses =
        match transport.websocket_exchange(&[connect, first, second, abort, wait_one, wait_two]) {
            Ok(responses) => responses,
            Err(error) => {
                return ConformanceOutcome {
                    name,
                    passed: false,
                    detail: format!("websocket exchange failed: {error}"),
                };
            }
        };
    if responses.len() != 6 {
        return ConformanceOutcome {
            name,
            passed: false,
            detail: format!("expected 6 websocket responses, found {}", responses.len()),
        };
    }

    let connect_ok = responses[0]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let queued_one = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("summary"))
        .and_then(Value::as_str);
    let queued_two = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("summary"))
        .and_then(Value::as_str);
    let abort_ok = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("aborted"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let abort_ids = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("runIds"))
        .and_then(Value::as_array);
    let wait_one_status = responses[4]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let wait_two_status = responses[5]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);

    let has_run_one = abort_ids.is_some_and(|values| {
        values
            .iter()
            .any(|value| value.as_str() == Some(run_id_one.as_str()))
    });
    let has_run_two = abort_ids.is_some_and(|values| {
        values
            .iter()
            .any(|value| value.as_str() == Some(run_id_two.as_str()))
    });

    if connect_ok
        && queued_one == Some("queued")
        && queued_two == Some("queued")
        && abort_ok
        && has_run_one
        && has_run_two
        && wait_one_status == Some("aborted")
        && wait_two_status == Some("aborted")
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "chat.abort without runId cancels all session deferred runs".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected session-wide abort lifecycle, found queuedOne={queued_one:?}, queuedTwo={queued_two:?}, aborted={abort_ok}, hasRunOne={has_run_one}, hasRunTwo={has_run_two}, waitOne={wait_one_status:?}, waitTwo={wait_two_status:?}"
            ),
        }
    }
}

fn run_ws_agent_wait_timeout_for_missing_run<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.agent_wait_timeout_for_missing_run";
    let run_id = unique_run_id("conformance-missing");
    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let wait = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-wait"),
        "method": "agent.wait",
        "params": {
            "runId": run_id,
            "timeoutMs": 50
        }
    });

    let responses = match transport.websocket_exchange(&[connect, wait]) {
        Ok(responses) => responses,
        Err(error) => {
            return ConformanceOutcome {
                name,
                passed: false,
                detail: format!("websocket exchange failed: {error}"),
            };
        }
    };
    if responses.len() != 2 {
        return ConformanceOutcome {
            name,
            passed: false,
            detail: format!("expected 2 websocket responses, found {}", responses.len()),
        };
    }

    let connect_ok = responses[0]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(false);
    let wait_status = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let wait_run_id = responses[1]
        .get("payload")
        .and_then(|payload| payload.get("runId"))
        .and_then(Value::as_str);

    if connect_ok && wait_status == Some("timeout") && wait_run_id == Some(run_id.as_str()) {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "agent.wait returns timeout for unknown run ids".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected timeout for unknown run, found status={wait_status:?}, runId={wait_run_id:?}"
            ),
        }
    }
}

fn run_ws_chat_abort_rejects_run_session_mismatch<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_abort_rejects_run_session_mismatch";
    let run_id = unique_run_id("conformance-mismatch");
    let session_key = format!("agent:main:{run_id}");

    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let agent = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-agent"),
        "method": "agent",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
            "agentId": "main",
            "input": "session mismatch",
            "deferred": true,
        }
    });
    let abort = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-abort"),
        "method": "chat.abort",
        "params": {
            "runId": run_id,
            "sessionKey": format!("{session_key}-other"),
        }
    });

    let responses = match transport.websocket_exchange(&[connect, agent, abort]) {
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
    let abort_ok = responses[2]
        .get("ok")
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let abort_error = responses[2]
        .get("error")
        .and_then(|payload| payload.get("code"))
        .and_then(Value::as_str);

    if connect_ok
        && queued_summary == Some("queued")
        && !abort_ok
        && abort_error == Some("INVALID_REQUEST")
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "chat.abort rejects runId when sessionKey does not match".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected INVALID_REQUEST on mismatched sessionKey, found queued={queued_summary:?}, ok={abort_ok}, code={abort_error:?}"
            ),
        }
    }
}

fn run_ws_chat_abort_completed_run_noop<T: ConformanceTransport>(
    transport: &T,
) -> ConformanceOutcome {
    let name = "ws.chat_abort_completed_run_noop";
    let run_id = unique_run_id("conformance-completed");
    let session_key = format!("agent:main:{run_id}");

    let connect = ws_connect_frame(&format!("{run_id}-connect"));
    let agent = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-agent"),
        "method": "agent",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
            "agentId": "main",
            "input": "complete then abort",
            "deferred": true,
        }
    });
    let wait = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-wait"),
        "method": "agent.wait",
        "params": {
            "runId": run_id,
            "timeoutMs": 2000
        }
    });
    let abort = serde_json::json!({
        "type": "req",
        "id": format!("{run_id}-abort"),
        "method": "chat.abort",
        "params": {
            "runId": run_id,
            "sessionKey": session_key,
        }
    });

    let responses = match transport.websocket_exchange(&[connect, agent, wait, abort]) {
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
    let wait_status = responses[2]
        .get("payload")
        .and_then(|payload| payload.get("status"))
        .and_then(Value::as_str);
    let abort_aborted = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("aborted"))
        .and_then(Value::as_bool)
        .unwrap_or(true);
    let abort_run_ids = responses[3]
        .get("payload")
        .and_then(|payload| payload.get("runIds"))
        .and_then(Value::as_array);
    let run_id_present = abort_run_ids.is_some_and(|values| {
        values
            .iter()
            .any(|value| value.as_str() == Some(run_id.as_str()))
    });

    if connect_ok
        && queued_summary == Some("queued")
        && wait_status == Some("completed")
        && !abort_aborted
        && run_id_present
    {
        ConformanceOutcome {
            name,
            passed: true,
            detail: "chat.abort is a no-op for completed runs".to_owned(),
        }
    } else {
        ConformanceOutcome {
            name,
            passed: false,
            detail: format!(
                "expected completed-run abort no-op, found queued={queued_summary:?}, wait={wait_status:?}, aborted={abort_aborted}, runIdPresent={run_id_present}"
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
