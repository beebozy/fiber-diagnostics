# fiber-diagnostics

A diagnostics backend for the CKB Fiber Network (FNN) — polls live Fiber nodes, classifies failures across 8 categories (node, peer, channel, liquidity, invoice, routing, fee/policy, and asset-mismatch conditions), and exposes them as a clean JSON API for a dashboard to consume.

FNN's `get_payment` RPC returns failures as free-text (`failed_error: Option<String>`), not structured error codes — there's no enum to switch on for "why did this fail." This project turns that into a rule-based classifier backed by real polled state, not guesswork.

**Submission category:** Category 2 — Node, Routing, Cross-Chain, and Diagnostics Infrastructure

---

## Project overview

`fiber-diagnostics` is an infrastructure tool for FNN node operators. It continuously polls one or more Fiber nodes over RPC, persists their state (node reachability, peers, channels, in-flight payments, invoices) to SQLite, and runs an ordered rule engine over that state to surface plain-English diagnostic issues — instead of requiring an operator to manually cross-reference `list_channels`, `list_peers`, `get_payment`, and raw logs to figure out why something is broken.

**Target audience:** Fiber node operators and developers building on top of Fiber who need to know why a payment, channel, or peer connection is failing, without reading verbose `RUST_LOG=info` output or making a dozen manual RPC calls per incident.

---

## What problem does it solve?

Diagnosing a Fiber node issue today means manually correlating several RPC calls (`node_info`, `list_peers`, `list_channels`, `get_payment`, `get_invoice`, the gossip graph) plus raw logs, with no single place that says "here's what's wrong and why." FNN's lack of structured payment error codes makes this worse — `failed_error` is just a string, so any tool built on top of it has to be a rule engine, not a lookup table.

