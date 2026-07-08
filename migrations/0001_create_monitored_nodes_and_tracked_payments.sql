CREATE TABLE monitored_nodes (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    rpc_url TEXT NOT NULL UNIQUE,
    enabled INTEGER NOT NULL DEFAULT 1,
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE tracked_payments (
    payment_hash TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    invoice TEXT,
    target_pubkey TEXT,
    amount_raw TEXT,
    asset_type TEXT,
    tracking_status TEXT NOT NULL DEFAULT 'active',
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);
