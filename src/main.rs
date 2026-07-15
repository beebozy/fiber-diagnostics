mod diagnostics;
mod payment_tracker;
mod rpc_client;
use axum::{
    extract::{Extension, Path, Query},
    http::StatusCode,
    routing::{get, post, delete},
    Json, Router,
};
use diagnostics::engine::DiagnosticsEngine;
use diagnostics::issue::Issue;
use diagnostics::repository::load_data;
use rpc_client::FiberRpcClient;
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tower_http::cors::{Any, CorsLayer};

#[derive(sqlx::FromRow, Debug, serde::Serialize, serde::Deserialize, Clone)]
struct MonitoredNode {
    id: String,
    name: String,
    rpc_url: String,
    enabled: bool,
}

async fn fetch_monitored_nodes(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<MonitoredNode>> {
    let nodes = sqlx::query_as::<_, MonitoredNode>(
        "SELECT id, name, rpc_url, enabled FROM monitored_nodes WHERE enabled = 1",
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
        Ok(_) => {
            log_poller_run(
                pool,
                "node_info",
                Some(node_id),
                None,
                true,
                None,
                &started,
                &finished,
            )
            .await
        }
        Err(e) => {
            log_poller_run(
                pool,
                "node_info",
                Some(node_id),
                None,
                false,
                Some(&e.to_string()),
                &started,
                &finished,
            )
            .await
        }
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
        Ok(_) => {
            log_poller_run(
                pool,
                "list_peers",
                Some(node_id),
                None,
                true,
                None,
                &started,
                &finished,
            )
            .await
        }
        Err(e) => {
            log_poller_run(
                pool,
                "list_peers",
                Some(node_id),
                None,
                false,
                Some(&e.to_string()),
                &started,
                &finished,
            )
            .await
        }
    }

    match &peers_result {
        Ok(result) => {
            if let Some(peers) = result["peers"].as_array() {
                let seen_pubkeys: Vec<String> = peers
                    .iter()
                    .filter_map(|p| p["pubkey"].as_str().map(|s| s.to_string()))
                    .collect();

                // Mark any peer this node previously reported as connected,
                // but that isn't in THIS round's list_peers response, as
                // disconnected. Without this, connected never flips to 0.
                if seen_pubkeys.is_empty() {
                    let _ = sqlx::query(
                        "UPDATE peer_status_current SET connected = 0, updated_at = ? WHERE node_id = ? AND connected = 1"
                    )
                    .bind(&now).bind(node_id)
                    .execute(pool).await;
                } else {
                    let placeholders = seen_pubkeys.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    let sql = format!(
                        "UPDATE peer_status_current SET connected = 0, updated_at = ? \
                         WHERE node_id = ? AND connected = 1 AND peer_pubkey NOT IN ({placeholders})"
                    );
                    let mut q = sqlx::query(&sql).bind(&now).bind(node_id);
                    for pk in &seen_pubkeys {
                        q = q.bind(pk);
                    }
                    let _ = q.execute(pool).await;
                }

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
    let started = chrono::Utc::now().to_rfc3339();
    let channels_result = client.list_channels().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &channels_result {
        Ok(_) => {
            log_poller_run(
                pool,
                "list_channels",
                Some(node_id),
                None,
                true,
                None,
                &started,
                &finished,
            )
            .await
        }
        Err(e) => {
            log_poller_run(
                pool,
                "list_channels",
                Some(node_id),
                None,
                false,
                Some(&e.to_string()),
                &started,
                &finished,
            )
            .await
        }
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
    let now = chrono::Utc::now().to_rfc3339();
    let started = chrono::Utc::now().to_rfc3339();
    let result = client.graph_nodes().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &result {
        Ok(_) => {
            log_poller_run(
                pool,
                "graph_nodes",
                None,
                Some("global"),
                true,
                None,
                &started,
                &finished,
            )
            .await
        }
        Err(e) => {
            log_poller_run(
                pool,
                "graph_nodes",
                None,
                Some("global"),
                false,
                Some(&e.to_string()),
                &started,
                &finished,
            )
            .await
        }
    }

    if let Ok(r) = result {
        if let Some(nodes) = r["nodes"].as_array() {
            println!("graph_nodes OK — {} known nodes", nodes.len());
            for node in nodes {
                if let Some(pubkey) = node["pubkey"].as_str() {
                    let node_name = node["node_name"].as_str();
                    let addresses_json = node["addresses"].to_string();
                    let chain_hash = node["chain_hash"].as_str();
                    let auto_accept_min_ckb_funding_amount_raw = node["auto_accept_min_ckb_funding_amount"].as_str();
                    let udt_cfg_infos_json = node["udt_cfg_infos"].to_string();
                    let timestamp_raw = node["timestamp"].as_str();

                    let _ = sqlx::query(
                        "INSERT INTO graph_node_current
                            (pubkey, node_name, addresses_json, chain_hash,
                             auto_accept_min_ckb_funding_amount_raw, udt_cfg_infos_json,
                             timestamp_raw, observed_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                         ON CONFLICT(pubkey) DO UPDATE SET
                            node_name=excluded.node_name, addresses_json=excluded.addresses_json,
                            chain_hash=excluded.chain_hash,
                            auto_accept_min_ckb_funding_amount_raw=excluded.auto_accept_min_ckb_funding_amount_raw,
                            udt_cfg_infos_json=excluded.udt_cfg_infos_json,
                            timestamp_raw=excluded.timestamp_raw,
                            observed_at=excluded.observed_at, updated_at=excluded.updated_at"
                    )
                    .bind(pubkey)
                    .bind(node_name)
                    .bind(addresses_json)
                    .bind(chain_hash)
                    .bind(auto_accept_min_ckb_funding_amount_raw)
                    .bind(udt_cfg_infos_json)
                    .bind(timestamp_raw)
                    .bind(&now)
                    .bind(&now)
                    .execute(pool)
                    .await;
                }
            }
        }
    }

    let started = chrono::Utc::now().to_rfc3339();
    let result = client.graph_channels().await;
    let finished = chrono::Utc::now().to_rfc3339();

    match &result {
        Ok(_) => {
            log_poller_run(
                pool,
                "graph_channels",
                None,
                Some("global"),
                true,
                None,
                &started,
                &finished,
            )
            .await
        }
        Err(e) => {
            log_poller_run(
                pool,
                "graph_channels",
                None,
                Some("global"),
                false,
                Some(&e.to_string()),
                &started,
                &finished,
            )
            .await
        }
    }

    if let Ok(r) = result {
        if let Some(channels) = r["channels"].as_array() {
            println!("graph_channels OK — {} known channels", channels.len());
            for ch in channels {
                if let Some(outpoint) = ch["channel_outpoint"].as_str() {
                    let node1_pubkey = ch["node1"].as_str().unwrap_or("");
                    let node2_pubkey = ch["node2"].as_str().unwrap_or("");
                    let capacity_raw = ch["capacity"].as_str();
                    let chain_hash = ch["chain_hash"].as_str();
                    let udt_type_script_json = ch["udt_type_script"].to_string();
                    let created_timestamp_raw = ch["created_timestamp"].as_str();
                    let update_info_node1_json = ch["update_info_of_node1"].to_string();
                    let update_info_node2_json = ch["update_info_of_node2"].to_string();

                    let _ = sqlx::query(
                        "INSERT INTO graph_channel_current
                            (channel_outpoint, node1_pubkey, node2_pubkey, capacity_raw,
                             chain_hash, udt_type_script_json, created_timestamp_raw,
                             update_info_node1_json, update_info_node2_json, observed_at, updated_at)
                         VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                         ON CONFLICT(channel_outpoint) DO UPDATE SET
                            node1_pubkey=excluded.node1_pubkey, node2_pubkey=excluded.node2_pubkey,
                            capacity_raw=excluded.capacity_raw, chain_hash=excluded.chain_hash,
                            udt_type_script_json=excluded.udt_type_script_json,
                            created_timestamp_raw=excluded.created_timestamp_raw,
                            update_info_node1_json=excluded.update_info_node1_json,
                            update_info_node2_json=excluded.update_info_node2_json,
                            observed_at=excluded.observed_at, updated_at=excluded.updated_at"
                    )
                    .bind(outpoint)
                    .bind(node1_pubkey)
                    .bind(node2_pubkey)
                    .bind(capacity_raw)
                    .bind(chain_hash)
                    .bind(udt_type_script_json)
                    .bind(created_timestamp_raw)
                    .bind(update_info_node1_json)
                    .bind(update_info_node2_json)
                    .bind(&now)
                    .bind(&now)
                    .execute(pool)
                    .await;
                }
            }
        }
    }
}

#[derive(Debug, Deserialize)]
struct IssueQuery {
    kind: Option<String>,
    severity: Option<String>,
}

#[derive(Debug, Serialize)]
struct IssuesResponse {
    generated_at: String,
    count: usize,
    issues: Vec<Issue>,
}

#[derive(Debug, Serialize)]
struct ApiError {
    error: String,
}

fn apply_filters(mut issues: Vec<Issue>, filter: &IssueQuery) -> Vec<Issue> {
    if let Some(kind) = &filter.kind {
        issues.retain(|issue| issue.kind == *kind);
    }
    if let Some(severity) = &filter.severity {
        issues.retain(|issue| issue.severity.to_string().eq_ignore_ascii_case(severity));
    }
    issues
}

fn wrap(issues: Vec<Issue>) -> IssuesResponse {
    IssuesResponse {
        generated_at: chrono::Utc::now().to_rfc3339(),
        count: issues.len(),
        issues,
    }
}

type IssueCache = Arc<RwLock<Vec<Issue>>>;

async fn refresh_issue_cache(cache: &IssueCache, pool: &sqlx::SqlitePool) {
    match load_data(pool).await {
        Ok(data) => {
            let issues = DiagnosticsEngine::evaluate(data);
            let mut write_guard = cache.write().await;
            *write_guard = issues;
        }
        Err(err) => {
            eprintln!("failed to refresh issue cache: {err}");
        }
    }
}

#[derive(Debug, Deserialize)]
struct SendPaymentRequest {
    node_id: String,
    invoice: String,
}

#[derive(Debug, Serialize)]
struct SendPaymentApiResponse {
    payment_hash: String,
}

async fn post_send_payment(
    Extension(pool): Extension<sqlx::SqlitePool>,
    Json(req): Json<SendPaymentRequest>,
) -> Result<Json<SendPaymentApiResponse>, (StatusCode, Json<ApiError>)> {
    let rpc_url: Option<String> = sqlx::query_scalar("SELECT rpc_url FROM monitored_nodes WHERE id = ?")
        .bind(&req.node_id)
        .fetch_optional(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let rpc_url = rpc_url.ok_or_else(|| {
        (
            StatusCode::BAD_REQUEST,
            Json(ApiError { error: format!("unknown node_id '{}'", req.node_id) }),
        )
    })?;

    let payment_hash = payment_tracker::send_and_track(&pool, &req.node_id, &rpc_url, &req.invoice)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    Ok(Json(SendPaymentApiResponse { payment_hash }))
}

async fn get_issues(
    Query(filter): Query<IssueQuery>,
    Extension(cache): Extension<IssueCache>,
) -> Result<Json<IssuesResponse>, (StatusCode, Json<ApiError>)> {
    let issues = cache.read().await.clone();
    let issues = apply_filters(issues, &filter);
    Ok(Json(wrap(issues)))
}

async fn get_issues_by_kind(
    Path(kind): Path<String>,
    Query(filter): Query<IssueQuery>,
    Extension(cache): Extension<IssueCache>,
) -> Result<Json<IssuesResponse>, (StatusCode, Json<ApiError>)> {
    let issues = cache.read().await.clone();
    let mut issues = apply_filters(issues, &filter);
    issues.retain(|issue| issue.kind == kind);
    Ok(Json(wrap(issues)))
}

#[derive(Debug, Serialize)]
struct NetworkStats {
    generated_at: String,
    graph_nodes: i64,
    graph_channels: i64,
    monitored_nodes: i64,
    nodes_online: i64,
    nodes_offline: i64,
    total_peers: i64,
    total_channels: i64,
    active_issues: usize,
}

async fn get_stats(
    Extension(pool): Extension<sqlx::SqlitePool>,
    Extension(cache): Extension<IssueCache>,
) -> Result<Json<NetworkStats>, (StatusCode, Json<ApiError>)> {
    let graph_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM graph_node_current")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let graph_channels: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM graph_channel_current")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let monitored_nodes: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM monitored_nodes WHERE enabled = 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let nodes_online: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_status_current WHERE rpc_reachable = 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let nodes_offline: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM node_status_current WHERE rpc_reachable = 0")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let total_peers: i64 = sqlx::query_scalar("SELECT COUNT(DISTINCT peer_pubkey) FROM peer_status_current WHERE connected = 1")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let total_channels: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM channel_status_current")
        .fetch_one(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    let active_issues = cache.read().await.len();

    Ok(Json(NetworkStats {
        generated_at: chrono::Utc::now().to_rfc3339(),
        graph_nodes,
        graph_channels,
        monitored_nodes,
        nodes_online,
        nodes_offline,
        total_peers,
        total_channels,
        active_issues,
    }))
}

async fn get_nodes(
    Extension(pool): Extension<sqlx::SqlitePool>,
) -> Result<Json<Vec<MonitoredNode>>, (StatusCode, Json<ApiError>)> {
    let nodes = sqlx::query_as::<_, MonitoredNode>(
        "SELECT id, name, rpc_url, enabled FROM monitored_nodes"
    )
    .fetch_all(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    Ok(Json(nodes))
}

#[derive(Deserialize)]
struct CreateNodeRequest {
    id: String,
    name: String,
    rpc_url: String,
}

async fn post_node(
    Extension(pool): Extension<sqlx::SqlitePool>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<Json<MonitoredNode>, (StatusCode, Json<ApiError>)> {
    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO monitored_nodes (id, name, rpc_url, enabled, created_at, updated_at)
         VALUES (?, ?, ?, 1, ?, ?)
         ON CONFLICT(id) DO UPDATE SET
            name=excluded.name, rpc_url=excluded.rpc_url, enabled=1, updated_at=excluded.updated_at"
    )
    .bind(&req.id)
    .bind(&req.name)
    .bind(&req.rpc_url)
    .bind(&now)
    .bind(&now)
    .execute(&pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    Ok(Json(MonitoredNode {
        id: req.id,
        name: req.name,
        rpc_url: req.rpc_url,
        enabled: true,
    }))
}

async fn delete_node(
    Path(id): Path<String>,
    Extension(pool): Extension<sqlx::SqlitePool>,
) -> Result<StatusCode, (StatusCode, Json<ApiError>)> {
    let _ = sqlx::query("DELETE FROM node_status_current WHERE node_id = ?").bind(&id).execute(&pool).await;
    let _ = sqlx::query("DELETE FROM peer_status_current WHERE node_id = ?").bind(&id).execute(&pool).await;
    let _ = sqlx::query("DELETE FROM channel_status_current WHERE node_id = ?").bind(&id).execute(&pool).await;
    let _ = sqlx::query("DELETE FROM tracked_payments WHERE node_id = ?").bind(&id).execute(&pool).await;

    sqlx::query("DELETE FROM monitored_nodes WHERE id = ?")
        .bind(&id)
        .execute(&pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ApiError { error: e.to_string() })))?;

    Ok(StatusCode::NO_CONTENT)
}


#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    // Fast loop: node/peer/channel state, every 5s
    let issue_cache: IssueCache = Arc::new(RwLock::new(Vec::new()));
    refresh_issue_cache(&issue_cache, &pool).await;

    let fast_pool = pool.clone();
    let issue_cache_clone = issue_cache.clone();
    let fast_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(5));
        loop {
            ticker.tick().await;
            match fetch_monitored_nodes(&fast_pool).await {
                Ok(nodes) => {
                    for node in nodes {
                        poll_node(&node.id, &node.rpc_url, &fast_pool).await;
                    }
                    refresh_issue_cache(&issue_cache_clone, &fast_pool).await;
                }
                Err(e) => eprintln!("failed to fetch monitored_nodes: {e}"),
            }
        }
    });

    let api_cache = issue_cache.clone();
    let api_pool = pool.clone();
    let app = Router::new()
        .route("/issues", get(get_issues))
        .route("/payments", post(post_send_payment))
        .route("/issues/{kind}", get(get_issues_by_kind))
        .route("/stats", get(get_stats))
        .route("/nodes", get(get_nodes).post(post_node))
        .route("/nodes/{id}", delete(delete_node))
        .layer(Extension(api_cache))
        .layer(Extension(api_pool))
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );
    let api_addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    let server_handle = tokio::spawn(async move {
        let listener = tokio::net::TcpListener::bind(api_addr)
            .await
            .expect("failed to bind API listener");
        println!("API server listening on http://{api_addr}");
        axum::serve(listener, app).await.expect("API server failed");
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

    // Payment/invoice tracking loop, every 10s
    let payment_pool = pool.clone();
    let payment_handle = tokio::spawn(async move {
        let mut ticker = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            ticker.tick().await;
            payment_tracker::poll_tracked_payments(&payment_pool).await;
        }
    });

    let _ = tokio::join!(fast_handle, slow_handle, server_handle, payment_handle);
    Ok(())
}
