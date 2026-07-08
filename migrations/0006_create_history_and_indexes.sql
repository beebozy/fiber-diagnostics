CREATE TABLE node_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    pubkey TEXT,
    peers_count INTEGER,
    channel_count INTEGER,
    pending_channel_count INTEGER,
    rpc_reachable INTEGER NOT NULL,
    rpc_error TEXT,
    observed_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);

CREATE TABLE channel_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    node_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    peer_pubkey TEXT NOT NULL,
    state_name TEXT NOT NULL,
    enabled INTEGER NOT NULL,
    local_balance_raw TEXT NOT NULL,
    remote_balance_raw TEXT NOT NULL,
    offered_tlc_balance_raw TEXT NOT NULL,
    received_tlc_balance_raw TEXT NOT NULL,
    observed_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);

CREATE TABLE payment_status_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    payment_hash TEXT NOT NULL,
    node_id TEXT NOT NULL,
    status TEXT NOT NULL,
    failed_error TEXT,
    fee_raw TEXT,
    observed_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);

CREATE TABLE issue_events (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    issue_id TEXT NOT NULL,
    fingerprint TEXT NOT NULL,
    event_type TEXT NOT NULL,
    code TEXT NOT NULL,
    severity TEXT NOT NULL,
    node_id TEXT,
    peer_pubkey TEXT,
    channel_id TEXT,
    payment_hash TEXT,
    title TEXT NOT NULL,
    message TEXT NOT NULL,
    recommendation TEXT NOT NULL,
    metadata_json TEXT,
    observed_at TEXT NOT NULL
);

CREATE TABLE poller_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    poller_type TEXT NOT NULL,
    node_id TEXT,
    target_key TEXT,
    success INTEGER NOT NULL,
    error_message TEXT,
    started_at TEXT NOT NULL,
    finished_at TEXT NOT NULL
);

CREATE INDEX idx_peer_status_node_id ON peer_status_current(node_id);
CREATE INDEX idx_channel_status_node_id ON channel_status_current(node_id);
CREATE INDEX idx_channel_status_peer_pubkey ON channel_status_current(peer_pubkey);
CREATE INDEX idx_payment_status_node_id ON payment_status_current(node_id);
CREATE INDEX idx_payment_status_status ON payment_status_current(status);
CREATE INDEX idx_issues_status ON issues_current(status);
CREATE INDEX idx_issues_code ON issues_current(code);
CREATE INDEX idx_issues_node_id ON issues_current(node_id);
CREATE INDEX idx_issues_payment_hash ON issues_current(payment_hash);
CREATE INDEX idx_node_history_node_time ON node_status_history(node_id, observed_at);
CREATE INDEX idx_channel_history_channel_time ON channel_status_history(node_id, channel_id, observed_at);
CREATE INDEX idx_payment_history_payment_time ON payment_status_history(payment_hash, observed_at);
CREATE INDEX idx_issue_events_fingerprint_time ON issue_events(fingerprint, observed_at);
CREATE INDEX idx_poller_runs_type_time ON poller_runs(poller_type, started_at);
