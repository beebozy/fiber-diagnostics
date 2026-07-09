-- Migration: create node_status_current
CREATE TABLE IF NOT EXISTS node_status_current (
    node_id TEXT PRIMARY KEY,
    pubkey TEXT,
    rpc_reachable INTEGER NOT NULL DEFAULT 1,
    last_polled_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
