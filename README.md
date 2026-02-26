# Reclaw Conformance

`reclaw-conformance` verifies runtime behavior against Reclaw/OpenClaw protocol invariants.

## Current Scenarios

- `healthz.ok_true`: `/healthz` must return `{ "ok": true }`
- `readyz.ok_true`: `/readyz` must return `{ "ok": true }`
- `info.protocol_version`: `/info` must include `protocolVersion == 3`
- `info.methods_include_health_status`: `/info` must expose method list entries for `health` and `status`
- `channels.unknown_webhook_not_found`: unknown channel webhooks must return HTTP `404` with `error.code == "NOT_FOUND"`
- `ws.handshake_requires_connect_first_frame`: WS gateway must reject a non-`connect` first request with `INVALID_REQUEST`
- `ws.agent_deferred_wait_completes`: deferred `agent` runs must transition `queued -> completed` through `agent.wait`
- `ws.chat_abort_cancels_deferred_run`: `chat.abort` must cancel deferred runs and `agent.wait` must report `aborted`

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
