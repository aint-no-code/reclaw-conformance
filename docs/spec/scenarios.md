# Conformance Scenarios

## `healthz.ok_true`

- Endpoint: `GET /healthz`
- Requirement: HTTP `200`
- Requirement: body includes `{ "ok": true }`

## `info.protocol_version`

- Endpoint: `GET /info`
- Requirement: HTTP `200`
- Requirement: body includes numeric `protocolVersion`
- Requirement: `protocolVersion == 3`

## `ws.agent_deferred_wait_completes`

- Surface: WebSocket `/ws`
- Requirement: `agent` with `deferred=true` returns queued summary
- Requirement: `agent.wait` returns `status == "completed"`
- Requirement: wait payload includes completed `result.output` and `result.sessionKey`

## `ws.chat_abort_cancels_deferred_run`

- Surface: WebSocket `/ws`
- Requirement: queued deferred run can be canceled by `chat.abort`
- Requirement: `agent.wait` returns `status == "aborted"`
- Requirement: aborted wait payload keeps `result.output == null`
