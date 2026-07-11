# fiber-diagnostics API

Base URL (local dev): `http://127.0.0.1:3000`

The API serves an **in-memory snapshot**, rebuilt from scratch every ~5s poll
cycle (see `generated_at` below to know how fresh it is). It never queries
FNN directly and never computes anything per-request — it just reads
whatever the background poller + diagnostics engine last produced.

## GET /issues

Returns every currently-detected issue across all monitored nodes.

**Query params (optional, both combinable):**

| param | type | example | effect |
|---|---|---|---|
| `kind` | string | `?kind=NODE_DOWN` | exact match against `issue.kind` (case-sensitive, see Known Issues below) |
| `severity` | string | `?severity=critical` | case-insensitive match against `issue.severity` |

**Response `200`:**
```json
{
  "generated_at": "2026-07-10T14:32:01.123Z",
  "count": 2,
  "issues": [
    {
      "kind": "node-down",
      "severity": "Critical",
      "node_id": "badnode",
      "description": "Node badnode is unreachable through RPC"
    },
    {
      "kind": "channel-not-ready",
      "severity": "Warning",
      "node_id": "node1",
      "description": "Channel 0xdb4a... is not ready: state=NegotiatingFunding, enabled=true"
    }
  ]
}
```

- `generated_at` — when this snapshot was assembled (ISO 8601, UTC). Use this
  to show "updated Ns ago" or to detect a stalled backend (if it stops
  advancing, the poller loop has died).
- `count` — `issues.length`, provided so you don't need to re-derive it.
- No issues matching the filters (or no issues at all) → `count: 0`,
  `issues: []`, still a `200`, not an error.

**Response, error case (`5xx`):**
```json
{ "error": "human-readable message" }
```
Always JSON, same shape regardless of status code. Currently unreachable in
practice (reading an in-memory cache doesn't fail) — reserved for when a
real failure mode exists.

## GET /issues/{kind}

Same as `GET /issues`, pre-filtered to one `kind`. Equivalent to
`GET /issues?kind={kind}`, kept as a separate route for convenience/nicer
URLs. Same query params (`severity`) still apply on top of the path filter.

```
GET /issues/NODE_DOWN
GET /issues/channel-not-ready?severity=warning
```

## Issue shape

| field | type | notes |
|---|---|---|
| `kind` | string | one of the 8 values below. **Casing is inconsistent right now** — see Known Issues. |
| `severity` | string | one of `"Critical"`, `"Warning"`, `"Info"` (exact casing, PascalCase) |
| `node_id` | string | which monitored node this issue belongs to |
| `description` | string | human-readable, safe to display directly |

## All `kind` values (as of today — exact strings, case-sensitive)

| kind | category | confirmed with |
|---|---|---|
| `NODE_DOWN` | node unreachable via RPC | real data |
| `channel-not-ready` | channel not in ChannelReady state, or disabled | real data |
| `insufficient-balance` | channel local balance too low relative to remote | real data |
| `invoice-expired` | invoice past expiry (by status or by timestamp) | real + synthetic |
| `no-route` | payment failed, no route found | synthetic |
| `fee-too-low` | payment failed, fee below requirement | synthetic |
| `asset-mismatch` | payment asset type ≠ invoice currency | synthetic |
| `peer-offline` | peer disconnected | **not yet observable — see Known Issues** |

## Known issues / things not yet fixed (as of today)

- **`kind` casing is inconsistent** (`NODE_DOWN` vs `channel-not-ready`)
  and matching is case-sensitive. Don't assume a normalized convention yet —
  use the exact strings in the table above. This will likely change to a
  single consistent casing in a future update; if/when it does, this doc
  gets updated first.
- **`peer-offline` can never fire right now.** The poller always writes
  `connected=1` and never sets it to `0` when a peer drops — a backend bug,
  not a frontend concern, but don't spend time debugging a "peer offline"
  UI against live data until this is fixed upstream.
- **CORS is not yet configured** on the server. If you're running the
  dashboard on a different origin/port (which you will be, in dev), browser
  `fetch` calls to this API will currently be blocked. This needs a fix on
  the backend before cross-origin requests will work — flag if you hit this
  before it's resolved.
- **`GET /issues/{unknown-kind}`** (a kind that doesn't exist) currently
  returns `200` with an empty `issues: []`, not a `404`. Don't treat an
  empty result as confirmation the kind string you used is correct — typos
  fail silently.
- `no-route`, `fee-too-low`, `asset-mismatch` are only proven against
  hand-inserted synthetic rows so far, not real failed payments yet (the
  payment-tracking poller that would populate these from real activity is
  still in progress).

## Suggested way to work against this without waiting on the live backend

Ask for (or generate) a static fixture JSON containing one example of each
`kind` above, matching this exact response shape. Build the dashboard UI
against that fixture first — decouples your work from needing two live FNN
nodes running, and from `peer-offline` being permanently empty. Swap the
fixture for the real `fetch('http://127.0.0.1:3000/issues')` call once the
UI is built.
