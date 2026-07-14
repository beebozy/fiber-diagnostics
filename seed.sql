INSERT INTO monitored_nodes (
    id,
    name,
    rpc_url,
    enabled,
    created_at,
    updated_at
)
VALUES
(
    'node1',
    'Local Node A',
    'http://127.0.0.1:8227',
    1,
    datetime('now'),
    datetime('now')
),
(
    'node2',
    'Local Node B',
    'http://127.0.0.1:8237',
    1,
    datetime('now'),
    datetime('now')
);


-- sqlite3 fiber-diagnostics.db < seed.sql