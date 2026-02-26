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

## `ws.channels_status_includes_account_views`

- Surface: WebSocket `/ws`
- Requirement: `channels.status` response includes `channels`, `channelOrder`, `channelLabels`
- Requirement: response includes account-aware views: `channelsById`, `channelAccounts`, `channelDefaultAccountId`
- Requirement: default `webchat` account is represented as `channelDefaultAccountId.webchat == "default"`

## `tools.invoke_gateway_request`

- Endpoint: `POST /tools/invoke`
- Requirement: `tool == "gateway.request"` with `args.method == "health"` returns HTTP `200`
- Requirement: response includes `{ "ok": true, "result": { "ok": true } }`

## `tools.invoke_gateway_request_action_fallback`

- Endpoint: `POST /tools/invoke`
- Requirement: `tool == "gateway.request"` with `action == "health"` and empty `args` returns HTTP `200`
- Requirement: response includes `{ "ok": true, "result": { "ok": true } }`

## `ws.channels_logout_account_persists`

- Surface: WebSocket `/ws`
- Requirement: `channels.logout` accepts `accountId` and returns it in payload
- Requirement: subsequent `channels.status` includes the account in `channelAccounts`
- Requirement: disconnected account does not force aggregated channel summary disconnected if other accounts remain connected

## `ws.agent_deferred_wait_completes`

- Surface: WebSocket `/ws`
- Requirement: `agent` with `deferred=true` returns queued summary
- Requirement: `agent.wait` returns `status == "completed"`
- Requirement: wait payload includes completed `result.output` and `result.sessionKey`

## `ws.chat_send_deferred_wait_completes`

- Surface: WebSocket `/ws`
- Requirement: `chat.send` with `deferred=true` returns `status == "queued"` and `message == null`
- Requirement: `agent.wait` on the idempotency run id returns `status == "completed"`
- Requirement: wait payload includes completed `result.output` and `result.sessionKey`

## `ws.chat_abort_cancels_deferred_run`

- Surface: WebSocket `/ws`
- Requirement: queued deferred run can be canceled by `chat.abort`
- Requirement: `agent.wait` returns `status == "aborted"`
- Requirement: aborted wait payload keeps `result.output == null`

## `ws.chat_abort_cancels_deferred_chat_send_run`

- Surface: WebSocket `/ws`
- Requirement: deferred `chat.send` run can be canceled by `chat.abort`
- Requirement: `agent.wait` returns `status == "aborted"`
- Requirement: aborted wait payload keeps `result.output == null`

## `ws.chat_abort_session_wide_cancels_deferred_chat_send_runs`

- Surface: WebSocket `/ws`
- Requirement: `chat.abort` without `runId` cancels all deferred `chat.send` runs in the same session
- Requirement: abort response `runIds` includes all canceled run ids
- Requirement: `agent.wait` on each canceled run returns `status == "aborted"`

## `ws.chat_abort_session_wide_cancels_runs`

- Surface: WebSocket `/ws`
- Requirement: `chat.abort` without `runId` cancels all queued/running runs for the session
- Requirement: abort response `runIds` includes all canceled runs
- Requirement: `agent.wait` on each canceled run returns `status == "aborted"`

## `ws.agent_wait_timeout_for_missing_run`

- Surface: WebSocket `/ws`
- Requirement: `agent.wait` on unknown `runId` returns `status == "timeout"`
- Requirement: timeout payload echoes the requested `runId`

## `ws.chat_abort_rejects_run_session_mismatch`

- Surface: WebSocket `/ws`
- Requirement: aborting by `runId` with a different `sessionKey` fails
- Requirement: response is `ok == false` with `error.code == "INVALID_REQUEST"`

## `ws.chat_abort_completed_run_noop`

- Surface: WebSocket `/ws`
- Requirement: once `agent.wait` reports `completed`, subsequent `chat.abort` is a no-op
- Requirement: abort response is `ok == true`, `aborted == false`, with `runIds` including the target run
