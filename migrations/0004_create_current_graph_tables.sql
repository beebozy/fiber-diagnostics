CREATE TABLE graph_node_current (
    pubkey TEXT PRIMARY KEY,
    node_name TEXT,
    addresses_json TEXT NOT NULL DEFAULT '[]',
    chain_hash TEXT,
    auto_accept_min_ckb_funding_amount_raw TEXT,
    udt_cfg_infos_json TEXT,
    timestamp_raw TEXT,
    observed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);

CREATE TABLE graph_channel_current (
    channel_outpoint TEXT PRIMARY KEY,
    node1_pubkey TEXT NOT NULL,
    node2_pubkey TEXT NOT NULL,
    capacity_raw TEXT,
    chain_hash TEXT,
    udt_type_script_json TEXT,
    created_timestamp_raw TEXT,
    update_info_node1_json TEXT,
    update_info_node2_json TEXT,
    observed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
