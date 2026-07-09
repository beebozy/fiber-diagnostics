mod rpc_client;
mod diagnostics;
use rpc_client::FiberRpcClient;

#[derive(sqlx::FromRow, Debug)]
struct MonitoredNode {
    id: String,
    rpc_url: String,
}

async fn fetch_monitored_nodes(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<MonitoredNode>> {
    let nodes = sqlx::query_as::<_, MonitoredNode>(
        "SELECT id, rpc_url FROM monitored_nodes WHERE enabled = 1"
    )
    .fetch_all(pool)
    .await?;
    Ok(nodes)
}

async fn log_poller_run(
    pool: &sqlx::SqlitePool,
    poller_type: &str,
    node_id: Option<&str>,
    target_key: Option<&str>,
    success: bool,
    error_message: Option<&str>,
    started_at: &str,
    finished_at: &str,
) {
    let _ = sqlx::query(
        "INSERT INTO poller_runs (poller_type, node_id, target_key, success, error_message, started_at, finished_at)
         VALUES (?, ?, ?, ?, ?, ?, ?)"
    )
    .bind(poller_type)
    .bind(node_id)
    .bind(target_key)
    .bind(success as i64)
    .bind(error_message)
    .bind(started_at)
    .bind(finished_at)
    .execute(pool)
    .await;
}

async fn poll_node(node_id: &str, rpc_url: &str, pool: &sqlx::SqlitePool) {
    let client = FiberRpcClient::new(rpc_url);
    let now = chrono::Utc::now().to_rfc3339();

    // --- node_info ---
    let started = chrono::Utc::now().to_rfc3339();
    let info_result = client.node_info().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &info_result {
        Ok(_) => log_poller_run(pool, "node_info", Some(node_id), None, true, None, &started, &finished).await,
        Err(e) => log_poller_run(pool, "node_info", Some(node_id), None, false, Some(&e.to_string()), &started, &finished).await,
    }

    match info_result {
        Ok(info) => {
            let _ = sqlx::query(
                "INSERT INTO node_status_current (node_id, pubkey, rpc_reachable, rpc_error, last_polled_at, last_success_at, updated_at)
                 VALUES (?, ?, 1, NULL, ?, ?, ?)
                 ON CONFLICT(node_id) DO UPDATE SET
                    pubkey=excluded.pubkey, rpc_reachable=1, rpc_error=NULL,
                    last_polled_at=excluded.last_polled_at, last_success_at=excluded.last_success_at, updated_at=excluded.updated_at"
            )
            .bind(node_id).bind(info["pubkey"].as_str())
            .bind(&now).bind(&now).bind(&now)
            .execute(pool).await;

            println!("[{node_id}] node_info OK");
        }
        Err(e) => {
            eprintln!("[{node_id}] UNREACHABLE: {e}");
            let _ = sqlx::query(
                "INSERT INTO node_status_current (node_id, rpc_reachable, rpc_error, last_polled_at, updated_at)
                 VALUES (?, 0, ?, ?, ?)
                 ON CONFLICT(node_id) DO UPDATE SET
                    rpc_reachable=0, rpc_error=excluded.rpc_error,
                    last_polled_at=excluded.last_polled_at, updated_at=excluded.updated_at"
            )
            .bind(node_id).bind(e.to_string()).bind(&now).bind(&now)
            .execute(pool).await;
            return;
        }
    }

    // --- list_peers ---
    let started = chrono::Utc::now().to_rfc3339();
    let peers_result = client.list_peers().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &peers_result {
        Ok(_) => log_poller_run(pool, "list_peers", Some(node_id), None, true, None, &started, &finished).await,
        Err(e) => log_poller_run(pool, "list_peers", Some(node_id), None, false, Some(&e.to_string()), &started, &finished).await,
    }

    if let Ok(result) = peers_result {
        if let Some(peers) = result["peers"].as_array() {
            for peer in peers {
                let _ = sqlx::query(
                    "INSERT INTO peer_status_current (node_id, peer_pubkey, address, connected, last_seen_at, updated_at)
                     VALUES (?, ?, ?, 1, ?, ?)
                     ON CONFLICT(node_id, peer_pubkey) DO UPDATE SET
                        address=excluded.address, connected=1,
                        last_seen_at=excluded.last_seen_at, updated_at=excluded.updated_at"
                )
                .bind(node_id)
                .bind(peer["pubkey"].as_str())
                .bind(peer["address"].as_str())
                .bind(&now).bind(&now)
                .execute(pool).await;
            }
            println!("[{node_id}] list_peers OK — {} peers", peers.len());
        }
    } else if let Err(e) = &client.list_peers().await {
        eprintln!("[{node_id}] list_peers failed: {e}");
    }

    // --- list_channels ---
    let started = chrono::Utc::now().to_rfc3339();
    let channels_result = client.list_channels().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &channels_result {
        Ok(_) => log_poller_run(pool, "list_channels", Some(node_id), None, true, None, &started, &finished).await,
        Err(e) => log_poller_run(pool, "list_channels", Some(node_id), None, false, Some(&e.to_string()), &started, &finished).await,
    }

    if let Ok(result) = channels_result {
        if let Some(channels) = result["channels"].as_array() {
            for ch in channels {
                let _ = sqlx::query(
                    "INSERT INTO channel_status_current
                        (node_id, channel_id, peer_pubkey, channel_outpoint, is_public,
                         state_name, state_flags, enabled, funding_udt_type_script_json,
                         local_balance_raw, remote_balance_raw, offered_tlc_balance_raw,
                         received_tlc_balance_raw, latest_commitment_transaction_hash,
                         tlc_expiry_delta_raw, tlc_fee_proportional_millionths_raw,
                         shutdown_transaction_hash, created_at_raw, last_seen_at, updated_at)
                     VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                     ON CONFLICT(node_id, channel_id) DO UPDATE SET
                        peer_pubkey=excluded.peer_pubkey, channel_outpoint=excluded.channel_outpoint,
                        is_public=excluded.is_public, state_name=excluded.state_name,
                        state_flags=excluded.state_flags, enabled=excluded.enabled,
                        funding_udt_type_script_json=excluded.funding_udt_type_script_json,
                        local_balance_raw=excluded.local_balance_raw, remote_balance_raw=excluded.remote_balance_raw,
                        offered_tlc_balance_raw=excluded.offered_tlc_balance_raw,
                        received_tlc_balance_raw=excluded.received_tlc_balance_raw,
                        latest_commitment_transaction_hash=excluded.latest_commitment_transaction_hash,
                        tlc_expiry_delta_raw=excluded.tlc_expiry_delta_raw,
                        tlc_fee_proportional_millionths_raw=excluded.tlc_fee_proportional_millionths_raw,
                        shutdown_transaction_hash=excluded.shutdown_transaction_hash,
                        last_seen_at=excluded.last_seen_at, updated_at=excluded.updated_at"
                )
                .bind(node_id)
                .bind(ch["channel_id"].as_str())
                .bind(ch["pubkey"].as_str())
                .bind(ch["channel_outpoint"].as_str())
                .bind(ch["is_public"].as_bool().unwrap_or(false) as i64)
                .bind(ch["state"]["state_name"].as_str())
                .bind(ch["state"]["state_flags"].as_str())
                .bind(ch["enabled"].as_bool().unwrap_or(false) as i64)
                .bind(ch["funding_udt_type_script"].as_object().map(|_| ch["funding_udt_type_script"].to_string()))
                .bind(ch["local_balance"].as_str())
                .bind(ch["remote_balance"].as_str())
                .bind(ch["offered_tlc_balance"].as_str())
                .bind(ch["received_tlc_balance"].as_str())
                .bind(ch["latest_commitment_transaction_hash"].as_str())
                .bind(ch["tlc_expiry_delta"].as_str())
                .bind(ch["tlc_fee_proportional_millionths"].as_str())
                .bind(ch["shutdown_transaction_hash"].as_str())
                .bind(ch["created_at"].as_str())
                .bind(&now).bind(&now)
                .execute(pool).await;
            }
            println!("[{node_id}] list_channels OK — {} channels", channels.len());
        }
    }
}

async fn poll_graph(client: &FiberRpcClient, pool: &sqlx::SqlitePool) {
    let started = chrono::Utc::now().to_rfc3339();
    let result = client.graph_nodes().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &result {
        Ok(_) => log_poller_run(pool, "graph_nodes", None, Some("global"), true, None, &started, &finished).await,
        Err(e) => log_poller_run(pool, "graph_nodes", None, Some("global"), false, Some(&e.to_string()), &started, &finished).await,
    }

    if let Ok(r) = result {
        if let Some(nodes) = r["nodes"].as_array() {
            println!("graph_nodes OK — {} known nodes", nodes.len());
        }
    }

    let started = chrono::Utc::now().to_rfc3339();
    let result = client.graph_channels().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &result {
        Ok(_) => log_poller_run(pool, "graph_channels", None, Some("global"), true, None, &started, &finished).await,
        Err(e) => log_poller_run(pool, "graph_channels", None, Some("global"), false, Some(&e.to_string()), &started, &finished).await,
    }

    if let Ok(r) = result {
        if let Some(channels) = r["channels"].as_array() {
            println!("graph_channels OK — {} known channels", channels.len());
        }
    }
}

use diagnostics::engine::DiagnosticsEngine;
use diagnostics::repository::load_nodes;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    // Fast loop: node/peer/channel state, every 5s
    let fast_pool = pool.clone();
    let fast_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            ticker.tick().await;
            match fetch_monitored_nodes(&fast_pool).await {
                Ok(nodes) => {
                    for node in nodes {
                        poll_node(&node.id, &node.rpc_url, &fast_pool).await;
                    }
                }
                Err(e) => eprintln!("failed to fetch monitored_nodes: {e}"),
            }
        }
    });

    // Slow loop: graph data, every 30s
    let slow_pool = pool.clone();
    let slow_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(30));
        loop {
            ticker.tick().await;
            match fetch_monitored_nodes(&slow_pool).await {
                Ok(nodes) => {
                    if let Some(first) = nodes.first() {
                        let client = FiberRpcClient::new(&first.rpc_url);
                        poll_graph(&client, &slow_pool).await;
                    }
                }
                Err(e) => eprintln!("failed to fetch monitored_nodes for graph poll: {e}"),
            }
        }
    });

    let _ = tokio::join!(fast_handle, slow_handle);
    Ok(())
}