CREATE TABLE issues_current (
    id TEXT PRIMARY KEY,
    code TEXT NOT NULL,
    severity TEXT NOT NULL,
    status TEXT NOT NULL,
    node_id TEXT,
    peer_pubkey TEXT,
    channel_id TEXT,
    payment_hash TEXT,
    fingerprint TEXT NOT NULL UNIQUE,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    first_detected_at TEXT NOT NULL,
    last_detected_at TEXT NOT NULL,
    resolved_at TEXT,
    metadata_json TEXT,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);
