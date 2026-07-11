use anyhow::Result;
use sqlx::SqlitePool;

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct NodeStatus {
    pub node_id: String,
    pub rpc_reachable: bool,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PeerStatus {
    pub node_id: String,
    pub peer_pubkey: String,
    pub address: String,
    pub connected: bool,
    pub last_seen_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ChannelStatus {
    pub node_id: String,
    pub channel_id: String,
    pub peer_pubkey: String,
    pub channel_outpoint: Option<String>,
    pub is_public: bool,
    pub state_name: String,
    pub state_flags: Option<String>,
    pub enabled: bool,
    pub funding_udt_type_script_json: Option<String>,
    pub local_balance_raw: String,
    pub remote_balance_raw: String,
    pub offered_tlc_balance_raw: String,
    pub received_tlc_balance_raw: String,
    pub latest_commitment_transaction_hash: Option<String>,
    pub tlc_expiry_delta_raw: Option<String>,
    pub tlc_fee_proportional_millionths_raw: Option<String>,
    pub shutdown_transaction_hash: Option<String>,
    pub created_at_raw: Option<String>,
    pub last_seen_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PaymentStatus {
    pub payment_hash: String,
    pub node_id: String,
    pub target_pubkey: Option<String>,
    pub invoice: Option<String>,
    pub amount_raw: Option<String>,
    pub asset_type: Option<String>,
    pub status: String,
    pub failed_error: Option<String>,
    pub fee_raw: Option<String>,
    pub router_json: Option<String>,
    pub created_at_raw: Option<String>,
    pub last_updated_at_raw: Option<String>,
    pub observed_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InvoiceStatus {
    pub payment_hash: String,
    pub invoice: Option<String>,
    pub invoice_status: Option<String>,
    pub amount_raw: Option<String>,
    pub currency: Option<String>,
    pub expiry_time_raw: Option<String>,
    pub final_expiry_delta_raw: Option<String>,
    pub udt_type_script_json: Option<String>,
    pub parsed_invoice_json: Option<String>,
    pub observed_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct DiagnosticsData {
    pub nodes: Vec<NodeStatus>,
    pub peers: Vec<PeerStatus>,
    pub channels: Vec<ChannelStatus>,
    pub payments: Vec<PaymentStatus>,
    pub invoices: Vec<InvoiceStatus>,
}

pub async fn load_data(pool: &SqlitePool) -> Result<DiagnosticsData> {
    let nodes =
        sqlx::query_as::<_, NodeStatus>("SELECT node_id, rpc_reachable FROM node_status_current")
            .fetch_all(pool)
            .await?;

    let peers = sqlx::query_as::<_, PeerStatus>(
        "SELECT node_id, peer_pubkey, address, connected, last_seen_at, updated_at FROM peer_status_current",
    )
    .fetch_all(pool)
    .await?;

    let channels = sqlx::query_as::<_, ChannelStatus>(
        "SELECT node_id, channel_id, peer_pubkey, channel_outpoint, is_public, state_name, state_flags, enabled, funding_udt_type_script_json, local_balance_raw, remote_balance_raw, offered_tlc_balance_raw, received_tlc_balance_raw, latest_commitment_transaction_hash, tlc_expiry_delta_raw, tlc_fee_proportional_millionths_raw, shutdown_transaction_hash, created_at_raw, last_seen_at, updated_at FROM channel_status_current",
    )
    .fetch_all(pool)
    .await?;

    let payments = sqlx::query_as::<_, PaymentStatus>(
        "SELECT payment_hash, node_id, target_pubkey, invoice, amount_raw, asset_type, status, failed_error, fee_raw, router_json, created_at_raw, last_updated_at_raw, observed_at, updated_at FROM payment_status_current",
    )
    .fetch_all(pool)
    .await?;

    let invoices = sqlx::query_as::<_, InvoiceStatus>(
        "SELECT payment_hash, invoice, invoice_status, amount_raw, currency, expiry_time_raw, final_expiry_delta_raw, udt_type_script_json, parsed_invoice_json, observed_at, updated_at FROM invoice_status_current",
    )
    .fetch_all(pool)
    .await?;

    Ok(DiagnosticsData {
        nodes,
        peers,
        channels,
        payments,
        invoices,
    })
}
