mod rpc_client;
use rpc_client::FiberRpcClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let pool = sqlx::SqlitePool::connect(&std::env::var("DATABASE_URL")?).await?;
    let client = FiberRpcClient::new("http://127.0.0.1:8227");
    let info = client.node_info().await?;

    sqlx::query(
        "INSERT INTO node_status_current (node_id, pubkey, rpc_reachable, last_polled_at, updated_at)
         VALUES (?, ?, 1, ?, ?)
         ON CONFLICT(node_id) DO UPDATE SET pubkey=excluded.pubkey, rpc_reachable=1, last_polled_at=excluded.last_polled_at, updated_at=excluded.updated_at"
    )
    .bind("node1")
    .bind(info["pubkey"].as_str())
    .bind(chrono::Utc::now().to_rfc3339())
    .bind(chrono::Utc::now().to_rfc3339())
    .execute(&pool)
    .await?;

    println!("Inserted node1 status into database.");

    Ok(())
}