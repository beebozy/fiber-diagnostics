use anyhow::Result;
use sqlx::SqlitePool;

use crate::diagnostics::rules::node_down::NodeStatus;

pub async fn load_nodes(pool: &SqlitePool) -> Result<Vec<NodeStatus>> {
    let rows = sqlx::query!(
        r#"
        SELECT
            node_id,
            rpc_reachable
        FROM node_status_current
        "#
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .into_iter()
        .map(|r| NodeStatus {
            node_id: r.node_id.expect("node_id should not be null"),
            rpc_reachable: r.rpc_reachable != 0,
        })
        .collect())
}
