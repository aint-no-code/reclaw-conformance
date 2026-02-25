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
