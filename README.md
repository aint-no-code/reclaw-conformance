# Reclaw Conformance

`reclaw-conformance` verifies runtime behavior against Reclaw/OpenClaw protocol invariants.

## Current Scenarios

- `healthz.ok_true`: `/healthz` must return `{ "ok": true }`
- `info.protocol_version`: `/info` must include `protocolVersion == 3`

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
