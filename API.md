# API reference (v1)

The machine-readable contract is [`openapi.yaml`](openapi.yaml). The API is served by the dashboard binary in `--web` mode and is intended for the local machine only.

## Base URL

```
http://127.0.0.1:3000/api/v1
```

## Endpoints

| Method | Path | Purpose |
|---|---|---|
| GET | `/usage` | Full usage snapshot: totals, comparisons, per-day, sessions, models, projects |
| GET | `/usage/daily?range=30d\|all` | Daily rows, newest first |
| GET | `/usage/sessions?q=&sort=&dir=` | Filtered and sorted sessions |
| GET | `/usage/costs` | Cost estimates from `pricing.json` |
| GET | `/settings` | Current UI settings |
| PUT | `/settings` | Replace settings (idempotent) |

Health checks live outside the versioned prefix: `GET /health/live` and `GET /health/ready`.

## Examples

```sh
curl http://127.0.0.1:3000/api/v1/usage
curl 'http://127.0.0.1:3000/api/v1/usage/sessions?q=greeting&sort=total&dir=desc'
curl http://127.0.0.1:3000/api/v1/usage/costs
curl -X PUT http://127.0.0.1:3000/api/v1/settings \
  -H 'Content-Type: application/json' \
  -d '{"autoSyncSeconds":15,"allDays":false}'
```

## Errors

All failures use the same shape:

```json
{"error":{"code":"invalid_settings","message":"autoSyncSeconds must be one of 0, 1, 5, 15 or 60"}}
```

- `400`/`422` — invalid query or body
- `500` — database or configuration error (`database_error`, `write_error`)
- `503` — health readiness check failed

## Configuration files

Both live in the per-OS config directory (`~/.config/opencode-token-dashboard` on Linux, overridable with `OTD_CONFIG_DIR`):

- `config.json` — `{"autoSyncSeconds":1,"allDays":false}`
- `pricing.json` — rates in USD per million tokens, keyed by model id:

```json
{
  "models": {
    "provider/model": {
      "input": 2.0,
      "output": 8.0,
      "cache_read": 0.5,
      "cache_write": 0.0
    }
  }
}
```

No prices are bundled or fetched; the app only uses rates you supply.

## Observability

Every response carries `x-request-id`. Request logs contain method, path, status and duration — never request bodies or credentials.
