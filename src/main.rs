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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;

    let nodes = fetch_monitored_nodes(&pool).await?;
    println!("Found {} monitored nodes\n", nodes.len());

    for node in nodes {
        println!("--- Polling {} ({}) ---", node.id, node.rpc_url);

        let client = FiberRpcClient::new(&node.rpc_url);
        let info = client.node_info().await?;

        sqlx::query(
            "INSERT INTO node_status_current (node_id, pubkey, rpc_reachable, last_polled_at, updated_at)
             VALUES (?, ?, 1, ?, ?)
             ON CONFLICT(node_id) DO UPDATE SET pubkey=excluded.pubkey, rpc_reachable=1, last_polled_at=excluded.last_polled_at, updated_at=excluded.updated_at"
        )
        .bind(&node.id)
        .bind(info["pubkey"].as_str())
        .bind(chrono::Utc::now().to_rfc3339())
        .bind(chrono::Utc::now().to_rfc3339())
        .execute(&pool)
        .await?;

        println!("Inserted {} status into database.\n", node.id);
    }

    Ok(())
}