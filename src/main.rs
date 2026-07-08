mod rpc_client;
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

async fn poll_node(node_id: &str, rpc_url: &str, pool: &sqlx::SqlitePool) {
    let client = FiberRpcClient::new(rpc_url);
    let now = chrono::Utc::now().to_rfc3339();

    // --- node_info ---
    match client.node_info().await {
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
            return; // node's down, don't bother polling peers/channels on it
        }
    }

    // --- list_peers ---
    match client.list_peers().await {
        Ok(result) => {
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
        }
        Err(e) => eprintln!("[{node_id}] list_peers failed: {e}"),
    }

    // --- list_channels ---
    match client.list_channels().await {
        Ok(result) => {
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
                            peer_pubkey=excluded.peer_pubkey,
                            channel_outpoint=excluded.channel_outpoint,
                            is_public=excluded.is_public,
                            state_name=excluded.state_name,
                            state_flags=excluded.state_flags,
                            enabled=excluded.enabled,
                            funding_udt_type_script_json=excluded.funding_udt_type_script_json,
                            local_balance_raw=excluded.local_balance_raw,
                            remote_balance_raw=excluded.remote_balance_raw,
                            offered_tlc_balance_raw=excluded.offered_tlc_balance_raw,
                            received_tlc_balance_raw=excluded.received_tlc_balance_raw,
                            latest_commitment_transaction_hash=excluded.latest_commitment_transaction_hash,
                            tlc_expiry_delta_raw=excluded.tlc_expiry_delta_raw,
                            tlc_fee_proportional_millionths_raw=excluded.tlc_fee_proportional_millionths_raw,
                            shutdown_transaction_hash=excluded.shutdown_transaction_hash,
                            last_seen_at=excluded.last_seen_at,
                            updated_at=excluded.updated_at"
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
                    .bind(&now)
                    .bind(&now)
                    .execute(pool).await;
                }
                println!("[{node_id}] list_channels OK — {} channels", channels.len());
            }
        }
        Err(e) => eprintln!("[{node_id}] list_channels failed: {e}"),
    }
}

async fn poll_graph(client: &FiberRpcClient, pool: &sqlx::SqlitePool) {
    let now = chrono::Utc::now().to_rfc3339();

    match client.graph_nodes().await {
        Ok(result) => {
            if let Some(nodes) = result["nodes"].as_array() {
                println!("Sample graph node structure:\n{:#?}", nodes.first());
                // once field names are confirmed, replace the line above with real inserts, e.g.:
                // for n in nodes { sqlx::query("INSERT INTO graph_node_current ...").bind(...).execute(pool).await; }
            }
        }
        Err(e) => eprintln!("graph_nodes failed: {e}"),
    }

    match client.graph_channels().await {
        Ok(result) => {
            if let Some(channels) = result["channels"].as_array() {
                println!("Sample graph channel structure:\n{:#?}", channels.first());
            }
        }
        Err(e) => eprintln!("graph_channels failed: {e}"),
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
    let nodes = fetch_monitored_nodes(&pool).await?;
    println!("Found {} monitored nodes\n", nodes.len());

    for node in &nodes {
        println!("--- Polling {} ({}) ---", node.id, node.rpc_url);
        poll_node(&node.id, &node.rpc_url, &pool).await;
        println!();
    }

    if let Some(first) = nodes.first() {
        let graph_client = FiberRpcClient::new(&first.rpc_url);
        poll_graph(&graph_client, &pool).await;
    }

    println!("\nPoll cycle complete.");
    Ok(())
}