//! Polls FNN for the state of payments already being tracked (i.e. rows
//! that exist in tracked_payments with a known payment_hash) and writes
//! results into payment_status_current / invoice_status_current -- the
//! tables no_route/fee_too_low/asset_mismatch/invoice_expired read from.
//!
//! Does NOT originate payments (send_payment) -- this is the polling half
//! only. tracked_payments rows are seeded by hand for now (same pattern as
//! monitored_nodes rows), until send_payment origination is built.

use crate::rpc_client::FiberRpcClient;

/// Sends a real payment via send_payment and immediately registers the
/// resulting payment_hash in tracked_payments -- the next poll_tracked_payments
/// cycle picks it up automatically. No manual sqlite3 INSERT needed anymore.
pub async fn send_and_track(
    pool: &sqlx::SqlitePool,
    node_id: &str,
    rpc_url: &str,
    invoice: &str,
) -> anyhow::Result<String> {
    let client = FiberRpcClient::new(rpc_url);
    let result = client.send_payment(invoice).await?;
    let payment_hash = result["payment_hash"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("send_payment response missing payment_hash"))?
        .to_string();

    let now = chrono::Utc::now().to_rfc3339();
    sqlx::query(
        "INSERT INTO tracked_payments (payment_hash, node_id, invoice, tracking_status, created_at, updated_at)
         VALUES (?, ?, ?, 'active', ?, ?)
         ON CONFLICT(payment_hash) DO UPDATE SET tracking_status='active', updated_at=excluded.updated_at",
    )
    .bind(&payment_hash)
    .bind(node_id)
    .bind(invoice)
    .bind(&now)
    .bind(&now)
    .execute(pool)
    .await?;

    Ok(payment_hash)
}

#[derive(sqlx::FromRow, Debug)]
struct TrackedPayment {
    payment_hash: String,
    node_id: String,
}

/// node_id -> rpc_url, so each tracked payment can be polled against the
/// right node's client.
async fn load_node_urls(pool: &sqlx::SqlitePool) -> anyhow::Result<std::collections::HashMap<String, String>> {
    #[derive(sqlx::FromRow)]
    struct NodeUrl {
        id: String,
        rpc_url: String,
    }
    let rows = sqlx::query_as::<_, NodeUrl>("SELECT id, rpc_url FROM monitored_nodes WHERE enabled = 1")
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(|r| (r.id, r.rpc_url)).collect())
}

async fn load_tracked_payments(pool: &sqlx::SqlitePool) -> anyhow::Result<Vec<TrackedPayment>> {
    let rows = sqlx::query_as::<_, TrackedPayment>(
        "SELECT payment_hash, node_id FROM tracked_payments WHERE tracking_status = 'active'",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

async fn poll_one(client: &FiberRpcClient, node_id: &str, payment_hash: &str, pool: &sqlx::SqlitePool) {
    let now = chrono::Utc::now().to_rfc3339();

    // --- get_payment -> payment_status_current ---
    match client.get_payment(payment_hash).await {
        Ok(result) => {
            let router_json = result.get("routers").map(|r| r.to_string());
            let _ = sqlx::query(
                "INSERT INTO payment_status_current
                    (payment_hash, node_id, status, failed_error, fee_raw, router_json,
                     created_at_raw, last_updated_at_raw, observed_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                 ON CONFLICT(payment_hash) DO UPDATE SET
                    status=excluded.status, failed_error=excluded.failed_error,
                    fee_raw=excluded.fee_raw, router_json=excluded.router_json,
                    last_updated_at_raw=excluded.last_updated_at_raw,
                    observed_at=excluded.observed_at, updated_at=excluded.updated_at",
            )
            .bind(payment_hash)
            .bind(node_id)
            .bind(result["status"].as_str())
            .bind(result["failed_error"].as_str())
            .bind(result["fee"].as_str())
            .bind(router_json)
            .bind(result["created_at"].as_str())
            .bind(result["last_updated_at"].as_str())
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await;
            println!("[{node_id}] get_payment {payment_hash} -> {:?}", result["status"].as_str());
        }
        Err(e) => eprintln!("[{node_id}] get_payment {payment_hash} failed: {e}"),
    }

    // --- get_invoice -> invoice_status_current ---
    // NOTE: only invoice_status and the raw parsed_invoice_json blob are
    // mapped with confidence right now. currency/expiry_time_raw are
    // deliberately left unmapped (NULL) until a real response confirms the
    // nested invoice field names -- see payment_tracker.rs module comment.
    match client.get_invoice(payment_hash).await {
        Ok(result) => {
            let parsed_invoice_json = result.get("invoice").map(|i| i.to_string());
            let _ = sqlx::query(
                "INSERT INTO invoice_status_current
                    (payment_hash, invoice, invoice_status, parsed_invoice_json, observed_at, updated_at)
                 VALUES (?, ?, ?, ?, ?, ?)
                 ON CONFLICT(payment_hash) DO UPDATE SET
                    invoice=excluded.invoice, invoice_status=excluded.invoice_status,
                    parsed_invoice_json=excluded.parsed_invoice_json,
                    observed_at=excluded.observed_at, updated_at=excluded.updated_at",
            )
            .bind(payment_hash)
            .bind(result["invoice_address"].as_str())
            .bind(result["status"].as_str())
            .bind(parsed_invoice_json)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await;
            println!("[{node_id}] get_invoice {payment_hash} -> {:?}", result["status"].as_str());
        }
        Err(e) => eprintln!("[{node_id}] get_invoice {payment_hash} failed: {e}"),
    }
}

/// One pass: load active tracked_payments, poll each against its node.
pub async fn poll_tracked_payments(pool: &sqlx::SqlitePool) {
    let node_urls = match load_node_urls(pool).await {
        Ok(m) => m,
        Err(e) => {
            eprintln!("payment_tracker: failed to load monitored_nodes: {e}");
            return;
        }
    };

    let tracked = match load_tracked_payments(pool).await {
        Ok(t) => t,
        Err(e) => {
            eprintln!("payment_tracker: failed to load tracked_payments: {e}");
            return;
        }
    };

    for payment in tracked {
        let Some(rpc_url) = node_urls.get(&payment.node_id) else {
            eprintln!(
                "payment_tracker: tracked_payments row {} references unknown/disabled node_id {}",
                payment.payment_hash, payment.node_id
            );
            continue;
        };
        let client = FiberRpcClient::new(rpc_url.as_str());
        poll_one(&client, &payment.node_id, &payment.payment_hash, pool).await;
    }
}
