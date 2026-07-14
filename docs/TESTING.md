# Testing the diagnostic rules

Two levels of testing, and they prove different things:

- **Part 1 (below): synthetic DB rows.** Proves the *rule logic* is correct.
  Fast, but doesn't touch FNN, doesn't prove the RPC calls work, doesn't
  prove our own code parses real responses correctly.
- **Part 2 (further down): real running nodes.** Proves the *whole pipeline*
  — real RPC call → our poller's parsing → DB → rule → API. Slower, but
  it's the only thing that actually proves the tool works.


## Before you start

- First run -- sqlite3 fiber-diagnostics.db < seed.sql
- The API serves an **in-memory cache**, rebuilt from a full DB scan every
  ~5s (the fast poll loop). After changing the DB, **wait ~5s**, then curl.
- An issue disappears from `/issues` only when you fix or delete the
  underlying row — there's no separate "resolved" action, the next scan
  just won't find it anymore.
- Every synthetic ID below is prefixed `test-` on purpose, so the teardown
  script at the bottom can clean everything up in one shot without touching
  real data from your actual nodes.
- Pretty-print responses if useful: `curl ... | python3 -m json.tool`

Make sure `cargo run` is already running in another terminal before any of
this — these commands only insert data; the poll/diagnostics loop is what
turns it into an issue.

---

## 1. node-down

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO monitored_nodes
  (id, name, rpc_url, created_at, updated_at)