`fiber-diagnostics` closes that gap: a background poller keeps a live picture of node/peer/channel/payment/invoice state in a local database, and an ordered rule engine (deterministic checks first, string-matching last, per Fiber's own recommended approach) turns that state into concrete, categorized issues exposed over a REST API — the foundation a dashboard or CLI can build on without re-implementing any diagnostic logic itself.

---

## System design

### Two-team split, one shared SQLite schema as the contract

| Owner | Responsibility |
|---|---|
| **Data collection** | `FiberRpcClient` (all RPC methods), the node registry (`monitored_nodes`), the fast/slow polling loops, per-node failure isolation (a down node doesn't crash the poll cycle — it *is* the signal), `poller_runs` logging |
| **Diagnostics & API** | The ordered rule engine (8 categories), payment/invoice tracking, the axum REST API, response contract design |

### Runtime flow

```
Fiber Nodes (node1, node2, ...)
        |
        | JSON-RPC (FiberRpcClient)
        |
   ┌────┴─────────────────────────┐
   │                              │
Fast loop (5s)                Slow loop (30s)
node_info, list_peers,        graph_nodes,
list_channels                 graph_channels
   │                              │
   └────┬─────────────────────────┘
        |
   SQLite (*_current tables)
        |
   ┌────┴─────────────────────────┐
   │                              │
payment_tracker (10s)         Diagnostics engine
polls tracked_payments        (runs at end of each
via get_payment/get_invoice   fast-loop tick)
   │                              │
   └──────────────┬───────────────┘
                  |
         8 ordered rules -> Vec<Issue>
                  |
          In-memory issue cache
                  |
     axum REST API (CORS-enabled)
     GET /issues, /issues/{kind}
     POST /payments (originate + auto-track)
                  |
          Dashboard / CLI
```

### Key design decisions

- **In-memory issue cache, not persisted issue history.** The API serves whatever the last poll pass computed; there's no "resolved" state to manage — an issue simply stops appearing once its underlying condition clears on the next pass. Chosen for simplicity given the timeline; a persisted, deduplicated issue history (with acknowledge/mute states) was prototyped and works, but was deliberately descoped in favor of shipping.
- **Rule engine, not a lookup table**, per FNN's own unstructured `failed_error` string — rules are ordered deterministic checks (channel/peer state, computed balance thresholds) before falling back to `failed_error` substring matching, which is inherently approximate.
- **Amounts and UDT scripts are not plain JSON.** FNN serializes `u128`/`u64` amounts as hex strings, and an invoice's requested UDT is a raw CKB Molecule-encoded binary blob inside the invoice's `attrs`, not a JSON object. Both were discovered and handled explicitly (see [Current functionality](#current-functionality)) rather than assumed.

---

## Setup environment

**Stack:** Rust (2021 edition), axum 0.8 for the HTTP API, sqlx 0.8 (SQLite, runtime-checked queries) for persistence, tokio for the async runtime, reqwest for the FNN JSON-RPC client, tower-http for CORS, chrono/uuid/anyhow/thiserror/tracing for the usual supporting concerns.

**Prerequisites:** a running `fnn` process (or several) — see [Run a Fiber Node](#) or the [Docker guide](#).

### Local setup

```bash
export DATABASE_URL="sqlite://fiber-diagnostics.db"
cargo run
```

Migrations apply automatically on startup (`sqlx::migrate!`). The API listens on `http://127.0.0.1:3000`; register nodes to monitor with a plain insert — no code change required to add a node:

```bash
sqlite3 fiber-diagnostics.db "INSERT INTO monitored_nodes (id, name, rpc_url, created_at, updated_at) VALUES ('node1','node1','http://127.0.0.1:8227','<now>','<now>');"
```

See [`docs/API.md`](docs/API.md) for the full endpoint contract and [`docs/TESTING.md`](docs/TESTING.md) for a rule-by-rule testing runbook (both synthetic and real-node testing procedures).

---

## Tooling

- **FNN JSON-RPC** — `node_info`, `list_peers`, `list_channels`, `graph_nodes`, `graph_channels`, `send_payment`, `get_payment`, `parse_invoice`, `get_invoice`, `new_invoice`, `connect_peer`, `disconnect_peer`, `open_channel` (via a hand-rolled `FiberRpcClient`, no external Fiber SDK dependency).
- **CKB Molecule decoding** — a minimal hand-written decoder for the Script table format (`code_hash: Byte32`, `hash_type: byte`, `args: Bytes`), used to extract UDT identity from an invoice's binary-encoded `attrs`. Verified byte-for-byte against a real captured `get_invoice` response (see the pinned unit test in `asset_mismatch.rs`).
- **sqlx** with runtime-checked queries (`query_as`) rather than the `sqlx::query!` macro family — deliberate, to avoid requiring a reachable `DATABASE_URL` or a committed offline query cache at compile time while the schema was still moving.

---

## Current functionality

All 8 diagnostic categories are implemented. Status reflects actual testing performed, not aspiration:

| Category | Status |
|---|---|
| **Node Failure** (`node-down`) | Confirmed against real unreachable nodes |
| **Channel Readiness** (`channel-not-ready`) | Confirmed against real channel state (a wrong-constant bug was found and fixed here) |
| **Liquidity Failure** (`insufficient-balance`) | Confirmed against real channel balances (hex-string amount parsing bug found and fixed) |
| **Invoice Failure** (`invoice-expired`) | Confirmed via both real and synthetic invoices (hex-encoded expiry timestamp parsing fixed) |
| **Peer Connectivity** (`peer-offline`) | Confirmed real-time — poller now marks peers disconnected as they actually disconnect (previously hardcoded always-connected) |
| **Policy/Fee Failure** (`fee-too-low`) | Rule logic confirmed via synthetic failed-payment data |
| **Routing Failure** (`no-route`) | Rule logic confirmed via synthetic data; known gap — FNN's synchronous pre-flight pathfinding rejection (no `payment_hash` ever generated) isn't yet captured by the pipeline, only post-dispatch routing failures are |
| **Asset Mismatch** (`asset-mismatch`) | Redesigned after discovering the invoice's UDT is Molecule-encoded binary, not JSON — decoder is unit-tested against a real captured response |

**Also implemented:**

- REST API: `GET /issues` (with `kind`/`severity` filters), `GET /issues/{kind}`, `POST /payments` (originates a real payment and auto-registers it for tracking — no manual DB step required)
- CORS enabled for browser-based dashboard consumption
- JSON error responses and a `generated_at`/`count` wrapper on every response, so a dashboard can show data freshness

---

## Future functionality

- **Persisted issue history** (`issues_current`/`issue_events` with open/acknowledged/muted/resolved states) — prototyped and proven to work during development, descoped from the shipped version in favor of a simpler in-memory cache given the timeline.
- **Close the pre-flight routing-failure gap** — capture `send_payment`'s synchronous RPC error (not just post-dispatch failures) so `no-route` is fully provable end-to-end, not just via synthetic data.
- **CLI** (`fiber-diagnose watch/check/payment/channel/peers/node`) — the engine is already structured so the CLI would be a pure formatter on top of it, no diagnostic logic duplicated.
- **Dashboard** — the API contract (`docs/API.md`) is written and stable; the frontend itself is in progress.
- **Verify the channel-side UDT assumption** — `asset-mismatch` assumes `list_channels`' `funding_udt_type_script` comes back as a plain JSON object (standard CKB RPC convention); confirmed correct for native-CKB channels, not yet confirmed against a real UDT-funded channel.
- **`fee-too-low`/`no-route` real-node proof** beyond synthetic data, and broader `max_fee_amount` support on `POST /payments`.

---

## Frontend setup

*(coming soon)*

---

## Video link

*(add link here)*

## Hosted setup

*(add link here)*

## Screenshots

*(add screenshots here)*
