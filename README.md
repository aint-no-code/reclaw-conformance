# Reclaw Conformance

`reclaw-conformance` verifies runtime behavior against Reclaw/OpenClaw protocol invariants.

## Current Scenarios

- `healthz.ok_true`: `/healthz` must return `{ "ok": true }`
- `readyz.ok_true`: `/readyz` must return `{ "ok": true }`
- `info.protocol_version`: `/info` must include `protocolVersion == 3`
- `info.methods_include_health_status`: `/info` must expose method list entries for `health` and `status`
- `channels.unknown_webhook_not_found`: unknown channel webhooks must return HTTP `404` with `error.code == "NOT_FOUND"`
- `tools.invoke_gateway_request`: `/tools/invoke` with `gateway.request` health dispatch must return `ok == true` and `result.ok == true`
- `tools.invoke_gateway_request_action_fallback`: `/tools/invoke` with `gateway.request` and top-level `action` fallback must dispatch successfully
- `tools.invoke_rejects_unknown_tool`: `/tools/invoke` must reject unknown tools with HTTP `404` and `error.type == "not_found"`
- `ws.handshake_requires_connect_first_frame`: WS gateway must reject a non-`connect` first request with `INVALID_REQUEST`
- `ws.channels_status_includes_account_views`: `channels.status` must expose account-aware summary views (`channelsById`, `channelAccounts`, `channelDefaultAccountId`)
- `ws.channels_logout_account_persists`: `channels.logout` with `accountId` must persist account-specific disconnected state
- `ws.agent_deferred_wait_completes`: deferred `agent` runs must transition `queued -> completed` through `agent.wait`
- `ws.chat_send_deferred_wait_completes`: deferred `chat.send` runs must return `queued` and complete through `agent.wait`
- `ws.chat_abort_cancels_deferred_run`: `chat.abort` must cancel deferred runs and `agent.wait` must report `aborted`
- `ws.chat_abort_cancels_deferred_chat_send_run`: `chat.abort` must cancel deferred `chat.send` runs and `agent.wait` must report `aborted`
- `ws.chat_abort_session_wide_cancels_deferred_chat_send_runs`: `chat.abort` without `runId` must cancel all deferred `chat.send` runs in a session
- `ws.chat_abort_session_wide_cancels_runs`: `chat.abort` without `runId` must cancel all non-terminal runs for the session
- `ws.agent_wait_timeout_for_missing_run`: `agent.wait` for unknown runs must return `status == "timeout"`
- `ws.chat_abort_rejects_run_session_mismatch`: `chat.abort` must reject `runId` cancellation when `sessionKey` does not match
- `ws.chat_abort_completed_run_noop`: `chat.abort` on completed runs must return `aborted == false`

## Run

```bash
cargo run -- --base-url http://127.0.0.1:18789 --json
```

## Quality Gates

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features
```
