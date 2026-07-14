use crate::diagnostics::issue::Issue;
use crate::diagnostics::repository::DiagnosticsData;
use crate::diagnostics::rules::{
    asset_mismatch,
    channel_not_ready,
    fee_too_low,
    invoice_expired,
    insufficient_balance,
    no_route,
    node_down,
    peer_offline,
};

pub struct DiagnosticsEngine;

impl DiagnosticsEngine {
    pub fn evaluate(data: DiagnosticsData) -> Vec<Issue> {
        let mut issues = Vec::new();

        for node in &data.nodes {
            if let Some(issue) = node_down::evaluate(node) {
                issues.push(issue);
            }
        }

        for peer in &data.peers {
            if let Some(issue) = peer_offline::evaluate(peer) {
                issues.push(issue);
            }
        }

        for channel in &data.channels {
            if let Some(issue) = channel_not_ready::evaluate(channel) {
                issues.push(issue);
            }
            if let Some(issue) = insufficient_balance::evaluate(channel) {
                issues.push(issue);
            }
        }

        let payment_node_ids: std::collections::HashMap<&str, &str> = data
            .payments
            .iter()
            .map(|payment| (payment.payment_hash.as_str(), payment.node_id.as_str()))
            .collect();

        for invoice in &data.invoices {
            if let Some(issue) = invoice_expired::evaluate(invoice, &payment_node_ids) {
                issues.push(issue);
            }
        }

        for payment in &data.payments {
            if let Some(issue) = no_route::evaluate(payment) {
                issues.push(issue);
            }
            if let Some(issue) = fee_too_low::evaluate(payment) {
                issues.push(issue);
            }
        }

        issues.extend(asset_mismatch::evaluate(&data.payments, &data.invoices, &data.channels));

        issues
    }
}
