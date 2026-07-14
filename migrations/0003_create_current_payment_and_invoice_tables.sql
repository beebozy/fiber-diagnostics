CREATE TABLE payment_status_current (
    payment_hash TEXT PRIMARY KEY,
    node_id TEXT NOT NULL,
    target_pubkey TEXT,
    invoice TEXT,
    amount_raw TEXT,
    asset_type TEXT,
    status TEXT NOT NULL,
    failed_error TEXT,
    fee_raw TEXT,
    router_json TEXT,
    created_at_raw TEXT,
    last_updated_at_raw TEXT,
    observed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    FOREIGN KEY (node_id) REFERENCES monitored_nodes(id)
);

CREATE TABLE invoice_status_current (
    payment_hash TEXT PRIMARY KEY,
    invoice TEXT,
    invoice_status TEXT,
    amount_raw TEXT,
    currency TEXT,
    expiry_time_raw TEXT,
    final_expiry_delta_raw TEXT,
    udt_type_script_json TEXT,
    parsed_invoice_json TEXT,
    observed_at TEXT NOT NULL,
    updated_at TEXT NOT NULL
);
