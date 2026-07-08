CREATE TABLE node_status_current (
    node_id TEXT PRIMARY KEY,
    pubkey TEXT,
    node_name TEXT,
    version TEXT,
    commit_hash TEXT,
    chain_hash TEXT,
    addresses_json TEXT NOT NULL DEFAULT '[]',
    peers_count INTEGER NOT NULL DEFAULT 0,
    channel_count INTEGER NOT NULL DEFAULT 0,
    pending_channel_count INTEGER NOT NULL DEFAULT 0,
    rpc_reachable INTEGER NOT NULL DEFAULT 0,
    rpc_error TEXT,
    last_polled_at TEXT NOT NULL,
    last_success_at TEXT,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);

CREATE TABLE peer_status_current (
    node_id TEXT NOT NULL,
    peer_pubkey TEXT NOT NULL,
    address TEXT NOT NULL,
    connected INTEGER NOT NULL DEFAULT 1,
    last_seen_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (node_id, peer_pubkey),
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);

CREATE TABLE channel_status_current (
    node_id TEXT NOT NULL,
    channel_id TEXT NOT NULL,
    peer_pubkey TEXT NOT NULL,
    channel_outpoint TEXT,
    is_public INTEGER NOT NULL DEFAULT 0,
    state_name TEXT NOT NULL,
    state_flags TEXT,
    enabled INTEGER NOT NULL DEFAULT 0,
    funding_udt_type_script_json TEXT,
    local_balance_raw TEXT NOT NULL,
    remote_balance_raw TEXT NOT NULL,
    offered_tlc_balance_raw TEXT NOT NULL,
    received_tlc_balance_raw TEXT NOT NULL,
    latest_commitment_transaction_hash TEXT,
    tlc_expiry_delta_raw TEXT,
    tlc_fee_proportional_millionths_raw TEXT,
    shutdown_transaction_hash TEXT,
    created_at_raw TEXT,
    last_seen_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (node_id, channel_id),
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);
