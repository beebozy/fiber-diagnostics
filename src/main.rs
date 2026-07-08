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

    match client.node_info().await {
        Ok(info) => {
            let result = sqlx::query(
                "INSERT INTO node_status_current (node_id, pubkey, rpc_reachable, rpc_error, last_polled_at, last_success_at, updated_at)
                 VALUES (?, ?, 1, NULL, ?, ?, ?)
                 ON CONFLICT(node_id) DO UPDATE SET
                    pubkey=excluded.pubkey,
                    rpc_reachable=1,
                    rpc_error=NULL,
                    last_polled_at=excluded.last_polled_at,
                    last_success_at=excluded.last_success_at,
                    updated_at=excluded.updated_at"
            )
            .bind(node_id)
            .bind(info["pubkey"].as_str())
            .bind(&now)
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await;

            match result {
                Ok(_) => println!("[{node_id}] OK — pubkey {}", info["pubkey"].as_str().unwrap_or("?")),
                Err(e) => eprintln!("[{node_id}] DB write failed: {e}"),
            }
        }
        Err(e) => {
            eprintln!("[{node_id}] UNREACHABLE: {e}");

            let result = sqlx::query(
                "INSERT INTO node_status_current (node_id, rpc_reachable, rpc_error, last_polled_at, updated_at)
                 VALUES (?, 0, ?, ?, ?)
                 ON CONFLICT(node_id) DO UPDATE SET
                    rpc_reachable=0,
                    rpc_error=excluded.rpc_error,
                    last_polled_at=excluded.last_polled_at,
                    updated_at=excluded.updated_at"
            )
            .bind(node_id)
            .bind(e.to_string())
            .bind(&now)
            .bind(&now)
            .execute(pool)
            .await;

            if let Err(db_err) = result {
                eprintln!("[{node_id}] DB write failed while recording down-state: {db_err}");
            }
        }
    }
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