VALUES ('test-badnode','unreachable test','http://127.0.0.1:1','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/node-down
```

Expect one issue for `test-badnode`. This one goes through the *real* poll
path (not a synthetic status row) — the poller will genuinely try and fail
to reach `http://127.0.0.1:1`.

---

## 2. channel-not-ready

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO channel_status_current
  (node_id, channel_id, peer_pubkey, is_public, state_name, enabled,
   local_balance_raw, remote_balance_raw, offered_tlc_balance_raw, received_tlc_balance_raw,
   last_seen_at, updated_at)
VALUES ('node1','test-channel-negotiating','test-peer',0,'NegotiatingFunding',1,
   '0x0','0x0','0x0','0x0','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/channel-not-ready
```

Expect an issue for `test-channel-negotiating`. Safe from being overwritten
by the real poller — it only upserts channels it actually sees in
`list_channels`, it never deletes rows for channels that don't exist, so a
made-up `channel_id` persists undisturbed.

You already have two real `NegotiatingFunding` channels and multiple real
`ChannelReady` ones from earlier testing — a quick sanity re-check that
`ChannelReady` channels are *absent* from this list is worth doing after any
change to `channel_not_ready.rs`:
```bash
curl http://127.0.0.1:3000/issues/channel-not-ready | python3 -m json.tool
# every state= in the output should be NegotiatingFunding, never ChannelReady
```

---

## 3. insufficient-balance

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO channel_status_current
  (node_id, channel_id, peer_pubkey, is_public, state_name, enabled,
   local_balance_raw, remote_balance_raw, offered_tlc_balance_raw, received_tlc_balance_raw,
   last_seen_at, updated_at)
VALUES ('node1','test-channel-low-balance','test-peer',0,'ChannelReady',1,
   '0x0','0x174876e800','0x0','0x0','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/insufficient-balance
```

`0x174876e800` = 100,000,000,000 remote, local = 0 → clearly below
`remote/10`, should fire. Note `state_name='ChannelReady'` here on purpose —
proves this rule triggers independently of channel readiness, not as a side
effect of the channel also being flagged not-ready.

---

## 4. invoice-expired

```bash
# 0x5e0be100 = 2020-01-01T00:00:00Z, well in the past
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO payment_status_current
  (payment_hash, node_id, status, observed_at, updated_at)
VALUES ('test-invoice-expired','node1','Created','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO invoice_status_current
  (payment_hash, invoice_status, expiry_time_raw, observed_at, updated_at)
VALUES ('test-invoice-expired','Open','0x5e0be100','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/invoice-expired
```

`invoice_status` is deliberately `'Open'`, not `'Expired'` — this only fires
through the hex-timestamp parsing path, which is the part we fixed earlier.
If this comes back empty, that specific fix has regressed.

---

## 5. no-route

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO payment_status_current
  (payment_hash, node_id, status, failed_error, observed_at, updated_at)
VALUES ('test-payment-no-route','node1','Failed','PaymentError: no route found to destination','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/no-route
```

---

## 6. fee-too-low

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO payment_status_current
  (payment_hash, node_id, status, failed_error, observed_at, updated_at)
VALUES ('test-payment-fee-too-low','node1','Failed','PaymentError: fee too low for route','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/fee-too-low
```

---

## 7. asset-mismatch

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO payment_status_current
  (payment_hash, node_id, asset_type, status, observed_at, updated_at)
VALUES ('test-payment-asset-mismatch','node1','CKB','Inflight','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO invoice_status_current
  (payment_hash, currency, invoice_status, observed_at, updated_at)
VALUES ('test-payment-asset-mismatch','USDI','Open','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/asset-mismatch
```

---

## 8. peer-offline — currently CANNOT be end-to-end tested

This is the one open bug: the real poller always writes `connected=1` and
never sets it to `0` when a peer actually disconnects (`main.rs`'s
`peer_status_current` insert is hardcoded `VALUES (?, ?, ?, 1, ...)`).
Nothing you do from the API/DB side proves the *detection* path works,
because the poller can never produce the input that would trigger it.

You can still confirm the **rule logic itself** is correct (separately from
proving the real pipeline works) by inserting a synthetic disconnected peer
directly:

```bash
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO peer_status_current
  (node_id, peer_pubkey, address, connected, last_seen_at, updated_at)
VALUES ('node1','test-peer-offline','127.0.0.1:9000',0,'2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sleep 6
curl http://127.0.0.1:3000/issues/peer-offline
```

If this returns the issue, `peer_offline.rs` itself is fine — the bug is
entirely upstream in the poller, not in this rule. Worth doing once just to
isolate that, then flagging the poller fix to Habeeb rather than debugging
this rule further.

---

## Teardown — remove all synthetic test rows in one shot

```bash
sqlite3 fiber-diagnostics.db "
DELETE FROM monitored_nodes WHERE id LIKE 'test-%';
DELETE FROM node_status_current WHERE node_id LIKE 'test-%';
DELETE FROM channel_status_current WHERE channel_id LIKE 'test-%';
DELETE FROM peer_status_current WHERE peer_pubkey LIKE 'test-%';
DELETE FROM payment_status_current WHERE payment_hash LIKE 'test-%';
DELETE FROM invoice_status_current WHERE payment_hash LIKE 'test-%';
"
sleep 6
curl http://127.0.0.1:3000/issues
```

Should return only issues from your real nodes/channels again (or `count: 0`
if everything's currently healthy).

---

# Part 2 — Testing against your real running nodes

General pattern for every category below:

1. Do something real to `node1`/`node2` (kill a process, call FNN's RPC
   directly with `curl`) — this is the actual condition, not a fake row.
2. For **node/peer/channel** categories: our own poller picks this up
   automatically within 5s. Nothing else to do.
3. For **payment/invoice** categories: our poller doesn't discover payments
   on its own (there's no "watch everything" mode) — you still need one
   manual step, registering the real `payment_hash` into `tracked_payments`,
   so our `payment_tracker.rs` (its own 10s loop) picks it up and polls the
   real `get_payment`/`get_invoice` results. That's a legitimate part of the
   real pipeline, not a shortcut — it's just how "tell our tool what to
   watch" currently works, since there's no discovery mechanism yet.
4. Then `curl http://127.0.0.1:3000/issues/<kind>` as before.

Ports, per your actual setup: `node1` → `127.0.0.1:8227`, `node2` →
`127.0.0.1:8237`.

## 1. node-down (real)

```bash
# find and kill node2's real fnn process
ps aux | grep fnn
kill <node2's pid>

sleep 6
curl http://127.0.0.1:3000/issues/node-down
```

Restart node2 the way you normally do, wait 6s, confirm the issue is gone
from `/issues`.

## 2. peer-offline (real) — this will PROVE the known poller bug, not fix it

```bash
# get node2's peer_id from node1's list_peers first
curl -s -X POST http://127.0.0.1:8227 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_peers","params":[]}' | python3 -m json.tool

# then disconnect it from node1's side
curl -s -X POST http://127.0.0.1:8227 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"disconnect_peer","params":[{"peer_id":"<peer_id from above>"}]}'

sleep 6
curl http://127.0.0.1:3000/issues/peer-offline
```

**Expected result: empty, even though the peer really is disconnected.**
That's not a passing test — it's real-world confirmation of the bug we
already know about (`peer_status_current.connected` is hardcoded `1` in the
poller). This is worth doing once specifically to show Habeeb concrete
proof, not just "the rule doesn't work."

## 3. channel-not-ready (real)

You likely already have real `NegotiatingFunding` channels sitting there.
Check current real state directly against the node first:

```bash
curl -s -X POST http://127.0.0.1:8227 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"list_channels","params":[{}]}' | python3 -m json.tool
```

If nothing's currently `NegotiatingFunding`, open a fresh channel and check
again within the next few seconds (before it confirms):

```bash
curl -s -X POST http://127.0.0.1:8227 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":2,"method":"open_channel","params":[{"peer_id":"<node2 peer_id>","funding_amount":"0xba43b7400","public":true}]}'

sleep 6
curl http://127.0.0.1:3000/issues/channel-not-ready
```

## 4. insufficient-balance (real)

Check current real balances first — you already have real low-balance
channels from earlier testing:

```bash
curl http://127.0.0.1:3000/issues/insufficient-balance
```

To force a *new* low-balance condition for real, drain one side with actual
payments (see #5 below) and re-check.

## 5. invoice-expired (real)

```bash
# generate a random 32-byte preimage
PREIMAGE="0x$(openssl rand -hex 32)"

# create a real invoice on node2, expiring in 1 second (0x1 = 1 second hex)
curl -s -X POST http://127.0.0.1:8237 -H "Content-Type: application/json" \
  -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"new_invoice\",\"params\":[{\"amount\":\"0x5f5e100\",\"currency\":\"Fibt\",\"description\":\"expiry test\",\"expiry\":\"0x1\",\"final_cltv\":\"0x28\",\"payment_preimage\":\"$PREIMAGE\",\"hash_algorithm\":\"sha256\"}]}" | python3 -m json.tool
# note the payment_hash from the response
```

Wait a couple seconds for it to actually expire, then register it so our
tracker picks it up:

```bash
sqlite3 fiber-diagnostics.db "INSERT INTO tracked_payments
  (payment_hash, node_id, tracking_status, created_at, updated_at)
VALUES ('<payment_hash from above>', 'node2', 'active', '2026-07-11T00:00:00Z', '2026-07-11T00:00:00Z');"

sleep 11   # payment_tracker's loop is every 10s
curl http://127.0.0.1:3000/issues/invoice-expired
```

## 6. no-route (real)

Send a real payment toward a target with no path (e.g. a made-up pubkey not
in your two-node graph):

```bash
curl -s -X POST http://127.0.0.1:8227 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"send_payment","params":[{"target_pubkey":"0000000000000000000000000000000000000000000000000000000000000000","amount":"0x5f5e100"}]}'
# note the payment_hash

sqlite3 fiber-diagnostics.db "INSERT INTO tracked_payments (payment_hash, node_id, tracking_status, created_at, updated_at) VALUES ('<payment_hash>', 'node1', 'active', '2026-07-11T00:00:00Z', '2026-07-11T00:00:00Z');"

sleep 11
curl http://127.0.0.1:3000/issues/no-route
```

## 7. fee-too-low (real)

Same as above, but against a real invoice from node2, with `max_fee_amount`
set unreasonably low:

```bash
curl -s -X POST http://127.0.0.1:8227 -H "Content-Type: application/json" \
  -d '{"jsonrpc":"2.0","id":1,"method":"send_payment","params":[{"invoice":"<real invoice from node2 new_invoice>","max_fee_amount":"0x1"}]}'
# note the payment_hash, register in tracked_payments same as above, wait, curl /issues/fee-too-low
```

## 8. asset-mismatch (real) — Testable through hardcoding

```bash
# Case A: native-CKB channel, invoice wants a UDT -> should flag
sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO channel_status_current
  (node_id, channel_id, channel_outpoint, peer_pubkey, is_public, state_name, enabled,
   local_balance_raw, remote_balance_raw, offered_tlc_balance_raw, received_tlc_balance_raw,
   last_seen_at, updated_at)
VALUES ('node1','test-channel-native','test-outpoint-native','test-peer',0,'ChannelReady',1,
   '0x0','0x0','0x0','0x0','2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO payment_status_current
  (payment_hash, node_id, status, router_json, observed_at, updated_at)
VALUES ('test-payment-mismatch-a','node1','Success',
  '[{\"nodes\":[{\"channel_outpoint\":\"test-outpoint-native\"}]}]',
  '2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

sqlite3 fiber-diagnostics.db "INSERT OR REPLACE INTO invoice_status_current
  (payment_hash, invoice_status, parsed_invoice_json, observed_at, updated_at)
VALUES ('test-payment-mismatch-a','Open',
  '{\"data\":{\"attrs\":[{\"udt_script\":\"0x550000001000000030000000310000001142755a044bf2ee358cba9f2da187ce928c91cd4dc8692ded0337efa677d21a0120000000878fcc6f1f08d48e87bb1c3b3d5083f23f8a39c5d5c764f253b55b998526439b\"}]}}',
  '2026-07-11T00:00:00Z','2026-07-11T00:00:00Z');"

  sleep 6
  curl http://127.0.0.1:3000/issues/asset-mismatch
  
  ```
